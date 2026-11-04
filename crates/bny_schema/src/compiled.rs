// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Compiled barney schema

use bny_syntax::BnyInterner;
use indextree::{Arena, NodeId};

use crate::{Rule, SchemaDef, SchemaKey, spec::RuleSpec};

/// A compiled bny schema representation
#[derive(Debug)]
pub struct BnySchema {
    nodes: Arena<RuleSpec>,
}
// Export type of nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BnySchemaId(NodeId);

impl BnySchema {
    pub fn new() -> Self {
        Self {
            nodes: Arena::new(),
        }
    }

    /// Add a rule node into the tree
    /// TODO: NodeSpec needs to include all PropertySpec/ArgSpec
    fn add_rule(&mut self, spec: RuleSpec, parent: Option<BnySchemaId>) -> BnySchemaId {
        let id = if let Some(parent_id) = parent.map(|p| p.0) {
            parent_id.append_value(spec, &mut self.nodes)
        } else {
            self.nodes.new_node(spec)
        };

        eprintln!("{:#?}", id.debug_pretty_print(&self.nodes));
        BnySchemaId(id)
    }

    /// Build a BnySchema from a SchemaDef
    pub fn bind_from_def<'a, T: SchemaKey<'a>>(
        &mut self,
        intern: &mut BnyInterner,
        schema_def: &'a SchemaDef<'a, T>,
    ) {
        let mut processing = Vec::new();
        let batch = schema_def.nodes().iter().rev().map(|n| (n, None));
        processing.extend(batch);
        while let Some((node, parent)) = processing.pop() {
            let spec = node.spec(intern);
            let rule = RuleSpec::MatchNode(spec);
            let id = self.add_rule(rule, parent);

            match node.child() {
                Some(Rule::Nodes(nodes)) => {
                    processing.extend(nodes.iter().rev().map(move |n| (n, Some(id))))
                }
                Some(Rule::External(schema_ref)) => {
                    let sym = intern.intern(schema_ref.0);
                    self.add_rule(RuleSpec::ExternalSchema(sym), Some(id));
                }
                None => {}
            }
        }
    }
}

impl Default for BnySchema {
    fn default() -> Self {
        Self::new()
    }
}
