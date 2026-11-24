// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Build queue entry point for Barney CLI

use std::path::Path;

use miette::{Context, IntoDiagnostic};

// Load bootstrap config, handle emission of miette report if needed
pub async fn run_command(path: &Path) -> Result<(), miette::Report> {
    // Ensure we have a real path firstly!
    let _path = path
        .canonicalize()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to resolve path: {path:?}"))?;

    eprintln!("Not yet implemented");
    Ok(())
}
