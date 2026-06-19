//! Output formatting for CLI commands.

use colored::Colorize;

use crate::cli::args::OutputFormat;
use crate::error::Result;
use crate::types::{MediaAsset, SearchResult};

/// Output formatter for CLI results.
#[derive(Debug, Clone)]
pub struct OutputFormatter {
    format: OutputFormat,
    quiet: bool,
}

impl OutputFormatter {
    /// Create a new output formatter.
    #[must_use]
    pub fn new(format: OutputFormat, quiet: bool) -> Self {
        Self { format, quiet }
    }

    /// Format and print search results.
    pub fn format_search_results(&self, result: &SearchResult) -> Result<()> {
        if self.quiet && matches!(self.format, OutputFormat::Text) {
            return Ok(());
        }

        match self.format {
            OutputFormat::Text => self.format_search_results_text(result),
            OutputFormat::Json => self.format_search_results_json(result, true),
            OutputFormat::JsonCompact => self.format_search_results_json(result, false),
            OutputFormat::Tsv => self.format_search_results_tsv(result),
        }
    }

    /// Format search results as human-readable text.
    fn format_search_results_text(&self, result: &SearchResult) -> Result<()> {
        // Header
        println!(
            "{} Found {} results for '{}' in {}ms",
            "✓".green().bold(),
            result.assets.len().to_string().cyan(),
            result.query.bold(),
            result.duration_ms
        );

        if !result.provider_errors.is_empty() {
            for (provider, error) in &result.provider_errors {
                println!("  {} {}: {}", "⚠".yellow(), provider, error.dimmed());
            }
        }

        println!();

        // Results
        for (i, asset) in result.assets.iter().enumerate() {
            self.format_asset_text(i + 1, asset);
        }

        // Footer
        if result.total_count > result.assets.len() {
            println!(
                "{}",
                format!(
                    "Showing {} of {} total results. Use --page to see more.",
                    result.assets.len(),
                    result.total_count
                )
                .dimmed()
            );
        }

        Ok(())
    }

    /// Format a single asset as text.
    fn format_asset_text(&self, index: usize, asset: &MediaAsset) {
        let type_badge = match asset.media_type {
            crate::types::MediaType::Image => "IMG".cyan(),
            crate::types::MediaType::Video => "VID".magenta(),
            crate::types::MediaType::Audio => "AUD".yellow(),
            crate::types::MediaType::Gif => "GIF".green(),
            crate::types::MediaType::Vector => "SVG".blue(),
            crate::types::MediaType::Document => "DOC".bright_yellow(),
            crate::types::MediaType::Model3D => "3D ".bright_magenta(),
            crate::types::MediaType::Data => "DAT".bright_blue(),
            crate::types::MediaType::Code => "COD".bright_green(),
            crate::types::MediaType::Text => "TXT".white(),
        };

        println!(
            "{:>3}. {} {} {}",
            index.to_string().dimmed(),
            format!("[{}]", type_badge).bold(),
            asset.title.bold(),
            format!("({}:{})", asset.provider, asset.id).dimmed()
        );

        // Author info
        if let Some(ref author) = asset.author {
            if !author.is_empty() {
                println!("     {} {}", "by".dimmed(), author);
            }
        }

        // Dimensions
        if let (Some(w), Some(h)) = (asset.width, asset.height) {
            println!("     {} {}x{}", "size".dimmed(), w, h);
        }

        // License
        println!(
            "     {} {}",
            "license".dimmed(),
            asset.license.as_str().green()
        );

        // Download command hint
        println!(
            "     {} dx download {}:{}",
            "→".dimmed(),
            asset.provider,
            asset.id
        );

        println!();
    }

    /// Format search results as JSON.
    fn format_search_results_json(&self, result: &SearchResult, pretty: bool) -> Result<()> {
        let json = serde_json::json!({
            "query": result.query,
            "media_type": result.media_type.map(|t| t.as_str()),
            "total_count": result.total_count,
            "returned_count": result.assets.len(),
            "duration_ms": result.duration_ms,
            "providers_searched": result.providers_searched,
            "provider_errors": result.provider_errors.iter()
                .map(|(p, e)| serde_json::json!({"provider": p, "error": e}))
                .collect::<Vec<_>>(),
            "assets": result.assets.iter().map(|a| self.asset_to_json(a)).collect::<Vec<_>>(),
        });

        if pretty {
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            println!("{}", serde_json::to_string(&json)?);
        }

        Ok(())
    }

    /// Convert an asset to JSON value.
    fn asset_to_json(&self, asset: &MediaAsset) -> serde_json::Value {
        serde_json::json!({
            "id": asset.id,
            "provider": asset.provider,
            "media_type": asset.media_type.as_str(),
            "title": asset.title,
            "author": asset.author,
            "author_url": asset.author_url,
            "source_url": asset.source_url,
            "download_url": asset.download_url,
            "preview_url": asset.preview_url,
            "width": asset.width,
            "height": asset.height,
            "file_size": asset.file_size,
            "mime_type": asset.mime_type,
            "license": asset.license.as_str(),
            "provider_metadata": asset.provider_metadata,
            "provenance": asset.provenance(),
            "tags": asset.tags,
        })
    }

    /// Format search results as TSV.
    fn format_search_results_tsv(&self, result: &SearchResult) -> Result<()> {
        // Header
        println!("provider\tid\ttype\ttitle\tauthor\twidth\theight\tdownload_url");

        // Data
        for asset in &result.assets {
            let author = asset.author.as_deref().unwrap_or("");
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                asset.provider,
                asset.id,
                asset.media_type.as_str(),
                asset.title.replace('\t', " "),
                author.replace('\t', " "),
                asset.width.map(|w| w.to_string()).unwrap_or_default(),
                asset.height.map(|h| h.to_string()).unwrap_or_default(),
                asset.download_url,
            );
        }

        Ok(())
    }

    /// Format a single asset for display.
    pub fn format_asset(&self, asset: &MediaAsset) -> Result<()> {
        match self.format {
            OutputFormat::Text => {
                self.format_asset_text(1, asset);
            }
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&self.asset_to_json(asset))?
                );
            }
            OutputFormat::JsonCompact => {
                println!("{}", serde_json::to_string(&self.asset_to_json(asset))?);
            }
            OutputFormat::Tsv => {
                println!(
                    "{}\t{}\t{}\t{}",
                    asset.provider,
                    asset.id,
                    asset.media_type.as_str(),
                    asset.title
                );
            }
        }
        Ok(())
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new(OutputFormat::Text, false)
    }
}
