// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Barney syntax node support

use crate::{NodeTag, ProcessedEntry};

use kdl::KdlNode;
use miette::SourceSpan;
use std::{collections::HashMap, fmt::Debug, hash::Hash};

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
