// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Node schema definition
//!
//! Define a node matching mechanism, along with corresponding
//! match rules.

use std::marker::PhantomData;

use crate::SchemaKey;

/// Node schema
pub struct NodeDef<'a, T: SchemaKey<'a>> {
    _phantom: PhantomData<&'a T>,

    identity: T,
}

impl<'a, T: SchemaKey<'a>> NodeDef<'a, T> {
    /// Return the runtime identity
    pub fn identity(&self) -> T {
        self.identity
    }
}
