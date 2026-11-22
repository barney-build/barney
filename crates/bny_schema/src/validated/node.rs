// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Schema-validated node accessor

use bny_syntax::BnyNodeRef;

use crate::{
    AsBnyValueType, AsOptionalBnyValueType, NodeSpec, SchemaIndex, SchemaKey, ValidatedDocument,
    compiled::BnyRuleRef,
    validated::{MatchId, ValidationError},
};

/// A validated node returned from ValidatedDocument
#[derive(Debug)]
pub struct ValidatedNode<'a> {
    pub(super) matched: MatchId,
    pub(super) doc: &'a ValidatedDocument<'a>,
    pub(super) schema: SchemaIndex,
}

impl<'a> ValidatedNode<'a> {
    // Child level schema nodes
    pub fn nodes<T: SchemaKey<'a>>(
        &self,
        key: T,
    ) -> Result<impl Iterator<Item = ValidatedNode<'a>>, ValidationError> {
        self.doc
            .get_nodes_for_parent(Some(self.matched), key, self.schema)
    }

    // Grab the name
    pub fn name(&self) -> &'a str {
        let m = self.doc.arena[self.matched.0].get();
        self.doc.intern.resolve(m.node.name(self.doc.tree))
    }

    /// Return a property by identity
    pub fn property<T: AsOptionalBnyValueType<'a>>(
        &self,
        identity: impl SchemaKey<'a>,
    ) -> Result<T, ValidationError> {
        self.doc.get_property(self.matched, self.schema, identity)
    }

    /// Return the value for a single argument
    pub fn argument<T: AsBnyValueType<'a>>(
        &self,
        identity: impl SchemaKey<'a>,
    ) -> Result<T, ValidationError> {
        self.doc.get_argument(self.matched, self.schema, identity)
    }

    /// Return a list of values for a variadic argument range
    pub fn arguments<T: AsBnyValueType<'a>>(
        &self,
        identity: impl SchemaKey<'a>,
    ) -> Result<impl Iterator<Item = T>, ValidationError> {
        self.doc
            .get_arguments(self.matched, self.schema, identity, true)
    }
}

/// Matching results for validation runs
pub(super) enum ValidationResult {
    // matched a rule
    Matched {
        rule: BnyRuleRef,
        index: usize,
        node_spec: NodeSpec,
        node: BnyNodeRef,
    },
    // unmatched, try someone else
    Unmatched,
    // change schema validation context
    ChangeContext(SchemaIndex),
}
