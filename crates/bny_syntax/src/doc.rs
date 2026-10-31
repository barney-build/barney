// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax document

use std::vec;

use indextree::{Arena, NodeId};
use itertools::Itertools;
use kdl::{KdlDocument, KdlError, KdlNode};
use miette::{Diagnostic, SourceOffset, SourceSpan};
use thiserror::Error;

use crate::{NodeTag, ProcessedValue};

/// Internal node storage within the BnyTree arena
struct BnyNode {
    name: String,
    tag: Option<NodeTag>,
    args: Option<NodeId>,
    props: Option<NodeId>,
    span: SourceSpan,
}

/// An argument as provided to a node
pub struct BnyArgument {
    value: ProcessedValue,
    span: SourceSpan,
}

/// A property of a node
pub struct BnyProperty {
    name: String,
    value: ProcessedValue,
    name_span: SourceSpan,
    value_span: SourceSpan,
    span: SourceSpan,
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
                    span: raw.span(),
                },
                vec![],
            ),
            Some(Err(e)) => (
                Self {
                    name: raw.name().value().to_owned(),
                    tag: None,
                    args,
                    props,
                    span: raw.span(),
                },
                vec![e],
            ),
            None => (
                Self {
                    name: raw.name().value().to_owned(),
                    tag: None,
                    args,
                    props,
                    span: raw.span(),
                },
                vec![],
            ),
        }
    }
}

/// A BnySpec is used for parsing a `bny` file into
/// a set of nodes. Largely this is identical to a `kdl` file
/// but with the core tags etc locked in
pub struct BnySpec {
    contents: String,
}

/// A BnyTree is our IR format for any `.bny` file, currently fed by KDL
///
/// A Tree structure is generated along with any diagnostics encountered
/// in processing.
pub struct BnyTree<'a> {
    spec: &'a BnySpec,
    arena: Arena<BnyNode>,
    span: SourceSpan,

    // all arguments
    args: Arena<BnyArgument>,
    props: Arena<BnyProperty>,

    diagnostics: Vec<Error>,
}

/// Public contextual view of a node
pub struct BnyNodeRef<'a> {
    id: NodeId,
    tree: &'a BnyTree<'a>,
    prop_id: Option<NodeId>,
    arg_id: Option<NodeId>,
}

impl<'a> BnyNodeRef<'a> {
    /// Retrieve name by ref
    pub fn name(&self) -> &str {
        self.tree
            .arena
            .get_data(self.id)
            .map(|n| n.name.as_str())
            .unwrap_or_default()
    }

    /// Retrieve the node tag
    pub fn tag(&self) -> Option<NodeTag> {
        self.tree
            .arena
            .get_data(self.id)
            .and_then(|n| n.tag.clone())
    }

    /// Find all properties
    pub fn properties(&self) -> impl Iterator<Item = &'a BnyProperty> {
        self.prop_id
            .into_iter()
            .flat_map(move |id| id.following_siblings(&self.tree.props))
            .flat_map(move |id| self.tree.props.get_data(id))
    }

    /// Get the arguments
    pub fn arguments(&self) -> impl Iterator<Item = &'a BnyArgument> {
        self.arg_id
            .into_iter()
            .flat_map(move |id| id.following_siblings(&self.tree.args))
            .flat_map(move |id| self.tree.args.get_data(id))
    }

    // Yield children of this node
    pub fn children(&self) -> impl Iterator<Item = BnyNodeRef<'a>> {
        self.id.children(&self.tree.arena).flat_map(move |id| {
            self.tree.arena.get_data(id).map(move |n| BnyNodeRef {
                id,
                tree: self.tree,
                prop_id: n.props,
                arg_id: n.args,
            })
        })
    }

    /// Returns node span
    pub fn span(&self) -> SourceSpan {
        self.tree.arena[self.id].get().span
    }
}

impl<'a> BnyTree<'a> {
    /// Returns true if the result means we've got legal parsing,
    /// ie diagnostics are non fatal
    pub fn is_valid(&self) -> bool {
        self.diagnostics
            .iter()
            .find(|f| matches!(f.severity(), Some(miette::Severity::Error)))
            .is_none()
    }

    /// Access the diagnostics
    pub fn diagnostics(&self) -> &[Error] {
        self.diagnostics.iter().as_slice()
    }

    /// Complete source span
    pub fn span(&self) -> SourceSpan {
        self.span
    }

    /// Iterate the built nodes
    pub fn nodes(&'a self) -> impl Iterator<Item = BnyNodeRef<'a>> {
        self.arena.roots().flat_map(|id| {
            self.arena.get_data(id).map(|a| BnyNodeRef {
                id,
                tree: self,
                prop_id: a.props,
                arg_id: a.args,
            })
        })
    }

    /// Return underlying spec file
    pub fn spec(&self) -> &BnySpec {
        self.spec
    }
}

impl<'a> BnySpec {
    /// Returns a new BnySpec.
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
                        span: entry.span(),
                        name_span: name.span(),
                        value_span: entry.span(),
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
                        span: entry.span(),
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

        // Append into the parent scope if provided or create a new node
        let parent = if let Some(pid) = parent_id {
            pid.append_value(node, arena)
        } else {
            arena.new_node(node)
        };

        diagnostics.extend(
            raw.iter_children()
                .flat_map(|n| self.process_node(arena, args, props, n, Some(parent))),
        );

        diagnostics
    }

    /// Return all processed nodes and any diagnostics from our
    /// own pass
    pub fn process(&'a self) -> BnyTree<'a> {
        let mut arena = Arena::new();
        let mut args = Arena::new();
        let mut props = Arena::new();

        let doc = match KdlDocument::parse_v2(&self.contents) {
            Ok(d) => d,
            Err(e) => {
                return BnyTree {
                    spec: self,
                    span: SourceSpan::new(SourceOffset::from_location(&self.contents, 0, 0), 0),
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

        BnyTree {
            spec: self,
            span: doc.span(),
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

    pub fn span(&self) -> SourceSpan {
        self.span
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

    /// Return span for the whole property pair
    pub fn span(&self) -> SourceSpan {
        self.span
    }

    /// Return span for the identifier itself
    pub fn name_span(&self) -> SourceSpan {
        self.name_span
    }

    /// Return span for the value itself
    pub fn value_span(&self) -> SourceSpan {
        self.value_span
    }
}
#[cfg(test)]
mod tests {
    use crate::BnySpec;

    #[test]
    fn new_doc_simple() {
        let input = r#"document {
    child_node prop=2 lol {
        list
    }
}
"#;
        let doc = BnySpec::new(&input);
        let res = doc.process();
        assert_eq!(res.diagnostics.len(), 0);
        assert!(res.is_valid());
    }
}
