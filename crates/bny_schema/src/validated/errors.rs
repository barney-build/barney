// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Schema validation errors

use thiserror::Error;

use crate::{BnyValueType, ValueError};

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("cannot find key {key} in the active schema")]
    InvalidBinding { key: String },

    #[error("using key {key} in the wrong context ")]
    InvalidContext { key: String },

    #[error("symbolic {key} does not point to a node rule")]
    ExpectedNodeSymbolic { key: String },

    #[error("symbolic {key} does not point to a property rule")]
    ExpectedPropertySymbolic { key: String },

    #[error("symbolic {key} does not point to an argument rule")]
    ExpectedArgumentSymbolic { key: String },

    #[error(transparent)]
    ValueError(#[from] ValueError),

    #[error("wrong return type, schema expects {schema_type}, not {found_type}")]
    WrongReturnType {
        schema_type: BnyValueType,
        found_type: BnyValueType,
    },

    #[error("wrong cardinality applied")]
    WrongCardinality,

    #[error("properties not supported for this node")]
    NoProperties,

    #[error("arguments not supported for this node")]
    NoArguments,
}
