// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Basic "script" functionality
//!
//! unlike traditional shell scripts we want verifiable
//! call chains, custody, hashing, etc, without arbitrary
//! crap polluting the planet.

use std::{collections::HashMap, fmt::Debug, hash::Hash};

use kdl::KdlNode;

use crate::{
    ArgSpec, NodeDescent, NodeName, NodeSpec, NodeTagPolicy, ProcessedNode, TaggedValue,
    process::process_kdl_node,
};

/// A Script is simply a list of processed execution statements
/// that are in turn flow-guided by incredibly simple boolean
/// conditions (`with`) or list-based iterations (`foreach`)
/// Anything that is NOT a `with` or `foreach` is considered an
/// execution statement, ie invokes a functor
#[derive(Debug)]
pub enum Statement {
    /// A boolean test
    WithCondition {
        reference: String,
        statements: Vec<Statement>,
    },

    ForEachLoop {
        /// Name of the local argument
        arg: String,

        /// Name of the list to loop
        target: String,

        /// Child statements
        statements: Vec<Statement>,
    },

    Execute {
        /// Functor ID
        keyword: String,

        /// A set of arguments
        arguments: Vec<TaggedValue>,

        /// A set of properties
        /// TODO: Used TaggedValue
        properties: HashMap<String, String>,
    },
}

/// A script contains one or more statements,
/// any comment AST processing happened in fancy
/// DSL atop KDL land.
#[derive(Debug)]
pub struct Script {
    statements: Vec<Statement>,
}

/// Parsing flow for statements
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum StatementID {
    /// Dynamic, we may be an execute or explicit step
    Root,
    WithCondition,
    ForEachLoop,
    Execute,
}

impl From<StatementID> for usize {
    fn from(val: StatementID) -> Self {
        val as usize
    }
}

// compiled rules to recurse statements and process them
// accordingly.
static RULES: NodeSpec<'static, StatementID> = NodeSpec {
    name: NodeName::Dynamic,
    identity: StatementID::Root,
    tags: NodeTagPolicy::None,
    args: ArgSpec::Variable, // ignored
    props: &[],              // ignored
    children: &[
        // a with condition
        NodeSpec {
            name: NodeName::Static("with"),
            identity: StatementID::WithCondition,
            tags: NodeTagPolicy::None,
            // one argument only
            args: ArgSpec::Exactly(1),
            props: &[],    // ignored
            children: &[], // ignored
            descent: NodeDescent::Never,
        },
        // a foreach
        NodeSpec {
            name: NodeName::Static("foreach"),
            identity: StatementID::ForEachLoop,
            tags: NodeTagPolicy::None,
            args: ArgSpec::Exactly(2),
            props: &[],    // ignored
            children: &[], // ignored
            descent: NodeDescent::Never,
        },
        // an executor
        NodeSpec {
            name: NodeName::Dynamic,
            identity: StatementID::Execute,
            tags: NodeTagPolicy::None,
            args: ArgSpec::Variable,
            props: &[],    // dynamic
            children: &[], // not allowed
            descent: NodeDescent::Normal,
        },
    ],
    descent: NodeDescent::Normal,
};

impl<'a> Statement {
    /// Generate a Statement from a given node
    fn from_kdl_node(node: &'a KdlNode) -> Result<Vec<Statement>, super::Error> {
        let mut rules = vec![&RULES];
        let node = process_kdl_node(node, &mut rules)?;
        assert_eq!(node.identity, StatementID::Root);
        let mut statements = vec![];
        for child in node.children {
            match child.identity {
                StatementID::Root => panic!(),
                // Processed with condition
                StatementID::WithCondition => {
                    // The root level is skipped and we dont do automatic recursion
                    let processed = Statement::from_kdl_node(child.node)?;
                    let reference = child.args.first().unwrap().to_string();
                    statements.push(Statement::WithCondition {
                        reference,
                        statements: processed,
                    })
                }
                StatementID::ForEachLoop => {
                    // Likewise, root level is skipped, not automatic recursion
                    let processed = Statement::from_kdl_node(child.node)?;
                    // TODO: Improve like a LOT
                    let arg = child.args.first().unwrap().to_string();
                    let target = child.args.get(1).unwrap().to_string();
                    statements.push(Statement::ForEachLoop {
                        arg,
                        target,
                        statements: processed,
                    })
                }
                // TODO: Disallow child nodes for execute EXPLICITLY
                StatementID::Execute => statements.push(Statement::Execute {
                    keyword: child.name().to_string(),
                    arguments: child.args,
                    properties: child.props,
                }),
            }
        }
        Ok(statements)
    }
}

impl<'a> Script {
    pub fn from_node<I>(node: &'a ProcessedNode<'a, I>) -> Result<Self, super::Error>
    where
        I: Into<usize> + Eq + PartialEq + PartialOrd + Hash + Debug + Clone,
    {
        Self::from_kdl_node(node.node)
    }

    fn from_kdl_node(node: &'a KdlNode) -> Result<Self, super::Error> {
        let statements = Statement::from_kdl_node(node)?;
        Ok(Self { statements })
    }

    /// Returns all of the statements
    pub fn statements(&self) -> &[Statement] {
        self.statements.as_slice()
    }
}
