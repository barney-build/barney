// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Barney syntax entry support

use kdl::KdlEntry;
use miette::SourceSpan;

mod tag;
pub use tag::*;

use crate::ProcessedValue;

/// A baked entry, ie anything *after* the node declaration but not within the children.
/// Note a naked entry (non-property) can still be a non-string, in that the parent node
/// itself has a valid string ID already so subsequent arguments/etc may be integers or
/// whichever.
#[derive(Debug)]
pub struct ProcessedEntry {
    /// Optionally specifies if there is a binding: Property or Variable
    pub tag: Option<EntryTag>,
    pub value: ProcessedValue,
    span: SourceSpan,
}

impl ProcessedEntry {
    pub fn span(&self) -> SourceSpan {
        self.span
    }
}

impl<'a> TryFrom<&'a KdlEntry> for ProcessedEntry {
    type Error = crate::Error;

    fn try_from(entry: &'a KdlEntry) -> Result<Self, Self::Error> {
        let tag: Option<EntryTag> = if let Some(ty) = entry.ty() {
            Some(ty.try_into()?)
        } else {
            None
        };
        Ok(Self {
            tag,
            span: entry.span(),
            value: entry.value().into(),
        })
    }
}
