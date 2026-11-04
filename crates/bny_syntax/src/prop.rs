// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax document

use miette::SourceSpan;

use crate::{ProcessedValue, StringId};

/// A property of a node
#[derive(Debug)]
pub struct BnyProperty {
    pub(super) name: StringId,
    pub(super) value: ProcessedValue,
    pub(super) name_span: SourceSpan,
    pub(super) value_span: SourceSpan,
    pub(super) span: SourceSpan,
}

impl BnyProperty {
    /// Property name
    pub fn name(&self) -> StringId {
        self.name
    }

    /// Property value
    pub fn value(&self) -> &ProcessedValue {
        &self.value
    }

    /// Return span for the whole property pair
    pub fn span(&self) -> SourceSpan {
        self.span
    }

    /// Return span for the identifier itself
    pub fn name_span(&self) -> SourceSpan {
        self.name_span
    }

    /// Return span for the value itself
    pub fn value_span(&self) -> SourceSpan {
        self.value_span
    }
}
