// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax document

use indextree::{Arena, NodeId};
use kdl::KdlNode;
use miette::{Diagnostic, SourceSpan};

use crate::{BnyArgument, BnyProperty, BnySpec, Error};

/// Internal node storage within the BnyTree arena
pub(super) struct BnyNode {
    pub(super) name: String,
    pub(super) tag: Option<String>,
    pub(super) args: Option<NodeId>,
    pub(super) props: Option<NodeId>,
    pub(super) span: SourceSpan,
}

impl<'a> BnyNode {
    // Generate arena node only, not props/entries
    pub(super) fn from_kdl_node(
        raw: &'a KdlNode,
        args: Option<NodeId>,
        props: Option<NodeId>,
    ) -> Self {
        Self {
            name: raw.name().value().to_owned(),
            tag: raw.ty().map(|i| i.value().to_owned()),
            args,
            props,
            span: raw.span(),
        }
    }
}

/// A BnyTree is our IR format for any `.bny` file, currently fed by KDL
///
/// A Tree structure is generated along with any diagnostics encountered
/// in processing.
pub struct BnyTree<'a> {
    pub(super) spec: &'a BnySpec,
    pub(super) arena: Arena<BnyNode>,
    pub(super) span: SourceSpan,

    // all arguments
    pub(super) args: Arena<BnyArgument>,
    pub(super) props: Arena<BnyProperty>,

    pub(super) diagnostics: Vec<Error>,
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
    pub fn tag(&self) -> Option<&str> {
        self.tree
            .arena
            .get_data(self.id)
            .and_then(|b| b.tag.as_deref())
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
