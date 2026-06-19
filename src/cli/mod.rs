//! CLI module for the dx command-line interface.
//!
//! This module provides the command-line interface for interacting with
//! the DX Media library.

mod args;
mod commands;
mod output;

pub use args::{Args, Command};
pub use output::OutputFormatter;

use crate::error::Result;

/// Run the CLI application.
pub async fn run() -> Result<()> {
    // Initialize logging
    init_logging();

    // Parse command line arguments
    let args = Args::parse_args();

    // Execute the command
    commands::execute(args).await
}

/// Initialize the logging system.
fn init_logging() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
