// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Node schema definition
//!
//! Define a node matching mechanism, along with corresponding
//! match rules.

use bny_slots::Block;
use bny_syntax::BnyInterner;

use crate::{NodeSpec, Occurrence, PropertyDef, Rule, SchemaKey, SchemaName};

/// Node schema
#[derive(Debug)]
pub struct NodeDef<'a, T: SchemaKey<'a>> {
    identity: T,
    children: Option<Rule<'a, T>>,
    properties: Option<&'a [PropertyDef<'a, T>]>,
    occurrence: Occurrence,
}

impl<'a, T: SchemaKey<'a>> NodeDef<'a, T> {
    /// Construct a new node definition
    pub const fn new(identity: T) -> Self {
        Self {
            identity,
            children: None,
            properties: None,
            occurrence: Occurrence::unbounded(),
        }
    }

    /// Erase type into NodeSpec for compilation
    pub(crate) fn spec(
        &self,
        intern: &mut BnyInterner,
        index: usize,
        properties: Option<Block>,
    ) -> NodeSpec {
        NodeSpec {
            id: match self.identity().name() {
                SchemaName::Explicit(name) => Some(intern.intern(name)),
                SchemaName::Dynamic => None,
            },
            symbolic: self.identity().symbolic_id(),
            local_index: index,
            properties,
            global_index: None,
            occurrence: self.occurrence,
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

    pub const fn properties(mut self, properties: &'a [PropertyDef<'a, T>]) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Return properties rule as a reference
    pub(crate) fn props(&'a self) -> Option<&'a &'a [PropertyDef<'a, T>]> {
        self.properties.as_ref()
    }

    /// Control the occurrence rules for this node
    pub const fn occurrence(mut self, occurrence: Occurrence) -> Self {
        self.occurrence = occurrence;
        self
    }
}
