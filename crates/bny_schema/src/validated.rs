// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Schema validated document view

mod slots;

use std::{iter, range::Range};

use crate::{
    BnySchema, SchemaIndex, SchemaKey, compiled::BnyRuleRef, registry::LinkedSchemaRegistry,
    spec::RuleSpec, validated::slots::ChildSlotStorage,
};
use bny_syntax::{BnyInterner, BnyNodeRef, BnyTree};
use indextree::{Arena, NodeId};
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
    children: Range<usize>,
    node: BnyNodeRef,
    next: Option<MatchId>,
    schema: SchemaIndex,
}

/// Validated document is a read only view of the
/// schema validated input
#[derive(Debug)]
pub struct ValidatedDocument<'a, 'b> {
    registry: &'a LinkedSchemaRegistry,
    tree: &'a BnyTree<'b>,

    slots: ChildSlotStorage,
    arena: Arena<Match>,
    root_slots: Range<usize>,
    intern: &'a BnyInterner,
}

impl<'a, 'b> ValidatedDocument<'a, 'b> {
    fn process_rules(
        rules: impl Iterator<Item = BnyRuleRef>,
        schema: &'a BnySchema,
        tree: &'a BnyTree<'b>,
        frame: ValidationFrame,
    ) -> ValidationResult {
        let frame_name = frame.node.name(tree);

        // TODO: Loop twice by filter to ensure wildcard is last
        for rule in rules {
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

    /// Return a document view with all validation performed
    /// TODO: Also store all diagnostics along the way and validate bounds/occurence
    pub(super) fn new(
        registry: &'a LinkedSchemaRegistry,
        tree: &'a BnyTree<'b>,
        intern: &'a BnyInterner,
    ) -> Self {
        // Initial queue of root level rules + nodes
        let mut work_queue: Vec<ValidationFrame> = Vec::new();
        let mut arena: Arena<Match> = Arena::new();

        work_queue.extend(tree.nodes().map(|n| ValidationFrame {
            node: n,
            rule_base: None,
            schema_id: registry.entry,
            arena_base: None,
        }));

        // Total storage - the root rules
        let n_roots = registry.entry().n_rules;
        let root_slots: Range<usize> = Range {
            start: 0,
            end: n_roots,
        };
        let mut slots = ChildSlotStorage::new(n_roots);

        while let Some(frame) = work_queue.pop() {
            let schema = &registry.schemas[frame.schema_id.0];
            let result = if let Some(rule_base) = &frame.rule_base {
                Self::process_rules(rule_base.children(schema), schema, tree, frame)
            } else {
                Self::process_rules(schema.rules(), schema, tree, frame)
            };

            match result {
                // explicit match, descend into this node
                ValidationResult::Matched { rule, index } => {
                    // Reserve range for children rules
                    let our_child_count = rule.children(schema).count();
                    let child_range: Range<usize> = Range {
                        start: slots.len(),
                        end: slots.len() + our_child_count,
                    };
                    slots.reserve(our_child_count);

                    let matched_node = Match {
                        children: child_range,
                        node: frame.node,
                        next: None,
                        schema: frame.schema_id,
                    };

                    // Virtual root nodes or nested?
                    let (slot_range, parent_node) = if let Some(parent) = frame.arena_base {
                        (
                            arena[parent.0].get().children,
                            parent.0.append_value(matched_node, &mut arena),
                        )
                    } else {
                        (root_slots, arena.new_node(matched_node))
                    };

                    let slot_index = slot_range.start + index;
                    let head = slots.get_mut(slot_index);
                    if let Some(last) = head.last {
                        arena[last.0].get_mut().next = Some(MatchId(parent_node));
                    } else {
                        head.first = Some(MatchId(parent_node));
                    };
                    head.last = Some(MatchId(parent_node));

                    work_queue.extend(frame.node.children(tree).map(move |n| ValidationFrame {
                        node: n,
                        schema_id: frame.schema_id,
                        rule_base: Some(rule),
                        arena_base: Some(MatchId(parent_node)),
                    }));
                }
                // change context for processing children
                ValidationResult::ChangeContext(schema_index) => work_queue.push(ValidationFrame {
                    node: frame.node,
                    schema_id: schema_index,
                    rule_base: None,
                    arena_base: frame.arena_base,
                }),
                // no rules matched, don't bother descending
                ValidationResult::Unmatched => {}
            }
        }

        ValidatedDocument {
            registry,
            tree,
            intern,
            slots,
            arena,
            root_slots,
        }
    }

    fn get_nodes_for_parent<T: SchemaKey<'a>>(
        &'a self,
        parent: Option<MatchId>,
        key: T,
        schema: SchemaIndex,
    ) -> Result<impl Iterator<Item = ValidatedNode<'a, 'b>>, ValidationError> {
        let lookup = self.registry.schemas[schema.0]
            .resolve_symbolic(key.symbolic_id())
            .ok_or_else(|| ValidationError::InvalidBinding {
                key: format!("{:?}", key),
            })?;

        // Can't grab nodes from root if it has a parent in this accessor..
        let rule_offset = match parent {
            Some(p) => self.arena[p.0].get().children,
            None => self.root_slots,
        };

        // root context, *wrong* suckah.
        if lookup.parent.is_some() && parent.is_none() {
            return Err(ValidationError::InvalidContext {
                key: format!("{:?}", key),
            });
        }

        let itere = self
            .slots
            .get(rule_offset.start + lookup.local_offset as usize)
            .and_then(|l| l.first);

        let result =
            iter::successors(itere, |f| self.arena[f.0].get().next).map(|n| ValidatedNode {
                matched: n,
                doc: self,
                schema: self.arena[n.0].get().schema,
            });

        Ok(result)
    }

    // Root level schema nodes
    pub fn nodes<T: SchemaKey<'a>>(
        &'a self,
        key: T,
    ) -> Result<impl Iterator<Item = ValidatedNode<'a, 'b>>, ValidationError> {
        self.get_nodes_for_parent(None, key, self.registry.entry)
    }
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("cannot find key {key} in the active schema")]
    InvalidBinding { key: String },

    #[error("using key {key} in the wrong context ")]
    InvalidContext { key: String },

    #[error("derp")]
    Derp,
}

/// Encapsulates matching
#[derive(Debug)]
pub struct ValidatedNode<'a, 'b> {
    matched: MatchId,
    doc: &'a ValidatedDocument<'a, 'b>,
    schema: SchemaIndex,
}

impl<'a, 'b> ValidatedNode<'a, 'b> {
    // Child level schema nodes
    pub fn nodes<T: SchemaKey<'a>>(
        &'a self,
        key: T,
    ) -> Result<impl Iterator<Item = ValidatedNode<'a, 'b>>, ValidationError> {
        self.doc
            .get_nodes_for_parent(Some(self.matched), key, self.schema)
    }

    // Grab the name
    pub fn name(&self) -> &'a str {
        let m = self.doc.arena[self.matched.0].get();
        self.doc.intern.resolve(m.node.name(self.doc.tree))
    }
}

enum ValidationResult {
    // matched a rule
    Matched { rule: BnyRuleRef, index: usize },
    // unmatched, try someone else
    Unmatched,
    // change schema validation context
    ChangeContext(SchemaIndex),
}
