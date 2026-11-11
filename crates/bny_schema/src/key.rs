// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Key identification and semantic tagging
//!
//! Provide a runtime mechanism for matching a symbolic identifier
//! with the actual string representation in `bny_syntax`

use std::fmt;

/// Key lookup trait for runtime analysis
pub trait SchemaKey<'a>: Copy + fmt::Debug {
    /// Return the symbolic identity (author bound)
    fn symbolic_id(self) -> u8;

    /// Return a name matching mechanism
    fn name(&self) -> SchemaName<'a>;
}

/// Name matching mechanism for node and property rules
pub enum SchemaName<'a> {
    /// A matching rule in the correct context must have
    /// exactly this name
    Explicit(&'a str),

    /// A rule may match regardless of name as long as the
    /// other rule constraints are met
    Dynamic,
}
