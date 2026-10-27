// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Bootstrap configuration `module` actions

use itertools::Itertools;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::bootstrap::SpecIdentity;

use syntax::{self, ProcessedNode, Script, TaggedValue};

/// An Action as defined in the modules exported by various
/// KDL files.
/// These actions provide a modern alternative to macro-rich
/// string soup approaches of the past, providing a dynamic
/// lookup table of functors with Rust-style namespace resolution.
///
/// Unlike other action approaches this all boils down to shell script
/// emission with chain of custody and hashing as the central design
/// tenants.
#[derive(Debug)]
pub struct Action {
    pub(super) id: String,
    // args:
    // execute:
    _execute: Option<Script>,
}

/// Action specific error handling
#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error("processing arguments")]
    #[diagnostic()]
    OnePropertiesBlock {
        #[label("only one properties block supported")]
        span: SourceSpan,
    },

    #[error("processing execute")]
    #[diagnostic()]
    OneExecuteBlock {
        #[label("only one execute block supported")]
        span: SourceSpan,
    },

    #[error(transparent)]
    #[diagnostic(transparent)]
    Syntax(#[from] syntax::Error),
}

impl<'a> Action {
    /// Produce a new Action from the given processed node
    pub(super) fn new(node: &'a ProcessedNode<SpecIdentity>) -> Result<Self, super::Error> {
        assert_eq!(node.identity, SpecIdentity::ModuleAction);
        // Grab the action ID
        let action_id = node
            .args
            .first()
            .ok_or_else(|| super::Error::MissingArgument {
                span: node.node.span(),
            })?;
        // absolutely cant have errors
        assert!(matches!(action_id, TaggedValue::Content(_)));
        let id = action_id.to_string();

        // grab all Args
        let props = node
            .children
            .iter()
            .filter(|n| n.identity == SpecIdentity::ModuleActionProperties)
            .collect_vec();

        // grab arguments set and enforce 1 occurance
        if !props.is_empty() {
            let _props_root =
                props
                    .iter()
                    .exactly_one()
                    .map_err(|_| Error::OnePropertiesBlock {
                        span: node.node.span(),
                    })?;
        }

        let execs = node
            .children
            .iter()
            .filter(|n| n.identity == SpecIdentity::ModuleActionExecute)
            .collect_vec();

        let execute = if !execs.is_empty() {
            let exec_root = execs
                .iter()
                .exactly_one()
                .map_err(|_| Error::OneExecuteBlock {
                    span: node.node.span(),
                })?;
            Some(Script::from_kdl_node(exec_root.node).map_err(Error::Syntax)?)
        } else {
            None
        };

        Ok(Self {
            id,
            _execute: execute,
        })
    }

    /// Return ID reference
    pub fn id(&self) -> &str {
        &self.id
    }
}
