// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! type erased definitions into specifications

use bny_slots::Block;
use bny_syntax::StringId;

use crate::{BnyValueType, Occurrence, registry::SchemaIndex};

/// Type-erasure contract for NodeDef
#[derive(Debug, Clone, Copy)]
pub struct NodeSpec {
    // If we have an ID we're an explicit match
    pub(crate) id: Option<StringId>,

    // Symbolic match value
    pub(crate) symbolic: u8,

    // Index within parent context
    pub(crate) local_index: usize,

    /// Storage address of property rules
    pub(crate) properties: Option<Block>,

    /// Permitted occurrence rules
    pub(crate) occurrence: Occurrence,

    /// Empty / useless until hitting linked registry
    pub(crate) global_index: Option<usize>,
}

/// Type-erasure contract for PropertyDef
#[derive(Debug, Default)]
pub struct PropertySpec {
    /// Empty string ID means its a wildcard rule
    pub(crate) id: Option<StringId>,

    /// Symbolic match value
    pub(crate) symbolic: u8,

    /// Index within parent context (Node)
    pub(crate) local_index: usize,

    /// Accepted value type
    pub(crate) kind: BnyValueType,

    /// Required property?
    pub(crate) required: bool,
}

/// Encapsulates node rules into singular rule
#[derive(Debug)]
pub enum RuleSpec {
    MatchNode(NodeSpec),
    ExternalSchema(StringId),
    ExternalSchemaIndexed(SchemaIndex),
}
