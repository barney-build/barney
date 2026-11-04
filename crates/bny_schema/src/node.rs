// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Node schema definition
//!
//! Define a node matching mechanism, along with corresponding
//! match rules.

use std::marker::PhantomData;

use crate::{Rule, SchemaKey};

/// Node schema
pub struct NodeDef<'a, T: SchemaKey<'a>> {
    _phantom: PhantomData<&'a T>,

    identity: T,
    children: Option<&'a [&'a Rule<'a, T>]>,
}

impl<'a, T: SchemaKey<'a>> NodeDef<'a, T> {
    /// Return the runtime identity
    pub fn identity(&self) -> T {
        self.identity
    }

    /// Set the children matching rule
    pub const fn children(mut self, children: &'a [&'a Rule<'a, T>]) -> Self {
        self.children = Some(children);
        self
    }
}
