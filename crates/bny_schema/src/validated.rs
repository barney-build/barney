// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Schema validated document view

mod errors;
pub use errors::ValidationError;
mod node;
pub use node::ValidatedNode;

// helpers
mod args;
mod props;

use std::collections::VecDeque;

use crate::argument::ArgumentKind;
use crate::diagnostics::{SchemaDiagnostic, reporting};
use crate::validated::node::ValidationResult;
use crate::{AsBnyValueType, AsOptionalBnyValueType};
use crate::{
    BnySchema, SchemaIndex, SchemaKey,
    compiled::{BindingReference, BnyRuleRef},
    registry::LinkedSchemaRegistry,
    spec::RuleSpec,
};
use bny_diag::DiagnosticSink;
use bny_slots::{Block, LinkedSlotStorage, SlotStorage};
use bny_syntax::{BnyArgumentRef, BnyInterner, BnyNodeRef, BnyPropertyRef, BnyTree};
use indextree::{Arena, NodeId};
use itertools::{Either, Itertools};

/// Every node is paired to a contextual set of rules
/// If they're not explicitly provided then the rules
/// are extracted from the schema base
#[derive(Debug, Clone, Copy)]
struct ValidationFrame {
    node: BnyNodeRef,
    schema_id: SchemaIndex,
    rule_base: Option<BnyRuleRef>,
    arena_base: Option<MatchId>,
}

// Matching in our arena overlay
#[derive(Debug, Clone, Copy)]
struct MatchId(pub NodeId);

#[derive(Debug)]
struct Match {
    child_block: Block,
    node: BnyNodeRef,
    schema: SchemaIndex,
    props: Block,
    prop_defs: Option<Block>,
    args: Block,
    arg_defs: Option<Block>,
}

/// Validated document is a read only view of the
/// schema validated input
#[derive(Debug)]
pub struct ValidatedDocument<'a> {
    registry: &'a LinkedSchemaRegistry,
    tree: &'a BnyTree<'a>,

    slots: LinkedSlotStorage<MatchId>,
    arena: Arena<Match>,
    root_slots: Block,
    intern: &'a BnyInterner,

    property_matches: SlotStorage<Option<BnyPropertyRef>>,
    argument_matches: LinkedSlotStorage<BnyArgumentRef>,
}

impl<'a> ValidatedDocument<'a> {
    fn process_rules_internal(
        rules: impl Iterator<Item = BnyRuleRef>,
        schema: &BnySchema,
        tree: &BnyTree<'a>,
        frame: ValidationFrame,
    ) -> ValidationResult {
        let frame_name = frame.node.name(tree);

        for rule in rules.into_iter() {
            match rule.rule(schema) {
                RuleSpec::MatchNode(node_spec) => {
                    // If there is an explicit name set, and doesn't match, skip it.
                    if let Some(id) = node_spec.id
                        && id != frame_name
                    {
                        continue;
                    }

                    return ValidationResult::Matched {
                        rule,
                        index: node_spec.local_index,
                        node_spec: *node_spec,
                        node: frame.node,
                    };
                }
                // request a context change for children
                RuleSpec::ExternalSchemaIndexed(schema_index) => {
                    return ValidationResult::ChangeContext(*schema_index);
                }
                _ => {}
            };
        }

        ValidationResult::Unmatched
    }

    fn process_rules(
        rules: impl Iterator<Item = BnyRuleRef>,
        schema: &BnySchema,
        tree: &BnyTree<'a>,
        frame: ValidationFrame,
    ) -> ValidationResult {
        let (normal, wildcard): (Vec<_>, Vec<_>) =
            rules.into_iter().partition_map(|p| match p.rule(schema) {
                RuleSpec::MatchNode(node_spec) => {
                    if node_spec.id.is_some() {
                        Either::Left(p)
                    } else {
                        Either::Right(p)
                    }
                }
                _ => Either::Left(p),
            });
        match Self::process_rules_internal(normal.into_iter(), schema, tree, frame) {
            ValidationResult::Unmatched => {
                Self::process_rules_internal(wildcard.into_iter(), schema, tree, frame)
            }
            res => res,
        }
    }

    /// Chaining children slots
    fn count_slot_storage(schema: &BnySchema, rules: impl Iterator<Item = BnyRuleRef>) -> usize {
        rules
            .filter_map(|r| match r.rule(schema) {
                RuleSpec::MatchNode(node_spec) => Some(node_spec.local_index + 1),
                _ => None,
            })
            .max()
            .unwrap_or(0)
    }

    fn child_context(
        registry: &LinkedSchemaRegistry,
        schema_id: SchemaIndex,
        schema: &BnySchema,
        rules: impl Iterator<Item = BnyRuleRef>,
    ) -> (SchemaIndex, usize) {
        let mut count = 0;

        for rule in rules {
            match rule.rule(schema) {
                RuleSpec::MatchNode(node_spec) => {
                    count = count.max(node_spec.local_index + 1);
                }
                RuleSpec::ExternalSchemaIndexed(index) => {
                    let child_schema = &registry.schemas[index.0];

                    return (
                        *index,
                        Self::count_slot_storage(child_schema, child_schema.rules()),
                    );
                }
                _ => {}
            }
        }

        (schema_id, count)
    }

    /// Return a document view with all validation performed
    pub(super) fn new(
        registry: &'a LinkedSchemaRegistry,
        tree: &'a BnyTree<'a>,
        intern: &'a BnyInterner,
        sink: &mut DiagnosticSink,
    ) -> Self {
        // Initial queue of root level rules + nodes
        let mut work_queue: VecDeque<ValidationFrame> = VecDeque::new();
        let mut arena: Arena<Match> = Arena::new();

        work_queue.extend(tree.nodes().map(|n| ValidationFrame {
            node: n,
            rule_base: None,
            schema_id: registry.entry,
            arena_base: None,
        }));

        // Total storage - the root rules
        let n_roots = Self::count_slot_storage(registry.entry(), registry.entry().rules());

        let mut slots = LinkedSlotStorage::new();
        let root_slots = slots.allocate(n_roots);
        let mut property_matches = SlotStorage::new();
        let mut argument_matches = LinkedSlotStorage::new();

        let mut rule_tracking = LinkedSlotStorage::new();
        let rule_block = rule_tracking.allocate(registry.global_nodes());

        while let Some(frame) = work_queue.pop_front() {
            let schema = &registry.schemas[frame.schema_id.0];
            let result = if let Some(rule_base) = &frame.rule_base {
                Self::process_rules(rule_base.children(schema), schema, tree, frame)
            } else {
                Self::process_rules(schema.rules(), schema, tree, frame)
            };

            match result {
                // explicit match, descend into this node
                ValidationResult::Matched {
                    rule,
                    index,
                    node_spec,
                    node,
                } => {
                    // Reserve range for children rules
                    let (child_schema, our_child_count) = Self::child_context(
                        registry,
                        frame.schema_id,
                        schema,
                        rule.children(schema),
                    );

                    // hook up arguments
                    let a_block = args::validate(
                        &mut argument_matches,
                        tree,
                        schema,
                        rule,
                        &node_spec,
                        &node,
                        sink,
                    );

                    // hook up properties
                    let p_block = props::validate(
                        &mut property_matches,
                        tree,
                        schema,
                        rule,
                        &node_spec,
                        &node,
                        sink,
                    );

                    let slot_range = if let Some(parent) = frame.arena_base {
                        arena[parent.0].get().child_block
                    } else {
                        root_slots
                    };

                    let child_block = slots.allocate(our_child_count);

                    // Rule tracking pairs the parent-context slot storage with the rule ids for nodespecs
                    if let Some(guid) = node_spec.global_index.as_ref() {
                        rule_tracking.push(rule_block, *guid, Some(slot_range));
                    }

                    let matched_node = Match {
                        child_block,
                        node: frame.node,
                        schema: child_schema,
                        props: p_block,
                        prop_defs: node_spec.properties,
                        args: a_block,
                        arg_defs: node_spec.arguments,
                    };

                    // Virtual root nodes or nested?
                    let parent_node = if let Some(parent) = frame.arena_base {
                        parent.0.append_value(matched_node, &mut arena)
                    } else {
                        arena.new_node(matched_node)
                    };

                    // Do not push empty blocks!
                    if !slot_range.is_empty() {
                        slots.push(slot_range, index, MatchId(parent_node));
                    }

                    work_queue.extend(frame.node.children(tree).map(move |n| ValidationFrame {
                        node: n,
                        schema_id: frame.schema_id,
                        rule_base: Some(rule),
                        arena_base: Some(MatchId(parent_node)),
                    }));
                }
                // change context for processing children
                ValidationResult::ChangeContext(schema_index) => {
                    work_queue.push_back(ValidationFrame {
                        node: frame.node,
                        schema_id: schema_index,
                        rule_base: None,
                        arena_base: frame.arena_base,
                    })
                }
                // no rules matched, don't bother descending
                ValidationResult::Unmatched => {
                    // We have to retain original input structure so just ignore empty nodes (whitespace)
                    let resolved = intern.resolve(frame.node.name(tree));
                    if resolved.is_empty() {
                        continue;
                    }
                    let id = sink.insert_error(
                        SchemaDiagnostic::IllegalNode {
                            id: intern.resolve(frame.node.name(tree)).to_string(),
                        },
                        None,
                    );
                    sink.push_span(id, frame.node.span(tree), reporting::FIRST_SEEN);
                }
            }
        }

        let mut frames: VecDeque<(BnyRuleRef, SchemaIndex, Block, Option<MatchId>)> =
            VecDeque::new();
        frames.extend(
            registry
                .entry()
                .rules()
                .map(|r| (r, registry.entry, root_slots, None)),
        );

        while let Some((rule, index, block, parent_match)) = frames.pop_front() {
            let schema = &registry.schemas[index.0];
            if let RuleSpec::MatchNode(node_spec) = rule.rule(schema) {
                let occurs = slots.iter(block, node_spec.local_index).count() as u8;
                if let Err(e) = node_spec.occurrence.validate(occurs) {
                    let id = node_spec.id.map(|s| intern.resolve(s)).unwrap_or_default();

                    match occurs {
                        0 => {
                            let id = sink.insert_error(
                                e,
                                Some(&format!("Cannot find required node `{id}`!")),
                            );
                            if let Some(parent) = parent_match {
                                let node = arena[parent.0].get();
                                sink.push_span(id, node.node.span(tree), reporting::FIRST_SEEN);
                            }
                        }
                        _ => {
                            let root =
                                sink.insert_error(e, Some("Violated occurrence constraints"));
                            for item in slots.iter(block, node_spec.local_index) {
                                let node = arena[item.0].get();
                                sink.push_span(root, node.node.span(tree), reporting::FIRST_SEEN);
                            }
                        }
                    }
                }

                // For all of the children we'll now walk all of the rules :3
                for child in slots.iter(block, node_spec.local_index) {
                    let node = arena[child.0].get();
                    for arg in rule.arguments(schema, node_spec) {
                        if !arg.required {
                            continue;
                        }
                        let mapped = argument_matches.iter(node.args, arg.local_index).count();
                        if mapped < 1 {
                            let id = sink.insert(
                                miette::Severity::Error,
                                &format!("Missing argument {}", arg.local_index),
                                Some(&format!(
                                    "Provide argument `{}` on the `{}` node",
                                    arg.local_index,
                                    intern.resolve(node.node.name(tree))
                                )),
                            );
                            sink.push_span(id, node.node.span(tree), reporting::FIRST_SEEN);
                        }
                    }
                    for prop in rule.properties(schema, node_spec) {
                        if !prop.required {
                            continue;
                        }
                        let mapped = property_matches.get(node.props.0.start + prop.local_index);
                        if mapped.is_none() {
                            let id = node_spec.id.map(|s| intern.resolve(s)).unwrap_or_default();
                            let prop_id = prop.id.map(|p| intern.resolve(p)).unwrap_or_default();
                            let id = sink.insert(
                                miette::Severity::Error,
                                &format!("Missing required property {prop_id} on {id}"),
                                None,
                            );
                            sink.push_span(id, node.node.span(tree), reporting::PROPERTY);
                        }
                    }

                    for rule in rule.children(schema) {
                        match rule.rule(schema) {
                            RuleSpec::ExternalSchemaIndexed(schema_index) => {
                                let fog = &registry.schemas[schema_index.0];
                                frames.extend(
                                    fog.rules().map(|r| {
                                        (r, *schema_index, node.child_block, Some(*child))
                                    }),
                                );
                            }
                            _ => {
                                frames.push_back((rule, index, node.child_block, Some(*child)));
                            }
                        }
                    }
                }
            }
        }

        ValidatedDocument {
            registry,
            tree,
            intern,
            slots,
            arena,
            root_slots,
            property_matches,
            argument_matches,
        }
    }

    fn get_nodes_for_parent<T: SchemaKey<'a>>(
        &'a self,
        parent: Option<MatchId>,
        key: T,
        schema: SchemaIndex,
    ) -> Result<impl Iterator<Item = ValidatedNode<'a>>, ValidationError> {
        let lookup = self.registry.schemas[schema.0]
            .resolve_symbolic(key.symbolic_id())
            .ok_or_else(|| ValidationError::InvalidBinding {
                key: format!("{:?}", key),
            })?;

        // Ensure its a node binding
        let (matched_parent, local_offset) = match lookup {
            BindingReference::Node {
                parent,
                local_offset,
            } => (parent, local_offset),
            _ => {
                return Err(ValidationError::ExpectedNodeSymbolic {
                    key: format!("{:?}", key),
                });
            }
        };

        // Can't grab nodes from root if it has a parent in this accessor..
        let head_block = match parent {
            Some(p) => self.arena[p.0].get().child_block,
            None => self.root_slots,
        };

        // root context, *wrong* suckah.
        if matched_parent.is_some() && parent.is_none() {
            return Err(ValidationError::InvalidContext {
                key: format!("{:?}", key),
            });
        }

        let result = self
            .slots
            .iter(head_block, local_offset as usize)
            .map(|m| ValidatedNode {
                matched: *m,
                doc: self,
                schema: self.arena[m.0].get().schema,
            });

        Ok(result)
    }

    // Root level schema nodes
    pub fn nodes<T: SchemaKey<'a>>(
        &'a self,
        key: T,
    ) -> Result<impl Iterator<Item = ValidatedNode<'a>>, ValidationError> {
        self.get_nodes_for_parent(None, key, self.registry.entry)
    }

    fn get_argument<K: AsBnyValueType<'a>>(
        &'a self,
        m: MatchId,
        schema: SchemaIndex,
        key: impl SchemaKey<'a>,
    ) -> Result<K, ValidationError> {
        let arg = self
            .get_arguments::<K>(m, schema, key, false)?
            .next()
            .ok_or(ValidationError::NoArguments)?;
        Ok(arg)
    }

    fn get_arguments<K: AsBnyValueType<'a>>(
        &'a self,
        m: MatchId,
        schema: SchemaIndex,
        key: impl SchemaKey<'a>,
        variadic: bool,
    ) -> Result<impl Iterator<Item = K>, ValidationError> {
        let node = self.arena[m.0].get();
        let arg_defs = node.arg_defs.ok_or(ValidationError::NoArguments)?;

        let lookup = self.registry.schemas[schema.0]
            .resolve_symbolic(key.symbolic_id())
            .ok_or_else(|| ValidationError::InvalidBinding {
                key: format!("{:?}", key),
            })?;

        let (_, local_offset) = match lookup {
            BindingReference::Argument { id, local_offset } => (id, local_offset),
            _ => {
                return Err(ValidationError::ExpectedArgumentSymbolic {
                    key: format!("{:?}", key),
                });
            }
        };

        let arg_rule = self.registry.schemas[schema.0]
            .args
            .get(arg_defs.0.start + local_offset as usize);

        // ensure we respect variadic args
        match arg_rule.arg_kind {
            ArgumentKind::Positional => {
                if variadic {
                    return Err(ValidationError::InvalidBinding {
                        key: format!("{:?}", key),
                    });
                }
            }
            ArgumentKind::Variadic => {
                if !variadic {
                    return Err(ValidationError::InvalidBinding {
                        key: format!("{:?}", key),
                    });
                }
            }
        }

        let m = self
            .argument_matches
            .iter(node.args, local_offset as usize)
            .map(|m| m.value(self.tree))
            .flat_map(|v| K::from_processed_value(Some(v)).ok());
        Ok(m)
    }

    fn get_property<K: AsOptionalBnyValueType<'a>>(
        &self,
        m: MatchId,
        schema: SchemaIndex,
        key: impl SchemaKey<'a>,
    ) -> Result<K, ValidationError> {
        let node = self.arena[m.0].get();
        let prop_defs = node.prop_defs.ok_or(ValidationError::NoProperties)?;

        let lookup = self.registry.schemas[schema.0]
            .resolve_symbolic(key.symbolic_id())
            .ok_or_else(|| ValidationError::InvalidBinding {
                key: format!("{:?}", key),
            })?;

        let (_, local_offset) = match lookup {
            BindingReference::Property { id, local_offset } => (id, local_offset),
            _ => {
                return Err(ValidationError::ExpectedPropertySymbolic {
                    key: format!("{:?}", key),
                });
            }
        };

        let property_rule = self.registry.schemas[schema.0]
            .props
            .get(prop_defs.0.start + local_offset as usize);

        if property_rule.required != K::REQUIRED {
            return Err(ValidationError::WrongCardinality);
        }

        // verify type access
        if property_rule.kind != K::TYPE {
            return Err(ValidationError::WrongReturnType {
                schema_type: property_rule.kind,
                found_type: K::TYPE,
            });
        }

        let slot_idx = node.props.0.start + local_offset as usize;
        let property = self.property_matches.get(slot_idx);
        let value = property.map(|p| p.value(self.tree));
        Ok(K::from_processed_value(value)?)
    }
}
