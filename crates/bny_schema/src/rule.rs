// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Schema rules

use crate::{NodeDef, SchemaKey, SchemaRef};

/// Either a set of nodes from the current schema
/// *or* an external schema must apply.
pub enum Rule<T: SchemaKey<'a>, 'a> {
    /// Match any of the given node definitions
    Nodes(&'a [&NodeDef<'a, T>]),

    /// External schema satisfies children in this context
    External(SchemaRef<'a>),
}
