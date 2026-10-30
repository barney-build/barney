// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax document

use indextree::{Arena, NodeId};
use itertools::Itertools;
use kdl::{KdlDocument, KdlError, KdlNode};
use miette::Diagnostic;
use thiserror::Error;

use crate::NodeTag;

#[derive(Debug)]
pub struct BnyNode<'a> {
    raw: &'a KdlNode,
    tag: Option<NodeTag>,
}

/// Diagnostic errors from various sources
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic()]
    Kdl(#[from] KdlError),

    // TODO: Tag specific error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Tag(#[from] crate::Error),
}

impl<'a> BnyNode<'a> {
    // Generate arena node only, not props/entries
    fn from_kdl_node(raw: &'a KdlNode) -> (Self, Vec<Error>) {
        match raw.ty().map(|m| m.try_into().map_err(Error::Tag)) {
            Some(Ok(tag)) => (
                Self {
                    raw,
                    tag: Some(tag),
                },
                vec![],
            ),
            Some(Err(e)) => (Self { raw, tag: None }, vec![e]),
            None => (Self { raw, tag: None }, vec![]),
        }
    }
}

/// A BnyDocument is used for parsing a `bny` file into
/// a set of nodes. Largely this is identical to a `kdl` file
/// but with the core tags etc locked in
#[derive(Debug)]
pub struct BnyDocument {
    doc: KdlDocument,
}

/// A BnyResult yields diagnostics from parsing a `.bny` file
pub struct BnyResult<'a> {
    doc: &'a BnyDocument,
    arena: Arena<BnyNode<'a>>,
    pub diagnostics: Vec<Error>,
}

impl<'a> BnyResult<'a> {
    /// Returns true if the result means we've got legal parsing,
    /// ie diagnostics are non fatal
    pub fn is_valid(&self) -> bool {
        self.diagnostics
            .iter()
            .find(|f| matches!(f.severity(), Some(miette::Severity::Error)))
            .is_none()
    }
}

impl BnyDocument {
    /// Returns a new BnyDocument.
    /// Note: All KDL errors / processing happens here.
    pub fn new(data: &str) -> Result<Self, Error> {
        let doc = KdlDocument::parse_v2(data)?;
        Ok(Self { doc })
    }

    fn process_node<'a>(
        &self,
        arena: &mut Arena<BnyNode<'a>>,
        raw: &'a KdlNode,
        parent_id: Option<NodeId>,
    ) -> Vec<Error> {
        let (node, mut diagnostics) = BnyNode::from_kdl_node(raw);

        // Append into the parent scope if provided or create a new node
        let parent = if let Some(pid) = parent_id {
            pid.append_value(node, arena)
        } else {
            arena.new_node(node)
        };

        // Prohibit further processing
        if diagnostics
            .iter()
            .find(|f| matches!(f.severity(), Some(miette::Severity::Error)))
            .is_some()
        {
            return diagnostics;
        }

        let parent_ref = arena[parent].get();
        diagnostics.extend(
            parent_ref
                .raw
                .iter_children()
                .flat_map(|n| self.process_node(arena, n, Some(parent))),
        );

        diagnostics
    }

    /// Return all processed nodes and any diagnostics from our
    /// own pass
    pub fn process<'a>(&'a self) -> BnyResult<'a> {
        let mut arena = Arena::new();
        let diagnostics = self
            .doc
            .nodes()
            .iter()
            .flat_map(|n| self.process_node(&mut arena, n, None))
            .collect_vec();

        BnyResult {
            doc: self,
            arena,
            diagnostics,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::BnyDocument;

    #[test]
    fn new_doc_simple() {
        let input = r#"document {
    child_node {
        list
    }
}
"#;
        let doc = BnyDocument::new(&input).expect("unable to load doc");
        let res = doc.process();
        assert_eq!(res.diagnostics.len(), 0);
        assert!(res.is_valid());
    }
}
