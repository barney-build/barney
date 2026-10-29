// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax helpers / errors

use std::{collections::HashMap, fmt::Debug, hash::Hash};

use kdl::{KdlEntry, KdlNode, KdlValue};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

// export spec types
mod spec;
pub use spec::*;

mod process;
pub use process::process_kdl;

mod tag;
pub use tag::{NodeTag, Type};

mod script;
pub use script::{Script, Statement};

mod symbol;
pub use symbol::{Symbol, SymbolError, SymbolTable, SymbolType};

use crate::tag::EntryTag;

/// A "Baked" node when processed via AST
#[derive(Debug)]
pub struct ProcessedNode<'a, I>
where
    I: Into<usize> + Eq + PartialEq + PartialOrd + Hash + Debug + Clone,
{
    pub identity: I,
    pub tag: Option<NodeTag>,

    // TODO: Use type system with tagging + variable references
    pub args: Vec<ProcessedEntry>,
    pub props: HashMap<String, String>,

    pub children: Vec<ProcessedNode<'a, I>>,

    // Underlying KDL node
    pub(crate) node: &'a KdlNode,
}

/// A baked entry, ie anything *after* the node declaration but not within the children.
/// Note a naked entry (non-property) can still be a non-string, in that the parent node
/// itself has a valid string ID already so subsequent arguments/etc may be integers or
/// whichever.
#[derive(Debug)]
pub struct ProcessedEntry {
    /// Optionally specifies if there is a binding: Property or Variable
    pub tag: Option<EntryTag>,
    pub value: ProcessedValue,
    span: SourceSpan,
}

impl ProcessedEntry {
    pub fn span(&self) -> SourceSpan {
        self.span
    }
}

impl<'a> TryFrom<&'a KdlEntry> for ProcessedEntry {
    type Error = crate::Error;

    fn try_from(entry: &'a KdlEntry) -> Result<Self, Self::Error> {
        let tag: Option<EntryTag> = if let Some(ty) = entry.ty() {
            Some(ty.try_into()?)
        } else {
            None
        };
        Ok(Self {
            tag,
            span: entry.span(),
            value: entry.value().into(),
        })
    }
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

impl<'a, I> ProcessedNode<'a, I>
where
    I: Into<usize> + Eq + PartialEq + PartialOrd + Hash + Debug + Clone,
{
    /// Zero-copy name accessor
    pub fn name(&self) -> &str {
        self.node.name().value()
    }

    /// Expose the source span for the node
    pub fn span(&self) -> SourceSpan {
        self.node.span()
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
    TypeAttribute {
        #[label("This node cannot be used in conjunction with type attributes")]
        span: SourceSpan,
        tag: String,
    },
}
