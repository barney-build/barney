// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Processing of arguments for the ValidatedDocument

use bny_diag::DiagnosticSink;
use bny_slots::{Block, LinkedSlotStorage};
use bny_syntax::{BnyArgumentRef, BnyNodeRef, BnyTree};

use crate::{
    ArgumentSpec, BnySchema, BnyValueType, NodeSpec, argument::ArgumentKind, compiled::BnyRuleRef,
    diagnostics::reporting,
};

/// Control flow matching of arguments
pub(super) enum ArgumentMatchResult<'a> {
    /// Explicitly matched a rule
    Matched {
        /// Matching argument rule
        arg_ref: BnyArgumentRef,
        /// The next argument rule to use (or reuse)
        next_arg: Option<&'a ArgumentSpec>,
    },
    /// Couldn't find a match as yet
    Unmatched,
}

/// Argument processing for a while let loop
fn argument_processor<'a>(
    rule: &'a ArgumentSpec,
    arg_rules: &mut impl Iterator<Item = &'a ArgumentSpec>,
    arguments: &mut impl Iterator<Item = BnyArgumentRef>,
) -> ArgumentMatchResult<'a> {
    match rule.arg_kind {
        ArgumentKind::Positional => match arguments.next() {
            Some(arg_ref) => ArgumentMatchResult::Matched {
                arg_ref,
                next_arg: arg_rules.next(),
            },
            None => ArgumentMatchResult::Unmatched,
        },
        ArgumentKind::Variadic => match arguments.next() {
            Some(arg_ref) => ArgumentMatchResult::Matched {
                arg_ref,
                next_arg: Some(rule),
            },
            None => ArgumentMatchResult::Unmatched,
        },
    }
}

/// Validate arguments for a given node
pub(super) fn validate(
    storage: &mut LinkedSlotStorage<BnyArgumentRef>,
    tree: &BnyTree,
    schema: &BnySchema,
    rule: BnyRuleRef,
    node_spec: &NodeSpec,
    node: &BnyNodeRef,
    sink: &mut DiagnosticSink,
) -> Block {
    // Reserve arguments..
    let n_arg = rule.arguments(schema, node_spec).count();
    let a_block = storage.allocate(n_arg);

    let mut arguments = node.arguments(tree);
    let mut arg_rules = &mut rule.arguments(schema, node_spec);
    let mut arg_first = arg_rules.next();

    // skip iteration of rules for empty storage - all rules invalidated basically
    if a_block.is_empty() {
        arg_first = None;
    }

    while let Some(arg_rule) = arg_first {
        match argument_processor(arg_rule, &mut arg_rules, &mut arguments) {
            // Got a match, linked list semantics for repeats of the same rule (variadic)
            ArgumentMatchResult::Matched { arg_ref, next_arg } => {
                let encountered_type =
                    BnyValueType::from_syntax_value(arg_ref.value(tree), arg_rule.kind);
                match encountered_type {
                    // Valid, push
                    Ok(_) => storage.push(a_block, arg_rule.local_index, arg_ref),
                    // Invalid, drop
                    Err(e) => {
                        let id = sink.insert_error(e, None);
                        sink.push_span(id, arg_ref.span(tree), reporting::ARGUMENT);
                    }
                }
                arg_first = next_arg
            }
            // this slot is bork
            ArgumentMatchResult::Unmatched => {
                arg_first = arg_rules.next();
            }
        }
    }

    a_block
}
