// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Barney Schema Registry

use std::collections::HashMap;

use bny_syntax::{BnyInterner, BnyTree, StringId};
use itertools::Itertools;
use thiserror::Error;

use crate::{BnySchema, SchemaRef, ValidatedDocument, spec::RuleSpec};

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
        for schema in self.schemas.iter_mut() {
            for node in schema.nodes.iter_node_ids().collect_vec() {
                let rewrite = match schema.nodes[node].get() {
                    RuleSpec::ExternalSchema(string_id) => match self.mapping.get(string_id) {
                        Some(idx) => Some(idx),
                        None => {
                            return Err(RegistryError::UnknownSchema {
                                id: intern.resolve(*string_id).to_owned(),
                            });
                        }
                    },
                    _ => None,
                };
                if let Some(idx) = rewrite {
                    *schema.nodes[node].get_mut() = RuleSpec::ExternalSchemaIndexed(*idx);
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
        })
    }
}

impl LinkedSchemaRegistry {
    pub fn validate<'a, 'b>(&'a self, tree: &'a BnyTree<'b>) -> ValidatedDocument<'a, 'b> {
        ValidatedDocument::new(self, tree)
    }
}
