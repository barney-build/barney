// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Schema validated document view

use std::collections::VecDeque;

use crate::diagnostics::{SchemaDiagnostic, reporting};
use crate::value::ValueError;
use crate::{
    AsBnyValueType, BnySchema, BnyValueType, NodeSpec, PropertySpec, SchemaIndex, SchemaKey,
    compiled::{BindingReference, BnyRuleRef},
    registry::LinkedSchemaRegistry,
    spec::RuleSpec,
    value::{self},
};
use bny_diag::DiagnosticSink;
use bny_slots::{Block, LinkedSlotStorage, SlotStorage};
use bny_syntax::{BnyInterner, BnyNodeRef, BnyPropertyRef, BnyTree, ProcessedValue};
use indextree::{Arena, NodeId};
use itertools::{Either, Itertools};
use thiserror::Error;

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
}

#[derive(Debug, Clone, Copy)]
enum PropertyMatchResult {
    Matched { prop_ref: BnyPropertyRef },
    Unmatched,
}

impl<'a> ValidatedDocument<'a> {
    fn property_processor(
        node: &BnyNodeRef,
        property: &PropertySpec,
        tree: &BnyTree<'a>,
        sink: &mut DiagnosticSink,
    ) -> PropertyMatchResult {
        for disc_prop in node.properties(tree) {
            if let Some(prop_id) = property.id
                && prop_id != disc_prop.name(tree)
            {
                continue;
            }
            let stored_kind = match disc_prop.value(tree) {
                ProcessedValue::Boolean(_) => BnyValueType::Bool,
                ProcessedValue::Float(_) => BnyValueType::Float,
                ProcessedValue::String(_) => BnyValueType::String,
                ProcessedValue::Integer(i) => {
                    // we kinda have to accept the wanted integer kind at this point
                    let conversion = match property.kind {
                        // i32
                        BnyValueType::Int32 => i32::try_from(*i)
                            .map(|_| BnyValueType::Int32)
                            .map_err(ValueError::ConvertInt),
                        // u32
                        BnyValueType::Uint32 => u32::try_from(*i)
                            .map(|_| BnyValueType::Uint32)
                            .map_err(ValueError::ConvertInt),
                        // i64
                        BnyValueType::Int64 => i64::try_from(*i)
                            .map(|_| BnyValueType::Int64)
                            .map_err(ValueError::ConvertInt),
                        // u64
                        BnyValueType::Uint64 => u64::try_from(*i)
                            .map(|_| BnyValueType::Uint64)
                            .map_err(ValueError::ConvertInt),
                        _ => Ok(BnyValueType::Unknown),
                    };
                    match conversion {
                        Ok(c) => c,
                        Err(e) => {
                            let id = sink.insert_error(e, None);
                            sink.push_span(id, disc_prop.value_span(tree), reporting::PROPERTY);
                            BnyValueType::Unknown
                        }
                    }
                }
                ProcessedValue::Null => BnyValueType::Unknown,
            };

            if stored_kind != property.kind {
                let id = sink.insert_error(
                    SchemaDiagnostic::WrongValueType {
                        schema: property.kind.to_string(),
                        found: stored_kind.to_string(),
                    },
                    None,
                );
                sink.push_span(id, disc_prop.value_span(tree), reporting::PROPERTY);
                return PropertyMatchResult::Unmatched;
            }

            return PropertyMatchResult::Matched {
                prop_ref: disc_prop,
            };
        }
        PropertyMatchResult::Unmatched
    }

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

                    // Reserve property slots
                    let n_prop = rule.properties(schema, &node_spec).count();
                    let p_block = property_matches.allocate(n_prop, || None);

                    // Find all matching property rules
                    for prop in rule.properties(schema, &node_spec) {
                        let res = Self::property_processor(&node, prop, tree, sink);
                        let stored = match res {
                            PropertyMatchResult::Matched { prop_ref } => Some(prop_ref),
                            PropertyMatchResult::Unmatched => None,
                        };
                        property_matches.set(p_block.0.start + prop.local_index, stored);
                    }

                    let slot_range = if let Some(parent) = frame.arena_base {
                        arena[parent.0].get().child_block
                    } else {
                        root_slots
                    };

                    // Rule tracking pairs the parent-context slot storage with the rule ids for nodespecs
                    if let Some(guid) = node_spec.global_index.as_ref() {
                        rule_tracking.push(rule_block, *guid, Some(slot_range));
                    }

                    let child_block = slots.allocate(our_child_count);

                    let matched_node = Match {
                        child_block,
                        node: frame.node,
                        schema: child_schema,
                        props: p_block,
                        prop_defs: node_spec.properties,
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
                    let id = sink.insert_error(
                        SchemaDiagnostic::IllegalNode {
                            id: intern.resolve(frame.node.name(tree)).to_owned(),
                        },
                        None,
                    );
                    sink.push_span(id, frame.node.span(tree), reporting::FIRST_SEEN);
                }
            }
        }

        for rule_id in 0..registry.global_nodes() {
            for parent_set in rule_tracking.iter(rule_block, rule_id).flatten() {
                let rule_children = slots.iter(*parent_set, rule_id).count();
                if let Some(RuleSpec::MatchNode(n)) = registry.global_node(rule_id)
                    && let Err(e) = n.occurrence.validate(rule_children as u8)
                {
                    let diag = sink.insert_error(e, None);
                    for (child_idx, child) in slots.iter(*parent_set, rule_id).enumerate() {
                        let node = arena[child.0].get();
                        let label = if child_idx == 0 {
                            reporting::FIRST_SEEN
                        } else {
                            reporting::ALSO_SEEN
                        };
                        sink.push_span(diag, node.node.span(tree), label);
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

    fn get_property<K: AsBnyValueType<'a>>(
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

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("cannot find key {key} in the active schema")]
    InvalidBinding { key: String },

    #[error("using key {key} in the wrong context ")]
    InvalidContext { key: String },

    #[error("symbolic {key} does not point to a node rule")]
    ExpectedNodeSymbolic { key: String },

    #[error("symbolic {key} does not point to a property rule")]
    ExpectedPropertySymbolic { key: String },

    #[error(transparent)]
    ValueError(#[from] value::ValueError),

    #[error("wrong return type, schema expects {schema_type}, not {found_type}")]
    WrongReturnType {
        schema_type: BnyValueType,
        found_type: BnyValueType,
    },

    #[error("wrong cardinality applied")]
    WrongCardinality,

    #[error("properties not supported for this node")]
    NoProperties,
}

/// Encapsulates matching
#[derive(Debug)]
pub struct ValidatedNode<'a> {
    matched: MatchId,
    doc: &'a ValidatedDocument<'a>,
    schema: SchemaIndex,
}

impl<'a> ValidatedNode<'a> {
    // Child level schema nodes
    pub fn nodes<T: SchemaKey<'a>>(
        &self,
        key: T,
    ) -> Result<impl Iterator<Item = ValidatedNode<'a>>, ValidationError> {
        self.doc
            .get_nodes_for_parent(Some(self.matched), key, self.schema)
    }

    // Grab the name
    pub fn name(&self) -> &'a str {
        let m = self.doc.arena[self.matched.0].get();
        self.doc.intern.resolve(m.node.name(self.doc.tree))
    }

    pub fn property<T: AsBnyValueType<'a>>(
        &self,
        identity: impl SchemaKey<'a>,
    ) -> Result<T, ValidationError> {
        self.doc.get_property(self.matched, self.schema, identity)
    }
}

enum ValidationResult {
    // matched a rule
    Matched {
        rule: BnyRuleRef,
        index: usize,
        node_spec: NodeSpec,
        node: BnyNodeRef,
    },
    // unmatched, try someone else
    Unmatched,
    // change schema validation context
    ChangeContext(SchemaIndex),
}
