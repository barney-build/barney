// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Bootstrap configuration `module`

use std::collections::HashMap;

use itertools::Itertools;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::bootstrap::SpecIdentity;

use bny_syntax::{
    ArgSpec, NodeDescent, NodeName, NodeSpec, NodeTag, NodeTagPolicy, ProcessedNode, Symbol,
    SymbolError, SymbolTable, SymbolType, TaggedValue, Type,
};

mod action;
pub use action::Action;

/// A module within the bootstrap configuration
///
/// It can contain its own variables, public exports,
/// and also actions.
#[derive(Debug)]
pub struct Module {
    id: String,
    // TODO: Disallow any tags but var/arg
    vars: HashMap<String, TaggedValue>,
    exports: HashMap<String, TaggedValue>,
    // TODO: Consider sets
    actions: HashMap<String, Action>,
}

/// A `module` specific error
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    // Placeholder while we kick error handling up the bum
    #[error("fatal: missing argument")]
    #[diagnostic()]
    MissingArgument {
        #[label("fatal: missing argument")]
        span: SourceSpan,
    },

    #[error("action error")]
    #[diagnostic(transparent)]
    Action(#[from] action::Error),

    #[error("fatal: unsupported node")]
    #[diagnostic()]
    UnsupportedNode {
        #[label("unsupported node: {id:?}")]
        span: SourceSpan,

        id: SpecIdentity,
    },

    #[error("symbol processing")]
    #[diagnostic(help = "Defined symbols must not contain the `::` characters")]
    Symbol {
        #[label("illegal symbol name")]
        span: SourceSpan,
        #[source]
        source: SymbolError,
    },
}

/// Rules for the module nodespec
pub(super) static RULES: NodeSpec<'static, SpecIdentity> = NodeSpec {
    name: NodeName::Static("module"),
    identity: SpecIdentity::Module,
    tags: NodeTagPolicy::None,
    args: ArgSpec::Exactly(1),
    props: &[],
    children: &[
        // variables
        NodeSpec {
            name: NodeName::Static("variables"),
            identity: SpecIdentity::ModuleVariables,
            tags: NodeTagPolicy::None,
            args: ArgSpec::None,
            props: &[],
            children: &[NodeSpec {
                name: NodeName::Dynamic,
                identity: SpecIdentity::ModuleVariable,
                tags: NodeTagPolicy::None,
                args: ArgSpec::Exactly(1),
                props: &[],
                children: &[],
                descent: NodeDescent::Normal,
            }],
            descent: NodeDescent::Normal,
        },
        // exports
        NodeSpec {
            name: NodeName::Static("exports"),
            identity: SpecIdentity::ModuleExports,
            tags: NodeTagPolicy::None,
            args: ArgSpec::None,
            props: &[],
            children: &[NodeSpec {
                name: NodeName::Dynamic,
                identity: SpecIdentity::ModuleExport,
                tags: NodeTagPolicy::None,
                args: ArgSpec::Exactly(1),
                props: &[],
                children: &[],
                descent: NodeDescent::Normal,
            }],
            descent: NodeDescent::Normal,
        },
        NodeSpec {
            name: NodeName::Static("action"),
            identity: SpecIdentity::ModuleAction,
            tags: NodeTagPolicy::None,
            args: ArgSpec::Exactly(1),
            props: &[],
            children: &[
                NodeSpec {
                    name: NodeName::Static("properties"),
                    identity: SpecIdentity::ModuleActionProperties,
                    args: ArgSpec::None,
                    tags: NodeTagPolicy::None,
                    props: &[],
                    children: &[NodeSpec {
                        name: NodeName::Dynamic,
                        identity: SpecIdentity::ModuleActionProperty,
                        tags: NodeTagPolicy::Required(&[
                            NodeTag::TypeRestriction {
                                restrict: Type::List,
                            },
                            NodeTag::TypeRestriction {
                                restrict: Type::String,
                            },
                        ]),
                        args: ArgSpec::Variable,
                        props: &[],
                        children: &[],
                        descent: NodeDescent::Normal,
                    }],
                    descent: NodeDescent::Normal,
                },
                NodeSpec {
                    name: NodeName::Static("execute"),
                    identity: SpecIdentity::ModuleActionExecute,
                    args: ArgSpec::None,
                    tags: NodeTagPolicy::None,
                    props: &[],
                    children: &[],
                    descent: NodeDescent::Never,
                },
            ],
            descent: NodeDescent::Normal,
        },
    ],
    descent: NodeDescent::Normal,
};

impl Module {
    pub(crate) fn new(
        node: ProcessedNode<SpecIdentity>,
        symbols: &mut SymbolTable,
    ) -> Result<Self, Error> {
        // Pull the ID out
        let id = node
            .args
            .first()
            .ok_or_else(|| Error::MissingArgument { span: node.span() })?;
        let mut exports = HashMap::new();
        let mut vars = HashMap::new();
        let mut actions = HashMap::new();
        let namespace = id.to_string();

        for child in node.children {
            match child.identity {
                SpecIdentity::ModuleVariables => child
                    .children
                    .into_iter()
                    .filter(|c| c.identity == SpecIdentity::ModuleVariable)
                    .map(|n| (n.name().to_owned(), n.args.into_iter().next().unwrap()))
                    .for_each(|(k, v)| {
                        vars.insert(k, v);
                    }),
                SpecIdentity::ModuleExports => child
                    .children
                    .into_iter()
                    .filter(|c| c.identity == SpecIdentity::ModuleExport)
                    .map(|n| (n.name().to_owned(), n.args.into_iter().next().unwrap()))
                    .for_each(|(k, v)| {
                        exports.insert(k, v);
                    }),
                SpecIdentity::ModuleAction => {
                    let action = Action::new(&child)?;
                    symbols
                        .insert(Symbol::new(&namespace, action.id(), SymbolType::Function))
                        .map_err(|e| Error::Symbol {
                            span: child.span(),
                            source: e,
                        })?;
                    actions.insert(action.id.clone(), action);
                }
                _ => {
                    return Err(Error::UnsupportedNode {
                        span: child.span(),
                        id: child.identity,
                    });
                }
            }
        }

        Ok(Self {
            id: namespace,
            exports,
            vars,
            actions,
        })
    }

    /// Return the module ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return all export keys
    pub fn exports(&self) -> Vec<String> {
        self.exports.keys().cloned().collect_vec()
    }

    /// Return all variable keys
    pub fn vars(&self) -> Vec<String> {
        self.vars.keys().cloned().collect_vec()
    }

    /// Return all action IDs
    pub fn actions(&self) -> Vec<String> {
        self.actions.keys().cloned().collect_vec()
    }
}
