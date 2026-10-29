// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax helpers / errors

use std::fmt::Debug;

use kdl::KdlValue;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

// export spec types
mod spec;
pub use spec::*;

mod entry;
pub use entry::*;

mod node;
pub use node::*;

mod process;
pub use process::process_kdl;

pub use crate::node::NodeTag;

mod script;
pub use script::{Script, Statement};

mod symbol;
pub use symbol::{Symbol, SymbolError, SymbolTable, SymbolType};

/// Explicitly set type tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Accepts string data only
    String,

    /// Accepts (merges) list of string data only
    List,
}

/// A processed value within an entry, either a positional argument or a property pair's value
/// Note: Integer support directly maps to kdl.dev so we can later add type annotations for smaller
/// shortage if needed.
#[derive(Debug)]
pub enum ProcessedValue {
    Boolean(bool),
    Float(f64),
    String(String),
    Integer(i128),
    Null,
}

impl ProcessedValue {
    /// Same as KdlValue, valid string or nay
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ProcessedValue::String(s) => Some(s),
            _ => None,
        }
    }
}

impl<'a> From<&'a KdlValue> for ProcessedValue {
    fn from(value: &'a KdlValue) -> Self {
        match value {
            KdlValue::String(s) => Self::String(s.clone()),
            KdlValue::Integer(i) => Self::Integer(*i),
            KdlValue::Float(f) => Self::Float(*f),
            KdlValue::Bool(b) => Self::Boolean(*b),
            KdlValue::Null => Self::Null,
        }
    }
}

/// Some manner of syntax issue in our DSL atop KDL
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    /// ie some argument seek or otherwise went boom when we expect it to always work
    #[error("internal error")]
    InternalError {
        #[label("an internal error occurred. please report it")]
        span: SourceSpan,
    },

    #[error("invalid argument")]
    StringIdOnly {
        #[label("the argument must be a valid string")]
        span: SourceSpan,
    },

    /// Encountered an unexpected property, should be an ID
    #[error("unexpected property")]
    UnexpectedProperty {
        #[label("This should be an identifier, not a property")]
        span: SourceSpan,
    },

    #[error("unknown property")]
    UnknownProperty {
        #[label("No such property '{name}' exists")]
        span: SourceSpan,

        name: String,
    },

    #[error("node requires an ID")]
    ExpectedID {
        #[label("A valid string identifier needs to be provided")]
        span: SourceSpan,
    },

    #[error("unexpected argument count")]
    WrongArgumentCount {
        #[label("Wrong number of arguments, expected {expected} but got {found}")]
        span: SourceSpan,

        expected: usize,
        found: usize,
    },

    #[error("unexpected identifier")]
    UnexpectedIdentifier {
        #[label("Encountered an unexpected identifier: {id}")]
        span: SourceSpan,

        id: String,
    },

    #[error("unknown tag")]
    InvalidTag {
        #[label("Encountered an invalid annotation tag: {tag}")]
        span: SourceSpan,
        tag: String,
    },

    #[error("type attribute not supported here: {tag}")]
    IllegalTypeAttribute {
        #[label("This node cannot be used in conjunction with type attributes")]
        span: SourceSpan,
        tag: String,
    },

    #[error("type attribute not supported here: {tag}")]
    WrongTypeAttribute {
        #[label("This tag is unsupported for this node")]
        span: SourceSpan,
        tag: String,
    },

    #[error("type attribute missing")]
    MissingTypeAttribute {
        #[label("This node requires a type attribute")]
        span: SourceSpan,
    },
}
