//! Search command implementation.

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::DxMedia;
use crate::cli::OutputFormatter;
use crate::cli::args::{OutputFormat, SearchArgs};
use crate::error::Result;
use crate::tools::ToolOutput;
use crate::types::{SearchQuery, SearchResult};

/// Execute the search command.
pub async fn execute(args: SearchArgs, format: OutputFormat, quiet: bool) -> Result<()> {
    let dx = DxMedia::new()?;

    // Show progress indicator
    let spinner = if !quiet && matches!(format, OutputFormat::Text) {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        let search_type = if args.all {
            "all providers & scrapers"
        } else {
            "providers"
        };
        let mode_str = match args.mode {
            crate::cli::args::SearchModeArg::Quantity => "⚡ quantity",
            crate::cli::args::SearchModeArg::Quality => "🎯 quality",
        };
        pb.set_message(format!(
            "Searching {} for '{}' ({} mode)...",
            search_type,
            args.query_string(),
            mode_str
        ));
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        Some(pb)
    } else {
        None
    };

    // Convert CLI mode to types::SearchMode
    let search_mode: crate::types::SearchMode = args.mode.into();

    // Execute search - use unified search if --all is specified
    let result = if args.all {
        dx.search_all_with_mode(&args.query_string(), args.count, search_mode)
            .await?
    } else {
        // Build the search query for regular search
        let mut query = SearchQuery::new(args.query_string());
        query.count = args.count;
        query.page = args.page;
        query.media_type = args.media_type.and_then(Into::into);
        query.providers = args.providers.clone();
        query.orientation = args.orientation.map(Into::into);
        query.color = args.color.clone();
        query.mode = search_mode;

        dx.search_query(&query).await?
    };

    // Clear spinner
    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    let download_output = if args.download && !result.assets.is_empty() {
        if !quiet && matches!(format, OutputFormat::Text) {
            println!();
            println!("{}", "Downloading first result...".cyan());
        }

        let asset = &result.assets[0];
        let output = if let Some(ref output_dir) = args.output {
            dx.download_to_with_receipt(asset, std::path::Path::new(output_dir))
                .await?
        } else {
            dx.download_with_receipt(asset).await?
        };
        Some(output)
    } else {
        None
    };

    let formatter = OutputFormatter::new(format, quiet);

    match (format, download_output.as_ref()) {
        (OutputFormat::Json, Some(output)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&search_download_json(&result, output))?
            );
        }
        (OutputFormat::JsonCompact, Some(output)) => {
            println!(
                "{}",
                serde_json::to_string(&search_download_json(&result, output))?
            );
        }
        (_, Some(output)) => {
            formatter.format_search_results(&result)?;
            super::download::print_download_output(output, format, quiet)?;
        }
        (_, None) => formatter.format_search_results(&result)?,
    }

    Ok(())
}

fn search_download_json(result: &SearchResult, download: &ToolOutput) -> serde_json::Value {
    serde_json::json!({
        "success": download.success,
        "search": result.with_asset_provenance(),
        "download": download,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{ToolOutput, ToolReceipt};

    #[test]
    fn search_download_json_includes_tool_receipt() {
        let result = crate::types::SearchResult::new("mars");
        let download = ToolOutput::success_with_path("Downloaded", "downloads/mars.jpg")
            .with_receipt(ToolReceipt::provider_backed("media.download", "nasa"));

        let value = search_download_json(&result, &download);

        assert_eq!(value["success"], true);
        assert!(value["search"]["assets"].is_array());
        assert_eq!(value["download"]["metadata"]["tool.name"], "media.download");
        assert_eq!(
            value["download"]["metadata"]["tool.source_kind"],
            "provider-backed"
        );
        assert_eq!(value["download"]["metadata"]["tool.provider"], "nasa");
    }
}
