// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Processing of arguments for the ValidatedDocument

use bny_diag::DiagnosticSink;
use bny_slots::{Block, SlotStorage};
use bny_syntax::{BnyNodeRef, BnyPropertyRef, BnyTree};

use crate::{
    BnySchema, BnyValueType, NodeSpec, PropertySpec,
    compiled::BnyRuleRef,
    diagnostics::{SchemaDiagnostic, reporting},
};

/// Super basic property matching (will need to match args style soon)
#[derive(Debug, Clone, Copy)]
enum PropertyMatchResult {
    Matched { prop_ref: BnyPropertyRef },
    Unmatched,
}

/// Internal property processing
/// TODO: Mirror args.rs
fn property_processor(
    node: &BnyNodeRef,
    property: &PropertySpec,
    tree: &BnyTree,
    sink: &mut DiagnosticSink,
) -> PropertyMatchResult {
    for disc_prop in node.properties(tree) {
        if let Some(prop_id) = property.id
            && prop_id != disc_prop.name(tree)
        {
            continue;
        }
        let stored_kind = BnyValueType::from_syntax_value(disc_prop.value(tree), property.kind)
            .unwrap_or(BnyValueType::Unknown);
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

/// Validate properties for a given node/rule
pub(super) fn validate(
    storage: &mut SlotStorage<Option<BnyPropertyRef>>,
    tree: &BnyTree,
    schema: &BnySchema,
    rule: BnyRuleRef,
    node_spec: &NodeSpec,
    node: &BnyNodeRef,
    sink: &mut DiagnosticSink,
) -> Block {
    // Reserve property slots
    let n_prop = rule.properties(schema, node_spec).count();
    let p_block = storage.allocate(n_prop, || None);

    // Find all matching property rules
    for prop in rule.properties(schema, node_spec) {
        let res = property_processor(node, prop, tree, sink);
        let stored = match res {
            PropertyMatchResult::Matched { prop_ref } => Some(prop_ref),
            PropertyMatchResult::Unmatched => None,
        };
        storage.set(p_block.0.start + prop.local_index, stored);
    }

    p_block
}
