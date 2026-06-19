//! Unified Media CLI - Access media, icons, and fonts from one interface

pub mod args;
pub mod args_extended;
pub mod commands;
pub mod config;
mod config_format;
pub mod output;

use anyhow::Result;

pub async fn run() -> Result<()> {
    use clap::Parser;
    let cli = args::Cli::parse();
    commands::execute(cli).await
}
