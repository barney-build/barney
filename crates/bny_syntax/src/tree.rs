// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax document

use indextree::{Arena, NodeId};
use kdl::KdlNode;
use miette::{Diagnostic, SourceSpan};

use crate::{BnyArgument, BnyInterner, BnyProperty, BnySpec, Error, StringId};

/// Internal node storage within the BnyTree arena
#[derive(Debug)]
pub(super) struct BnyNode {
    pub(super) name: StringId,
    pub(super) tag: Option<StringId>,
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
        intern: &'a mut BnyInterner,
    ) -> Self {
        let name = intern.intern(raw.name().value());
        let tag = raw.ty().map(|i| intern.intern(i.value()));

        Self {
            name,
            tag,
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
#[derive(Debug)]
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
#[derive(Debug, Clone, Copy)]
pub struct BnyNodeRef {
    id: NodeId,
    prop_id: Option<NodeId>,
    arg_id: Option<NodeId>,
}

impl BnyNodeRef {
    /// Retrieve name by ref
    pub fn name(&self, tree: &BnyTree) -> StringId {
        tree.arena[self.id].get().name
    }

    /// Retrieve the node tag
    pub fn tag(&self, tree: &BnyTree) -> Option<StringId> {
        tree.arena[self.id].get().tag
    }

    /// Find all properties
    pub fn properties<'a>(&self, tree: &'a BnyTree<'a>) -> impl Iterator<Item = &'a BnyProperty> {
        self.prop_id
            .into_iter()
            .flat_map(move |id| id.following_siblings(&tree.props))
            .flat_map(move |id| tree.props.get_data(id))
    }

    /// Get the arguments
    pub fn arguments<'a>(&self, tree: &'a BnyTree<'a>) -> impl Iterator<Item = &'a BnyArgument> {
        self.arg_id
            .into_iter()
            .flat_map(move |id| id.following_siblings(&tree.args))
            .flat_map(move |id| tree.args.get_data(id))
    }

    // Yield children of this node
    pub fn children(&self, tree: &BnyTree) -> impl Iterator<Item = BnyNodeRef> {
        self.id.children(&tree.arena).flat_map(move |id| {
            tree.arena.get_data(id).map(move |n| BnyNodeRef {
                id,
                prop_id: n.props,
                arg_id: n.args,
            })
        })
    }

    /// Returns node span
    pub fn span(&self, tree: BnyTree) -> SourceSpan {
        tree.arena[self.id].get().span
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

    /// Iterate the root level nodes
    pub fn nodes(&'a self) -> impl Iterator<Item = BnyNodeRef> {
        self.arena.roots().flat_map(|id| {
            self.arena.get_data(id).map(|a| BnyNodeRef {
                id,
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
    use itertools::Itertools;

    use crate::{BnyInterner, BnySpec};

    #[test]
    fn new_doc_simple() {
        let input = r#"document schema="is it though" {
    child_node prop=2 lol {
        list
    }
}
"#;

        let doc = BnySpec::new(&input);
        let mut it = BnyInterner::default();
        let res = doc.process(&mut it);
        assert_eq!(res.diagnostics.len(), 0);
        assert!(res.is_valid());

        let nodes = res.nodes().collect_vec();
        let root = nodes.get(0).expect("no root of document");
        let name = it.resolve(root.name(&res));
        assert_eq!(name, "document");
        let props = root.properties(&res).collect_vec();
        assert_eq!(props.len(), 1);
        let prop = props.get(0).unwrap();
        let prop_name = it.resolve(prop.name());
        assert_eq!(prop_name, "schema");
        let prop_value = prop.value().as_string().expect("Should contain a string!");
        assert_eq!(prop_value, "is it though");
    }
}
