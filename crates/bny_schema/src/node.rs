// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Node schema definition
//!
//! Define a node matching mechanism, along with corresponding
//! match rules.

use crate::{Rule, SchemaKey};

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
    /// Return the runtime identity
    pub fn identity(&self) -> T {
        self.identity
    }

    /// Set the children matching rule
    pub const fn children(mut self, children: Rule<'a, T>) -> Self {
        self.children = Some(children);
        self
    }
}
