// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Barney Schema Implementation
//!
//! The bny_schema crate is built atop bny_syntax to
//! provide validation of `.bny` files and rich diagnostics.

mod key;
pub use key::*;

mod schema;
pub use schema::{SchemaDef, SchemaRef};

mod node;
pub use node::NodeDef;

mod rule;
pub use rule::Rule;
