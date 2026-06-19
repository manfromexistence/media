//! Download command implementation.

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};

use crate::DxMedia;
use crate::cli::args::{DownloadArgs, OutputFormat};
use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use crate::types::MediaType;

/// Execute the download command.
pub async fn execute(args: DownloadArgs, format: OutputFormat, quiet: bool) -> Result<()> {
    let dx = DxMedia::new()?;

    // Parse asset ID (format: provider:id)
    let (provider_name, asset_id) = parse_asset_id(&args.asset_id)?;

    if !quiet {
        println!("{} {}:{}", "Looking up".cyan(), provider_name, asset_id);
    }

    // Try to get the asset directly by ID first
    let asset = if let Some(provider) = dx.registry().get(provider_name) {
        if let Ok(Some(asset)) = provider.get_by_id(asset_id).await {
            Some(asset)
        } else {
            None
        }
    } else {
        None
    };

    // If direct lookup failed, fall back to search
    let asset = if let Some(asset) = asset {
        asset
    } else {
        // Search for the asset by ID in the specific provider
        let mut query = crate::types::SearchQuery::new(asset_id);
        query.providers = vec![provider_name.to_string()];
        query.count = 20; // Limit to avoid rate limits on anonymous requests

        let search_result = dx.search_query(&query).await?;

        // Find the asset with matching ID
        search_result
            .assets
            .into_iter()
            .find(|a| a.id == asset_id && a.provider == provider_name)
            .ok_or_else(|| DxError::NoResults {
                query: format!("{}:{}", provider_name, asset_id),
            })?
    };

    // Show progress
    let spinner = if !quiet && matches!(format, OutputFormat::Text) {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message(format!("Downloading '{}'...", asset.title));
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        Some(pb)
    } else {
        None
    };

    let mut output = if let Some(ref output_dir) = args.output {
        dx.download_to_with_receipt(&asset, Path::new(output_dir))
            .await?
    } else {
        dx.download_with_receipt(&asset).await?
    };

    if let Some(ref filename) = args.filename {
        output = rename_downloaded_output(output, filename, asset.media_type).await?;
    }

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    print_download_output(&output, format, quiet)?;

    Ok(())
}

/// Parse asset ID in format "provider:id" or just "id".
fn parse_asset_id(asset_id: &str) -> Result<(&str, &str)> {
    if let Some((provider, id)) = asset_id.split_once(':') {
        Ok((provider, id))
    } else {
        // Default to openverse if no provider specified
        Ok(("openverse", asset_id))
    }
}

async fn rename_downloaded_output(
    mut output: ToolOutput,
    filename: &str,
    media_type: MediaType,
) -> Result<ToolOutput> {
    let path = first_output_path(&output)?;
    validate_renamed_extension(&path, filename, media_type)?;

    let new_path = path.parent().unwrap_or(Path::new(".")).join(filename);
    tokio::fs::rename(&path, &new_path)
        .await
        .map_err(|e| DxError::FileIo {
            path: path.clone(),
            message: format!("Failed to rename file: {}", e),
            source: Some(e),
        })?;

    output.output_paths = vec![new_path.clone()];
    output = output.with_output_type_validation(&new_path, media_type);
    output = output
        .with_metadata("tool.output_renamed", "true")
        .with_metadata("tool.final_output_path", new_path.display().to_string());

    Ok(output)
}

fn validate_renamed_extension(path: &Path, filename: &str, media_type: MediaType) -> Result<()> {
    let original_extension = path.extension().and_then(|ext| ext.to_str());
    let Some(extension) = Path::new(filename).extension().and_then(|ext| ext.to_str()) else {
        if original_extension.is_some() {
            return Err(DxError::Download {
                url: path.display().to_string(),
                message: "Custom filename must preserve the downloaded file extension so receipt type validation remains accurate".to_string(),
            });
        }
        return Ok(());
    };

    if let Some(original_extension) = original_extension {
        if !extension.eq_ignore_ascii_case(original_extension) {
            return Err(DxError::Download {
                url: path.display().to_string(),
                message: format!(
                    "Custom filename extension '.{extension}' must preserve downloaded extension '.{original_extension}' so receipt type validation remains accurate"
                ),
            });
        }
    }

    if !media_type.matches_extension(extension) {
        return Err(DxError::Download {
            url: path.display().to_string(),
            message: format!(
                "Custom filename extension '.{extension}' does not match declared media type '{}'",
                media_type.as_str()
            ),
        });
    }

    Ok(())
}

fn first_output_path(output: &ToolOutput) -> Result<PathBuf> {
    output
        .output_paths
        .first()
        .cloned()
        .ok_or_else(|| DxError::Download {
            url: output
                .metadata
                .get("tool.download_url")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            message: "Download completed without an output path".to_string(),
        })
}

pub(super) fn print_download_output(
    output: &ToolOutput,
    format: OutputFormat,
    quiet: bool,
) -> Result<()> {
    match format {
        OutputFormat::Text => {
            if !quiet {
                for line in download_text_lines(output)? {
                    println!("{line}");
                }
            }
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(output)?),
        OutputFormat::JsonCompact => println!("{}", serde_json::to_string(output)?),
        OutputFormat::Tsv => println!("{}", download_tsv_row(output)?),
    }

    Ok(())
}

fn download_text_lines(output: &ToolOutput) -> Result<Vec<String>> {
    let path = first_output_path(output)?;
    let mut lines = vec![format!(
        "{} {}",
        "Downloaded:".green().bold(),
        path.display()
    )];

    push_download_metadata_line(output, &mut lines, "tool.source_kind", "Source");
    push_download_metadata_line(output, &mut lines, "tool.provider", "Provider");
    push_download_metadata_line(output, &mut lines, "tool.license", "License");
    push_download_metadata_line(output, &mut lines, "tool.receipt_completeness", "Receipt");
    push_download_metadata_line(
        output,
        &mut lines,
        "tool.type_validation",
        "Type validation",
    );
    push_download_metadata_line(
        output,
        &mut lines,
        "tool.output_type_validation",
        "Output validation",
    );

    Ok(lines)
}

fn push_download_metadata_line(
    output: &ToolOutput,
    lines: &mut Vec<String>,
    key: &str,
    label: &str,
) {
    if let Some(value) = output.metadata.get(key).filter(|value| !value.is_empty()) {
        lines.push(format!("{label}: {value}"));
    }
}

fn download_tsv_row(output: &ToolOutput) -> Result<String> {
    let path = first_output_path(output)?;
    let source_kind = output
        .metadata
        .get("tool.source_kind")
        .map(String::as_str)
        .unwrap_or("unknown");
    let provider = output
        .metadata
        .get("tool.provider")
        .map(String::as_str)
        .unwrap_or("");
    let license = output
        .metadata
        .get("tool.license")
        .map(String::as_str)
        .unwrap_or("unknown");

    Ok(format!(
        "{}\t{}\t{}\t{}",
        path.display(),
        source_kind,
        provider,
        license
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renamed_extension_must_match_declared_media_type() {
        let err =
            validate_renamed_extension(Path::new("downloads/photo"), "photo.txt", MediaType::Image)
                .expect_err("image downloads must not be renamed to text outputs");

        match err {
            DxError::Download { message, .. } => {
                assert!(message.contains("declared media type"));
                assert!(message.contains("image"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[test]
    fn renamed_extension_must_preserve_downloaded_extension() {
        let err = validate_renamed_extension(
            Path::new("downloads/photo.png"),
            "photo.jpg",
            MediaType::Image,
        )
        .expect_err("renaming must not make actual MIME validation stale");

        match err {
            DxError::Download { message, .. } => {
                assert!(message.contains("preserve downloaded extension"));
                assert!(message.contains(".png"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[test]
    fn tsv_download_row_includes_receipt_source_fields() {
        let output = ToolOutput::success_with_path("Downloaded", "downloads/photo.png")
            .with_metadata("tool.source_kind", "provider-backed")
            .with_metadata("tool.provider", "fixture")
            .with_metadata("tool.license", "cc0");

        assert_eq!(
            download_tsv_row(&output).expect("row should format"),
            "downloads/photo.png\tprovider-backed\tfixture\tcc0"
        );
    }

    #[test]
    fn text_download_output_lines_include_receipt_and_type_evidence() {
        let output = ToolOutput::success_with_path("Downloaded", "downloads/photo.png")
            .with_metadata("tool.source_kind", "provider-backed")
            .with_metadata("tool.provider", "fixture")
            .with_metadata("tool.license", "cc0")
            .with_metadata("tool.receipt_completeness", "explicit")
            .with_metadata("tool.type_validation", "passed");

        let lines = download_text_lines(&output).expect("text lines should format");

        assert!(lines.iter().any(|line| line.contains("Downloaded:")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Source: provider-backed"))
        );
        assert!(lines.iter().any(|line| line.contains("Provider: fixture")));
        assert!(lines.iter().any(|line| line.contains("License: cc0")));
        assert!(lines.iter().any(|line| line.contains("Receipt: explicit")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Type validation: passed"))
        );
    }
}
