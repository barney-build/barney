// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax document

use indextree::{Arena, NodeId};
use itertools::Itertools;
use kdl::{KdlDocument, KdlEntry, KdlError, KdlIdentifier, KdlNode};
use miette::Diagnostic;
use thiserror::Error;

use crate::NodeTag;

#[derive(Debug)]
pub struct BnyNode<'a> {
    raw: &'a KdlNode,
    tag: Option<NodeTag>,
    args: Option<NodeId>,
    props: Option<NodeId>,
}

#[derive(Debug)]
pub struct BnyArgument<'a> {
    raw: &'a KdlEntry,
}

#[derive(Debug)]
pub struct BnyProperty<'a> {
    name: &'a KdlIdentifier,
    raw: &'a KdlEntry,
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
    fn from_kdl_node(
        raw: &'a KdlNode,
        args: Option<NodeId>,
        props: Option<NodeId>,
    ) -> (Self, Vec<Error>) {
        match raw.ty().map(|m| m.try_into().map_err(Error::Tag)) {
            Some(Ok(tag)) => (
                Self {
                    raw,
                    tag: Some(tag),
                    args,
                    props,
                },
                vec![],
            ),
            Some(Err(e)) => (
                Self {
                    raw,
                    tag: None,
                    args,
                    props,
                },
                vec![e],
            ),
            None => (
                Self {
                    raw,
                    tag: None,
                    args,
                    props,
                },
                vec![],
            ),
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

    // all arguments
    args: Arena<BnyArgument<'a>>,
    props: Arena<BnyProperty<'a>>,

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
        args: &mut Arena<BnyArgument<'a>>,
        props: &mut Arena<BnyProperty<'a>>,
        raw: &'a KdlNode,
        parent_id: Option<NodeId>,
    ) -> Vec<Error> {
        let mut args_id: Option<NodeId> = None;
        let mut args_id_first: Option<NodeId> = None;

        let mut props_id: Option<NodeId> = None;
        let mut props_id_first: Option<NodeId> = None;

        for entry in raw.entries().iter().filter(|e| !e.is_empty()) {
            match entry.name() {
                // Property
                Some(name) => {
                    let prop = BnyProperty { name, raw: entry };
                    props_id = match props_id {
                        Some(id) => Some(id.insert_after_value(prop, props)),
                        None => {
                            let id = props.new_node(prop);
                            props_id_first = Some(id);
                            props_id_first
                        }
                    }
                }

                // Argument
                None => {
                    let arg = BnyArgument { raw: entry };
                    args_id = match args_id {
                        Some(id) => Some(id.insert_after_value(arg, args)),
                        None => {
                            let id = args.new_node(arg);
                            args_id_first = Some(id);
                            args_id_first
                        }
                    }
                }
            }
        }
        let (node, mut diagnostics) = BnyNode::from_kdl_node(raw, args_id_first, props_id_first);

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
                .flat_map(|n| self.process_node(arena, args, props, n, Some(parent))),
        );

        diagnostics
    }

    /// Return all processed nodes and any diagnostics from our
    /// own pass
    pub fn process<'a>(&'a self) -> BnyResult<'a> {
        let mut arena = Arena::new();
        let mut args = Arena::new();
        let mut props = Arena::new();

        let diagnostics = self
            .doc
            .nodes()
            .iter()
            .flat_map(|n| self.process_node(&mut arena, &mut args, &mut props, n, None))
            .collect_vec();

        BnyResult {
            doc: self,
            arena,
            args,
            props,
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
        eprintln!("arg set: {:?}", res.args);
        assert_eq!(res.diagnostics.len(), 0);
        assert!(res.is_valid());
    }
}
