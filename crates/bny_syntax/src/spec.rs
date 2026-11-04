// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax document

use std::vec;

use indextree::{Arena, NodeId};
use itertools::Itertools;
use kdl::{KdlDocument, KdlNode};
use miette::{SourceOffset, SourceSpan};

use crate::{BnyArgument, BnyProperty, BnyTree, Error, interner::BnyInterner, tree::BnyNode};

/// A BnySpec is used for parsing a `bny` file into
/// a set of nodes. Largely this is identical to a `kdl` file
/// but with the core tags etc locked in
pub struct BnySpec {
    contents: String,
    intern: BnyInterner,
}

impl<'a> BnySpec {
    /// Returns a new BnySpec.
    /// Note: All KDL errors / processing happens here.
    pub fn new(data: &str) -> Self {
        Self {
            contents: data.to_owned(),
            intern: BnyInterner::default(),
        }
    }

    /// Mutable access for the interner
    ///
    /// This allows layered implementation to share the same
    /// table.
    pub fn interner_mut(&mut self) -> &mut BnyInterner {
        &mut self.intern
    }

    /// Read-only access to the interner
    pub fn interner(&mut self) -> &BnyInterner {
        &self.intern
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
        let node = BnyNode::from_kdl_node(raw, args_id_first, props_id_first);

        // Append into the parent scope if provided or create a new node
        let parent = if let Some(pid) = parent_id {
            pid.append_value(node, arena)
        } else {
            arena.new_node(node)
        };

        raw.iter_children()
            .flat_map(|n| self.process_node(arena, args, props, n, Some(parent)))
            .collect_vec()
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
