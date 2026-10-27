// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax helpers / errors

use std::{collections::HashMap, fmt::Debug, hash::Hash, vec};

use kdl::KdlNode;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

// export spec types
mod spec;
pub use spec::*;

mod process;
pub use process::process_kdl;

mod tag;
pub use tag::{Tag, TaggedValue};

mod script;
pub use script::{Script, Statement};

mod symbol;
pub use symbol::{Symbol, SymbolError, SymbolTable, SymbolType};

/// A "Baked" node when processed via AST
#[derive(Debug)]
pub struct ProcessedNode<'a, I>
where
    I: Into<usize> + Eq + PartialEq + PartialOrd + Hash + Debug + Clone,
{
    pub identity: I,
    // TODO: Use type system with tagging + variable references
    pub args: Vec<TaggedValue>,
    pub props: HashMap<String, String>,

    pub children: Vec<ProcessedNode<'a, I>>,

    /// Underlying KDL node
    pub node: &'a KdlNode,
}

impl<'a, I> ProcessedNode<'a, I>
where
    I: Into<usize> + Eq + PartialEq + PartialOrd + Hash + Debug + Clone,
{
    // Zero-copy name accessor
    pub fn name(&self) -> &str {
        self.node.name().value()
    }
}

/// Some manner of syntax issue in our DSL atop KDL
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
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
}
