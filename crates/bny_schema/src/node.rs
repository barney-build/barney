// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Node schema definition
//!
//! Define a node matching mechanism, along with corresponding
//! match rules.

use bny_syntax::BnyInterner;

use crate::{NodeSpec, Rule, SchemaKey, SchemaName};

/// Node schema
#[derive(Debug)]
pub struct NodeDef<'a, T: SchemaKey<'a>> {
    identity: T,
    children: Option<Rule<'a, T>>,
}

/// Construct a new node definition
pub const fn node_def<'a, T: SchemaKey<'a>>(identity: T) -> NodeDef<'a, T> {
    NodeDef {
        identity,
        children: None,
    }
}

impl<'a, T: SchemaKey<'a>> NodeDef<'a, T> {
    /// Erase type into NodeSpec for compilation
    pub(crate) fn spec(&self, intern: &mut BnyInterner) -> NodeSpec {
        NodeSpec {
            id: match self.identity().name() {
                SchemaName::Explicit(name) => Some(intern.intern(name)),
                SchemaName::Dynamic => None,
            },
            symbolic: Some(self.identity().symbolic_id()),
        }
    }
    /// Return the runtime identity
    pub fn identity(&self) -> T {
        self.identity
    }

    /// Set the children matching rule
    pub const fn children(mut self, children: Rule<'a, T>) -> Self {
        self.children = Some(children);
        self
    }

    /// Return the child rule as a reference
    pub(crate) fn child(&'a self) -> Option<&'a Rule<'a, T>> {
        self.children.as_ref()
    }
}
