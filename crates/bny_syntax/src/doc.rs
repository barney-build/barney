// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax document

use std::vec;

use indextree::{Arena, NodeId};
use itertools::Itertools;
use kdl::{KdlDocument, KdlError, KdlNode};
use miette::Diagnostic;
use thiserror::Error;

use crate::{NodeTag, ProcessedValue};

pub struct BnyNode {
    name: String,
    tag: Option<NodeTag>,
    args: Option<NodeId>,
    props: Option<NodeId>,
}

pub struct BnyArgument {
    value: ProcessedValue,
}

pub struct BnyProperty {
    name: String,
    value: ProcessedValue,
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

impl<'a> BnyNode {
    // Generate arena node only, not props/entries
    fn from_kdl_node(
        raw: &'a KdlNode,
        args: Option<NodeId>,
        props: Option<NodeId>,
    ) -> (Self, Vec<Error>) {
        match raw.ty().map(|m| m.try_into().map_err(Error::Tag)) {
            Some(Ok(tag)) => (
                Self {
                    name: raw.name().value().to_owned(),
                    tag: Some(tag),
                    args,
                    props,
                },
                vec![],
            ),
            Some(Err(e)) => (
                Self {
                    name: raw.name().value().to_owned(),
                    tag: None,
                    args,
                    props,
                },
                vec![e],
            ),
            None => (
                Self {
                    name: raw.name().value().to_owned(),
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
pub struct BnyDocument {
    contents: String,
}

/// A BnyResult yields diagnostics from parsing a `.bny` file
pub struct BnyResult {
    arena: Arena<BnyNode>,

    // all arguments
    args: Arena<BnyArgument>,
    props: Arena<BnyProperty>,

    pub diagnostics: Vec<Error>,
}

// Contextual handle to a node
pub struct BnyNodeRef<'a> {
    id: NodeId,
    context: &'a BnyResult,
    prop_id: Option<NodeId>,
    arg_id: Option<NodeId>,
}

impl<'a> BnyNodeRef<'a> {
    /// Retrieve name by ref
    pub fn name(&self) -> &str {
        self.context
            .arena
            .get_data(self.id)
            .map(|n| n.name.as_str())
            .unwrap_or_default()
    }

    /// Retrieve the node tag
    pub fn tag(&self) -> Option<NodeTag> {
        self.context
            .arena
            .get_data(self.id)
            .and_then(|n| n.tag.clone())
    }

    /// Find all properties
    pub fn properties(&self) -> impl Iterator<Item = &'a BnyProperty> {
        self.prop_id
            .into_iter()
            .flat_map(move |id| id.following_siblings(&self.context.props))
            .flat_map(move |id| self.context.props.get_data(id))
    }

    /// Get the arguments
    pub fn arguments(&self) -> impl Iterator<Item = &'a BnyArgument> {
        self.arg_id
            .into_iter()
            .flat_map(move |id| id.following_siblings(&self.context.args))
            .flat_map(move |id| self.context.args.get_data(id))
    }

    // Yield children of this node
    pub fn children(&self) -> impl Iterator<Item = BnyNodeRef<'a>> {
        self.id.children(&self.context.arena).flat_map(move |id| {
            self.context.arena.get_data(id).map(move |n| BnyNodeRef {
                id,
                context: self.context,
                prop_id: n.props,
                arg_id: n.args,
            })
        })
    }
}

impl<'a> BnyResult {
    /// Returns true if the result means we've got legal parsing,
    /// ie diagnostics are non fatal
    pub fn is_valid(&self) -> bool {
        self.diagnostics
            .iter()
            .find(|f| matches!(f.severity(), Some(miette::Severity::Error)))
            .is_none()
    }

    /// Iterate the built nodes
    pub fn nodes(&'a self) -> impl Iterator<Item = BnyNodeRef<'a>> {
        self.arena.roots().flat_map(|id| {
            self.arena.get_data(id).map(|a| BnyNodeRef {
                id,
                context: self,
                prop_id: a.props,
                arg_id: a.args,
            })
        })
    }
}

impl BnyDocument {
    /// Returns a new BnyDocument.
    /// Note: All KDL errors / processing happens here.
    pub fn new(data: &str) -> Self {
        Self {
            contents: data.to_owned(),
        }
    }

    fn process_node(
        &self,
        arena: &mut Arena<BnyNode>,
        args: &mut Arena<BnyArgument>,
        props: &mut Arena<BnyProperty>,
        raw: &KdlNode,
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
                    let prop = BnyProperty {
                        name: name.value().to_owned(),
                        value: entry.value().into(),
                    };
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
                    let arg = BnyArgument {
                        value: entry.value().into(),
                    };
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

        // steal the children now
        let children = raw.iter_children().collect_vec();

        // Append into the parent scope if provided or create a new node
        let parent = if let Some(pid) = parent_id {
            pid.append_value(node, arena)
        } else {
            arena.new_node(node)
        };

        diagnostics.extend(
            children
                .into_iter()
                .flat_map(|n| self.process_node(arena, args, props, n, Some(parent))),
        );

        diagnostics
    }

    /// Return all processed nodes and any diagnostics from our
    /// own pass
    pub fn process(&self) -> BnyResult {
        let mut arena = Arena::new();
        let mut args = Arena::new();
        let mut props = Arena::new();

        let doc = match KdlDocument::parse_v2(&self.contents) {
            Ok(d) => d,
            Err(e) => {
                return BnyResult {
                    arena,
                    args,
                    props,
                    diagnostics: vec![e.into()],
                };
            }
        };

        let diagnostics = doc
            .nodes()
            .iter()
            .flat_map(|n| self.process_node(&mut arena, &mut args, &mut props, n, None))
            .collect_vec();

        BnyResult {
            arena,
            args,
            props,
            diagnostics,
        }
    }
}

impl BnyArgument {
    /// Arguments processed value
    pub fn value(&self) -> &ProcessedValue {
        &self.value
    }
}

impl BnyProperty {
    /// Property name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Property value
    pub fn value(&self) -> &ProcessedValue {
        &self.value
    }
}
#[cfg(test)]
mod tests {
    use crate::BnyDocument;

    #[test]
    fn new_doc_simple() {
        let input = r#"document {
    child_node prop=2 lol {
        list
    }
}
"#;
        let doc = BnyDocument::new(&input);
        let res = doc.process();
        assert_eq!(res.diagnostics.len(), 0);
        assert!(res.is_valid());
    }
}
