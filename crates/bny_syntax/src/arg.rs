// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax document

use miette::SourceSpan;

use crate::ProcessedValue;

/// An argument as provided to a node
pub struct BnyArgument {
    pub(super) value: ProcessedValue,
    pub(super) span: SourceSpan,
}

impl BnyArgument {
    /// Arguments processed value
    pub fn value(&self) -> &ProcessedValue {
        &self.value
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }
}
