// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Schema validated document view

use crate::{
    BnySchema, SchemaIndex, compiled::BnyRuleRef, registry::LinkedSchemaRegistry, spec::RuleSpec,
};
use bny_syntax::{BnyNodeRef, BnyTree};

/// Every node is paired to a contextual set of rules
/// If they're not explicitly provided then the rules
/// are extracted from the schema base
#[derive(Debug, Clone, Copy)]
struct ValidationFrame {
    node: BnyNodeRef,
    schema_id: SchemaIndex,
    rule_base: Option<BnyRuleRef>,
}

/// Validated document is a read only view of the
/// schema validated input
#[derive(Debug)]
pub struct ValidatedDocument<'a, 'b> {
    pub(super) _registry: &'a LinkedSchemaRegistry,
    pub(super) _tree: &'a BnyTree<'b>,
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

                    // TODO: Store into indextree arena the node as sibling if necessary
                    // along with properties/argument indices and then validate
                    return ValidationResult::Matched(rule);
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
    /// TODO: Store access indices for sibling oriented data as parallel lookup
    /// graph
    /// TODO: Also store all diagnostics along the way.
    pub(super) fn new(registry: &'a LinkedSchemaRegistry, tree: &'a BnyTree<'b>) -> Self {
        // Initial queue of root level rules + nodes
        let mut work_queue: Vec<ValidationFrame> = Vec::new();
        work_queue.extend(tree.nodes().map(|n| ValidationFrame {
            node: n,
            rule_base: None,
            schema_id: registry.entry,
        }));

        while let Some(frame) = work_queue.pop() {
            let schema = &registry.schemas[frame.schema_id.0];
            let result = if let Some(rule_base) = &frame.rule_base {
                Self::process_rules(rule_base.children(schema), schema, tree, frame)
            } else {
                Self::process_rules(schema.rules(), schema, tree, frame)
            };

            match result {
                // explicit match, descend into this node
                ValidationResult::Matched(rule) => {
                    work_queue.extend(frame.node.children(tree).map(move |n| ValidationFrame {
                        node: n,
                        schema_id: frame.schema_id,
                        rule_base: Some(rule),
                    }));
                }
                // change context for processing children
                ValidationResult::ChangeContext(schema_index) => work_queue.push(ValidationFrame {
                    node: frame.node,
                    schema_id: schema_index,
                    rule_base: None,
                }),
                // no rules matched, don't bother descending
                ValidationResult::Unmatched => {}
            }
        }

        ValidatedDocument {
            _registry: registry,
            _tree: tree,
        }
    }
}

enum ValidationResult {
    // matched a rule
    Matched(BnyRuleRef),
    // unmatched, try someone else
    Unmatched,
    // change schema validation context
    ChangeContext(SchemaIndex),
}
