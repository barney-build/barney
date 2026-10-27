// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Specification types for assisted KDL/DSL processing
//!

use std::{fmt::Debug, hash::Hash};

/// Node identifier rules
#[derive(Debug)]
pub enum NodeName {
    /// A fixed identifier is required
    Static(&'static str),

    /// Arbitrary name for the node supported
    Dynamic,
}

/// Schema splitting for dynamic vs fixed behaviours
#[derive(Debug)]
pub enum NodeDescent {
    /// Descend into all child nodes by static schema
    Normal,

    /// Do not descend into node tree, processing is at block level only
    Never,
}

/// Control evaluation of nodes to enforce schema
#[derive(Debug)]
pub struct NodeSpec<'a, I>
where
    I: Into<usize> + Eq + PartialEq + PartialOrd + Hash + Debug + Clone,
{
    /// The matching name for the node, ie `NodeName::Static("variables")`
    pub name: NodeName,

    /// Identity for the node to retain context in processing
    pub identity: I,

    /// Supported argument kinds
    pub args: ArgSpec,

    /// When not empty, limit the allowed properties
    pub props: &'a [PropSpec],

    /// Children of the node, if permitted
    pub children: &'a [NodeSpec<'a, I>],

    /// Descent behaviour
    pub descent: NodeDescent,
}

/// Argument spec for blocks
///
/// Determines the policy for processing arguments to a node,
/// for an ID based node this would be `ArgSpec::Exactly(1)`
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArgSpec {
    /// Does not support any arguments
    None,

    /// Exact number of arguments
    Exactly(usize),

    /// Any number of arguments.
    Variable,
}

/// Controls property evaluation (which are always built into maps)
/// TODO: Add type filtering here for the value kind
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PropSpec {
    /// The expected name of the property
    pub name: &'static str,
}
