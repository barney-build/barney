// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Diagnostics used by bny_schema

use thiserror::Error;

pub(crate) mod reporting {
    pub const FIRST_SEEN: &str = "First seen here";
    pub const ALSO_SEEN: &str = "and also here";
}

#[derive(Debug, Error)]
pub(crate) enum SchemaDiagnostic {
    #[error("Symbol `{id}` is not allowed here")]
    IllegalNode { id: String },
}
