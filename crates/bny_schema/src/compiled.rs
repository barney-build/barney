// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Compiled barney schema

use std::num::TryFromIntError;

use bny_slots::{Block, SlotStorage};
use bny_syntax::{BnyInterner, StringId};
use indextree::{Arena, NodeId};
use thiserror::Error;

use crate::{
    ArgumentSpec, NodeDef, NodeSpec, PropertySpec, Rule, SchemaDef, SchemaKey, spec::RuleSpec,
};

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
    table: BindingTable,
    pub(super) props: SlotStorage<PropertySpec>,
    pub(super) args: SlotStorage<ArgumentSpec>,
}

#[derive(Debug, Clone, Copy)]
pub struct BnyRuleRef {
    pub(crate) id: NodeId,
}

/// Tiny table for binding symbolic references
#[derive(Debug)]
pub(crate) struct BindingTable {
    table: [Option<BindingReference>; MAX_RULES],
}

impl BindingTable {
    /// Return a new binding table
    fn new() -> Self {
        Self {
            table: [None; MAX_RULES],
        }
    }

    /// Insert a binding into the table
    fn insert_binding(
        &mut self,
        symbolic: u8,
        binding: BindingReference,
    ) -> Result<(), SchemaError> {
        let symbolic_idx = symbolic as usize;
        match self.table[symbolic_idx] {
            Some(_) => Err(SchemaError::DuplicateSymbolic { symbolic }),
            None => {
                self.table[symbolic_idx] = Some(binding);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum BindingReference {
    Node {
        parent: Option<BnyRuleRef>,
        local_offset: u8,
    },
    Property {
        id: BnySchemaId,
        local_offset: u8,
    },
    Argument {
        id: BnySchemaId,
        local_offset: u8,
    },
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

    /// Return the propertys permitted for a nodespec in the given rule & schema
    pub fn properties<'a>(
        &self,
        schema: &'a BnySchema,
        spec: &NodeSpec,
    ) -> impl Iterator<Item = &'a PropertySpec> {
        spec.properties
            .iter()
            .flatten()
            .map(|idx| schema.props.get(idx))
    }

    pub fn arguments<'a>(
        &self,
        schema: &'a BnySchema,
        spec: &NodeSpec,
    ) -> impl Iterator<Item = &'a ArgumentSpec> {
        spec.arguments
            .iter()
            .flatten()
            .map(|idx| schema.args.get(idx))
    }
}

// Export type of nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BnySchemaId(NodeId);

impl BnySchema {
    /// Build the initial block of properties for a node definition
    fn build_properties<'a, T: SchemaKey<'a>>(
        property_rules: &mut SlotStorage<PropertySpec>,
        intern: &mut BnyInterner,
        node: &'a NodeDef<'a, T>,
    ) -> Option<Block> {
        match node.props() {
            // Explicit property rules
            Some(props) => {
                let property_block = property_rules.allocate(props.len(), Default::default);
                for (prop_idx, prop) in props.iter().enumerate() {
                    property_rules.set(
                        property_block.0.start + prop_idx,
                        prop.spec(intern, prop_idx),
                    );
                }
                Some(property_block)
            }
            // None supported
            None => None,
        }
    }

    fn build_arguments<'a, T: SchemaKey<'a>>(
        argument_rules: &mut SlotStorage<ArgumentSpec>,
        node: &'a NodeDef<'a, T>,
    ) -> Option<Block> {
        match node.args() {
            // Explicit property rules
            Some(args) => {
                let arg_block = argument_rules.allocate(args.len(), Default::default);
                for (arg_idx, arg) in args.iter().enumerate() {
                    argument_rules.set(arg_block.0.start + arg_idx, arg.spec(arg_idx));
                }
                Some(arg_block)
            }
            // None supported
            None => None,
        }
    }

    /// Add a rule node into the tree
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
    pub fn compile<'a, T: SchemaKey<'a>>(
        intern: &mut BnyInterner,
        schema_def: &'a SchemaDef<'a, T>,
    ) -> Result<BnySchema, SchemaError> {
        let mut nodes = Arena::new();
        let mut table = BindingTable::new();

        let mut processing = Vec::new();
        processing.extend(
            schema_def
                .nodes()
                .iter()
                .map(|n| (n, None))
                .enumerate()
                .rev(),
        );
        let mut property_rules: SlotStorage<PropertySpec> = SlotStorage::new();
        let mut argument_rules: SlotStorage<ArgumentSpec> = SlotStorage::new();

        while let Some((index, (node, parent))) = processing.pop() {
            let property_area = Self::build_properties(&mut property_rules, intern, node);
            let argument_area = Self::build_arguments(&mut argument_rules, node);
            let spec = node.spec(intern, index, property_area, argument_area);
            let symbolic = spec.symbolic;
            let rule = RuleSpec::MatchNode(spec);
            let id = Self::add_rule(&mut nodes, rule, parent);

            // Bind the node
            let binding = BindingReference::Node {
                parent: parent.as_ref().map(|p| BnyRuleRef { id: p.0 }),
                local_offset: u8::try_from(index)?,
            };
            table.insert_binding(symbolic, binding)?;

            // Bind the properties
            for (local_offset, prop_idx) in property_area.iter().flatten().enumerate() {
                let prop = property_rules.get(prop_idx);
                let binding = BindingReference::Property {
                    id,
                    local_offset: u8::try_from(local_offset)?,
                };
                table.insert_binding(prop.symbolic, binding)?;
            }

            // Bind the arguments
            for (local_offset, arg_idx) in argument_area.iter().flatten().enumerate() {
                let arg = argument_rules.get(arg_idx);
                let binding = BindingReference::Argument {
                    id,
                    local_offset: u8::try_from(local_offset)?,
                };
                table.insert_binding(arg.symbolic, binding)?;
            }

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
            table,
            props: property_rules,
            args: argument_rules,
        })
    }

    /// Resolve symbolic to the binding reference
    pub(crate) fn resolve_symbolic(&self, symbol: u8) -> Option<BindingReference> {
        self.table.table[symbol as usize]
    }
}
