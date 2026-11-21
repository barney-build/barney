// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Barney Schema Registry

use std::collections::HashMap;

use bny_diag::DiagnosticSink;
use bny_slots::SlotStorage;
use bny_syntax::{BnyInterner, BnyTree, StringId};
use itertools::Itertools;
use thiserror::Error;

use crate::{BnySchema, SchemaRef, ValidatedDocument, compiled::BnyRuleRef, spec::RuleSpec};

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("schema is already installed")]
    AlreadyRegistered,

    #[error("unknown entry schema")]
    UnknownEntrySchema,

    #[error("unknown schema: {id}")]
    UnknownSchema { id: String },
}

#[derive(Debug, Clone, Copy)]
pub struct SchemaIndex(pub(super) usize);

#[derive(Debug)]
pub struct SchemaRegistry<'a> {
    schemas: Vec<BnySchema>,
    entry: SchemaRef<'a>,
    mapping: HashMap<StringId, SchemaIndex>,
}

#[derive(Debug)]
pub struct LinkedSchemaRegistry {
    pub(super) entry: SchemaIndex,
    pub(super) schemas: Vec<BnySchema>,
    nodes: SlotStorage<Option<NodeRuleLookup>>,
    num_nodes: usize,
}

/// Rule lookups
#[derive(Debug)]
pub struct NodeRuleLookup {
    pub schema: SchemaIndex,
    pub rule: BnyRuleRef,
}

impl<'a> SchemaRegistry<'a> {
    /// Build a new schema registry defaulting to the entry schema
    pub fn new(entry: SchemaRef<'a>) -> Self {
        Self {
            entry,
            schemas: vec![],
            mapping: HashMap::new(),
        }
    }

    /// Install the schema
    pub fn install(&mut self, schema: BnySchema) -> Result<(), RegistryError> {
        let idx = SchemaIndex(self.schemas.len());
        let nom = schema.id();
        if let std::collections::hash_map::Entry::Vacant(e) = self.mapping.entry(nom) {
            e.insert(idx);
            self.schemas.push(schema);
            Ok(())
        } else {
            Err(RegistryError::AlreadyRegistered)
        }
    }

    // Ensure everything is linked and ticketyboo
    pub fn link(mut self, intern: &mut BnyInterner) -> Result<LinkedSchemaRegistry, RegistryError> {
        let mut global_node_idx = 0;
        let mut slots = SlotStorage::new();

        for (schema_idx, schema) in self.schemas.iter_mut().enumerate() {
            for node in schema.nodes.iter_node_ids().collect_vec() {
                let (rewrite_external, rewrite_node) = match schema.nodes[node].get() {
                    RuleSpec::ExternalSchema(string_id) => match self.mapping.get(string_id) {
                        Some(idx) => (Some(idx), None),
                        None => {
                            return Err(RegistryError::UnknownSchema {
                                id: intern.resolve(*string_id).to_owned(),
                            });
                        }
                    },
                    RuleSpec::MatchNode(n) => (None, Some(n)),
                    _ => (None, None),
                };
                if let Some(idx) = rewrite_external {
                    *schema.nodes[node].get_mut() = RuleSpec::ExternalSchemaIndexed(*idx);
                } else if let Some(n) = rewrite_node {
                    let mut n_copy = *n;
                    n_copy.global_index = Some(global_node_idx);
                    *schema.nodes[node].get_mut() = RuleSpec::MatchNode(n_copy);
                    slots.allocate(1, || None);
                    slots.set(
                        global_node_idx,
                        Some(NodeRuleLookup {
                            schema: SchemaIndex(schema_idx),
                            rule: BnyRuleRef { id: node },
                        }),
                    );
                    global_node_idx += 1;
                }
            }
        }

        let our_id = intern.intern(self.entry.0);
        let entry = self
            .mapping
            .get(&our_id)
            .ok_or(RegistryError::UnknownEntrySchema)?;

        Ok(LinkedSchemaRegistry {
            entry: *entry,
            schemas: self.schemas,
            num_nodes: global_node_idx,
            nodes: slots,
        })
    }
}

impl LinkedSchemaRegistry {
    pub fn validate<'a>(
        &'a self,
        tree: &'a BnyTree<'a>,
        intern: &'a BnyInterner,
        sink: &mut DiagnosticSink,
    ) -> ValidatedDocument<'a> {
        ValidatedDocument::new(self, tree, intern, sink)
    }

    /// Return the entry schema
    pub fn entry(&self) -> &BnySchema {
        &self.schemas[self.entry.0]
    }

    pub fn global_nodes(&self) -> usize {
        self.num_nodes
    }

    /// return globally registered node rule
    pub(crate) fn global_node(&self, index: usize) -> Option<&RuleSpec> {
        self.nodes
            .get(index)
            .as_ref()
            .map(|n| self.schemas[n.schema.0].nodes[n.rule.id].get())
    }
}
