// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Build queue entry point for Barney CLI

use std::path::Path;

use bny_recipe::bootstrap::BootstrapSpec;
use miette::{Context, IntoDiagnostic};
use tracing::info;

// Load bootstrap config, handle emission of miette report if needed
pub async fn run_command(path: &Path) -> Result<(), miette::Report> {
    // Ensure we have a real path firstly!
    let path = path
        .canonicalize()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to resolve path: {path:?}"))?;
    let distro = BootstrapSpec::from_path(&path)?;
    info!(config = ?path, "Loaded distro configuration: {distro:#?}");

    println!("Distro loaded: {distro:#?}");

    Ok(())
}
