// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Compiled barney schema

use std::num::TryFromIntError;

use bny_syntax::{BnyInterner, StringId};
use indextree::{Arena, NodeId};
use thiserror::Error;

use crate::{Rule, SchemaDef, SchemaKey, spec::RuleSpec};

const MAX_RULES: usize = (u8::MAX as usize) + 1;

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("duplicate symbolic binding: {symbolic}")]
    DuplicateSymbolic { symbolic: u8 },

    #[error(transparent)]
    IndexOverflow(#[from] TryFromIntError),
}

/// A compiled bny schema representation
#[derive(Debug)]
pub struct BnySchema {
    pub(super) nodes: Arena<RuleSpec>,
    id: StringId,

    // tiny table for easy rules
    map: [Option<BindingReference>; MAX_RULES],
}

#[derive(Debug, Clone, Copy)]
pub struct BnyRuleRef {
    pub(super) id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BindingReference {
    // The parent rule
    pub(crate) parent: Option<BnyRuleRef>,

    // Offset within the rule
    pub(crate) local_offset: u8,
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
    ) -> Result<BnySchema, SchemaError> {
        let mut nodes = Arena::new();
        let mut table: [Option<BindingReference>; MAX_RULES] = [None; 256];

        let mut processing = Vec::new();
        processing.extend(
            schema_def
                .nodes()
                .iter()
                .map(|n| (n, None))
                .enumerate()
                .rev(),
        );
        while let Some((index, (node, parent))) = processing.pop() {
            let spec = node.spec(intern, index);
            let symbolic = spec.symbolic;
            let symbolic_idx = symbolic as usize;
            let rule = RuleSpec::MatchNode(spec);
            let id = Self::add_rule(&mut nodes, rule, parent);

            // Symbolic mapping.
            table[symbolic_idx] = match table[symbolic_idx] {
                Some(_) => return Err(SchemaError::DuplicateSymbolic { symbolic }),
                None => Some(BindingReference {
                    parent: parent.as_ref().map(|p| BnyRuleRef { id: p.0 }),
                    local_offset: u8::try_from(index)?,
                }),
            };

            match node.child() {
                Some(Rule::Nodes(nodes)) => {
                    processing.extend(nodes.iter().map(move |n| (n, Some(id))).enumerate().rev());
                }
                Some(Rule::External(schema_ref)) => {
                    let sym = intern.intern(schema_ref.0);
                    Self::add_rule(&mut nodes, RuleSpec::ExternalSchema(sym), Some(id));
                }
                None => {}
            }
        }
        Ok(BnySchema {
            nodes,
            id: intern.intern(schema_def.id().0),
            map: table,
        })
    }

    pub(crate) fn resolve_symbolic(&self, symbol: u8) -> Option<BindingReference> {
        self.map[symbol as usize]
    }
}
