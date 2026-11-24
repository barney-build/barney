// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

use std::{path::PathBuf, process};

use clap::{Parser, Subcommand};
use cli::build;
use tokio::runtime;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CliEntry {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the distro
    Build { path: Option<PathBuf> },
}

/// Guarded main to enable future workers without bricking the
/// architecture.
fn main() {
    // TODO: Slightly more graceful init of tokio please
    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    // Run internal async enabled runtime
    rt.block_on(async move { priv_async_main().await });
}

/// Main entry point
async fn priv_async_main() {
    let cli = CliEntry::parse();

    let result = match &cli.command {
        // handle build command
        Commands::Build { path } => {
            let path = path.clone().unwrap_or(PathBuf::from("bootstrap.kdl"));
            build::run_command(&path).await
        }
    };

    // TODO: capture intended failure code from decorated error
    if let Err(e) = result {
        eprintln!("{e:?}");
        process::exit(cli::ExitCode::Abnormal.into())
    }
}
