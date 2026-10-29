// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Strong types can have tags associated with them
//! which also permit a referencing system
//! Internally they're still strings

mod entry;
pub use entry::EntryTag;
mod node;
pub use node::NodeTag;

/// Explicitly set type tag
#[derive(Debug, Clone)]
pub enum Type {
    /// Accepts string data only
    String,

    /// Accepts (merges) list of string data only
    List,
}
