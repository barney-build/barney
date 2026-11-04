// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax helpers / errors

mod arg;
pub use arg::BnyArgument;
mod prop;
use miette::Diagnostic;
pub use prop::BnyProperty;

mod interner;
pub use interner::{BnyInterner, StringId};

mod spec;
pub use spec::BnySpec;

mod tree;
use thiserror::Error;
pub use tree::{BnyNodeRef, BnyTree};
mod value;
pub use value::ProcessedValue;

mod symbol;
pub use symbol::*;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Kdl(#[from] kdl::KdlError),
}
