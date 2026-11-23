// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Diagnostics used by bny_schema

use thiserror::Error;

pub(crate) mod reporting {
    pub const FIRST_SEEN: &str = "First seen here";
    pub const PROPERTY: &str = "For this property";
    pub const ARGUMENT: &str = "For this argument";
    pub const IN_HERE: &str = "In this node";
    pub const ILLEGAL_NAME: &str = "This symbol is not recognised in this context";
}

#[derive(Debug, Error)]
pub(crate) enum SchemaDiagnostic {
    #[error("Symbol {id:?} is not allowed here")]
    IllegalNode { id: String },

    #[error("expected property to have value {schema}, instead it has {found}")]
    WrongValueType { schema: String, found: String },
}
