// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Compiled barney schema

use bny_syntax::{BnyInterner, StringId};
use indextree::{Arena, NodeId};

use crate::{Rule, SchemaDef, SchemaKey, spec::RuleSpec};

/// A compiled bny schema representation
#[derive(Debug)]
pub struct BnySchema {
    pub(super) nodes: Arena<RuleSpec>,
    id: StringId,
}

#[derive(Debug, Clone, Copy)]
pub struct BnyRuleRef {
    pub(super) id: NodeId,
}

impl BnyRuleRef {
    pub fn children(&self, schema: &BnySchema) -> impl Iterator<Item = BnyRuleRef> {
        self.id
            .children(&schema.nodes)
            .map(move |id| BnyRuleRef { id })
    }

    /// Return the actual rule
    pub fn rule<'a>(&'a self, schema: &'a BnySchema) -> &'a RuleSpec {
        schema.nodes[self.id].get()
    }
}

// Export type of nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BnySchemaId(NodeId);

impl BnySchema {
    /// Add a rule node into the tree
    /// TODO: NodeSpec needs to include all PropertySpec/ArgSpec
    fn add_rule(
        nodes: &mut Arena<RuleSpec>,
        spec: RuleSpec,
        parent: Option<BnySchemaId>,
    ) -> BnySchemaId {
        let id = if let Some(parent_id) = parent.map(|p| p.0) {
            parent_id.append_value(spec, nodes)
        } else {
            nodes.new_node(spec)
        };
        BnySchemaId(id)
    }

    /// Interned ID
    pub fn id(&self) -> StringId {
        self.id
    }

    /// Return an iterator of nodes in the root level
    pub fn rules(&self) -> impl Iterator<Item = BnyRuleRef> {
        self.nodes.roots().map(|id| BnyRuleRef { id })
    }

    /// Build a BnySchema from a SchemaDef
    /// TODO: Shit the bed when we encounter duplicate symbolics
    pub fn compile<'a, T: SchemaKey<'a>>(
        intern: &mut BnyInterner,
        schema_def: &'a SchemaDef<'a, T>,
    ) -> BnySchema {
        let mut nodes = Arena::new();

        let mut processing = Vec::new();
        let batch = schema_def.nodes().iter().rev().map(|n| (n, None));
        processing.extend(batch);
        while let Some((node, parent)) = processing.pop() {
            let spec = node.spec(intern);
            let rule = RuleSpec::MatchNode(spec);
            let id = Self::add_rule(&mut nodes, rule, parent);

            match node.child() {
                Some(Rule::Nodes(nodes)) => {
                    processing.extend(nodes.iter().rev().map(move |n| (n, Some(id))))
                }
                Some(Rule::External(schema_ref)) => {
                    let sym = intern.intern(schema_ref.0);
                    Self::add_rule(&mut nodes, RuleSpec::ExternalSchema(sym), Some(id));
                }
                None => {}
            }
        }
        BnySchema {
            nodes,
            id: intern.intern(schema_def.id().0),
        }
    }
}
