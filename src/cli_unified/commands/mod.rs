//! Command execution logic

pub mod font;
pub mod icon;
pub mod media;
pub mod tools;
pub mod tools_extended;

use anyhow::Result;
use std::path::PathBuf;

use crate::cli_unified::args::{Cli, Commands};
use crate::cli_unified::config::MediaConfig;

pub async fn execute(cli: Cli) -> Result<()> {
    // Initialize logging if verbose
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("dx_media=debug".parse()?),
            )
            .init();
    }

    // Load configuration
    let config = MediaConfig::load()?;

    match cli.command {
        Commands::Search {
            query,
            media_type,
            provider,
            limit,
        } => media::cmd_search(&query, &media_type, provider.as_deref(), limit, &cli.format).await,
        Commands::Download {
            asset_id,
            mut output,
            provider,
        } => {
            // Use config directory if output is default
            if output == PathBuf::from(".") {
                output = config.get_media_dir();
                config.ensure_dir(&output)?;
            }
            media::cmd_download(&asset_id, &output, provider.as_deref(), &cli.format).await
        }
        Commands::Icon { command } => {
            icon::execute_icon_command(command, &cli.format, &config).await
        }
        Commands::Font { command } => {
            font::execute_font_command(command, &cli.format, &config).await
        }
        Commands::Tools { command } => tools::execute_tool_command(command, &cli.format).await,
        Commands::Video { command } => tools_extended::execute_video_extended(command).await,
        Commands::Audio { command } => {
            tools_extended::execute_audio_extended(command, &cli.format).await
        }
        Commands::Image { command } => {
            tools_extended::execute_image_extended(command, &cli.format).await
        }
        Commands::Archive { command } => {
            tools_extended::execute_archive_extended(command, &config, &cli.format).await
        }
        Commands::Document { command } => {
            tools_extended::execute_document_extended(command, &cli.format).await
        }
        Commands::Utility { command } => {
            tools_extended::execute_utility_extended(command, &cli.format).await
        }
        Commands::Providers { provider_type } => {
            media::cmd_providers(&provider_type, &cli.format).await
        }
        Commands::Health => media::cmd_health().await,
    }
}
