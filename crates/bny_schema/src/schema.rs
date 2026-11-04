// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Schema definiton
//!
//! Core type for defining schema types and IDs.
//! These definitions are not the runtime validation
//! rules themselves, as thats part of the validation
//! building and schema *compilation*.

use std::marker::PhantomData;

use crate::SchemaKey;

/// Strong type for unique schema IDs / referencing
#[derive(Debug, Clone, Copy)]
pub struct SchemaRef<'a>(pub &'a str);

/// A schema definition
pub struct SchemaDef<'a, T: SchemaKey<'a>> {
    _phantom: PhantomData<&'a T>,

    // identity
    id: SchemaRef<'a>,
}

impl<'a, T: SchemaKey<'a>> SchemaDef<'a, T> {
    /// Construct a new schema definition
    pub const fn new(id: SchemaRef<'a>) -> Self {
        Self {
            _phantom: PhantomData,
            id,
        }
    }

    /// Access underlying ID
    pub const fn id(&self) -> SchemaRef<'a> {
        self.id
    }
}
