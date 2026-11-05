// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! type erased definitions into specifications

use bny_syntax::StringId;

use crate::registry::SchemaIndex;

#[derive(Debug)]
pub struct NodeSpec {
    // If we have an ID we're an explicit match
    pub(crate) id: Option<StringId>,

    // Symbolic match value
    pub(crate) symbolic: Option<u16>,
}

/// Encapsulates node rules into singular rule
#[derive(Debug)]
pub enum RuleSpec {
    MatchNode(NodeSpec),
    ExternalSchema(StringId),
    ExternalSchemaIndexed(SchemaIndex),
}
