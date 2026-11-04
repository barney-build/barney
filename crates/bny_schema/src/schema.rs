// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Schema definiton
//!
//! Core type for defining schema types and IDs.
//! These definitions are not the runtime validation
//! rules themselves, as thats part of the validation
//! building and schema *compilation*.

use crate::{NodeDef, SchemaKey};

/// Strong type for unique schema IDs / referencing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaRef<'a>(pub &'a str);

/// A schema definition
#[derive(Debug)]
pub struct SchemaDef<'a, T: SchemaKey<'a>> {
    // identity
    id: SchemaRef<'a>,

    /// Our root level nodes
    children: Option<&'a [&'a NodeDef<'a, T>]>,
}

impl<'a, T: SchemaKey<'a>> SchemaDef<'a, T> {
    /// Construct a new schema definition
    pub const fn new(id: SchemaRef<'a>) -> Self {
        Self { id, children: None }
    }

    /// Access underlying ID
    pub const fn id(&self) -> SchemaRef<'a> {
        self.id
    }

    /// Set the initial child nodes
    pub const fn children(mut self, children: &'a [&'a NodeDef<'a, T>]) -> Self {
        self.children = Some(children);
        self
    }

    pub fn nodes(&self) -> &'a [&'a NodeDef<'a, T>] {
        self.children.unwrap_or_default()
    }
}
