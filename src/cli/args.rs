//! Command-line argument parsing.

use clap::{Parser, Subcommand, ValueEnum};

const CLI_ABOUT: &str =
    "Universal digital asset acquisition with credential-aware provider metadata";
const CLI_LONG_ABOUT: &str = "DX Media searches and downloads media assets from \
provider-backed sources. Some providers are local or keyless, while others require \
credentials. Run `dx providers --format json` to inspect source_kind, \
requires_credentials, credential_status, and availability metadata.";

/// DX Media - Universal digital asset acquisition CLI.
#[derive(Debug, Parser)]
#[command(
    name = "dx",
    version,
    author,
    about = CLI_ABOUT,
    long_about = CLI_LONG_ABOUT
)]
pub struct Args {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Command,

    /// Output format.
    #[arg(short, long, global = true, default_value = "text")]
    pub format: OutputFormat,

    /// Enable verbose output.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors.
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

/// Available commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Search for media assets.
    #[command(alias = "s")]
    Search(SearchArgs),

    /// Download a media asset by ID.
    #[command(alias = "d")]
    Download(DownloadArgs),

    /// Scrape media from a website.
    #[command(alias = "sc")]
    Scrape(ScrapeArgs),

    /// List available providers.
    #[command(alias = "p")]
    Providers(ProvidersArgs),

    /// Show configuration information.
    Config,

    /// Check external dependencies (ffmpeg, etc).
    #[command(alias = "deps")]
    CheckDeps,

    /// Interactive mode (TUI).
    #[command(alias = "i")]
    Interactive,
}

/// Arguments for the search command.
#[derive(Debug, Parser)]
pub struct SearchArgs {
    /// Search query terms.
    #[arg(required = true)]
    pub query: Vec<String>,

    /// Media type filter.
    #[arg(short = 't', long, value_enum)]
    pub media_type: Option<MediaTypeArg>,

    /// Number of results to return.
    #[arg(short = 'n', long, default_value = "10")]
    pub count: usize,

    /// Page number for pagination.
    #[arg(short, long, default_value = "1")]
    pub page: usize,

    /// Specific providers to search (comma-separated).
    #[arg(short = 'P', long, value_delimiter = ',')]
    pub providers: Vec<String>,

    /// Image orientation filter.
    #[arg(long, value_enum)]
    pub orientation: Option<OrientationArg>,

    /// Filter by dominant color (e.g., "red", "blue", "orange").
    #[arg(long)]
    pub color: Option<String>,

    /// Automatically download the first result.
    #[arg(long)]
    pub download: bool,

    /// Output directory for downloads.
    #[arg(short, long)]
    pub output: Option<String>,

    /// Search ALL providers and scrapers concurrently for maximum results.
    #[arg(long)]
    pub all: bool,

    /// Search mode: quantity (fast, early-exit) or quality (wait for all providers).
    #[arg(short = 'm', long, value_enum, default_value = "quality")]
    pub mode: SearchModeArg,
}

impl SearchArgs {
    /// Get the search query as a single string.
    #[must_use]
    pub fn query_string(&self) -> String {
        self.query.join(" ")
    }
}

/// Arguments for the download command.
#[derive(Debug, Parser)]
pub struct DownloadArgs {
    /// Asset ID to download (format: provider:id).
    #[arg(required = true)]
    pub asset_id: String,

    /// Output directory.
    #[arg(short = 'd', long)]
    pub output: Option<String>,

    /// Custom filename (without -f since it conflicts with global --format).
    #[arg(long)]
    pub filename: Option<String>,
}

/// Arguments for the scrape command.
#[derive(Debug, Parser)]
pub struct ScrapeArgs {
    /// URL to scrape media from.
    #[arg(required = true)]
    pub url: String,

    /// Output directory for downloaded files.
    #[arg(short, long, default_value = ".")]
    pub output: String,

    /// Media type filter.
    #[arg(short = 't', long, value_enum, default_value = "image")]
    pub media_type: MediaTypeArg,

    /// Maximum number of assets to find.
    #[arg(short = 'n', long, default_value = "20")]
    pub count: usize,

    /// Depth of links to follow (0 = only the given URL).
    #[arg(short, long, default_value = "0")]
    pub depth: usize,

    /// File pattern to match (e.g., "*.jpg").
    #[arg(short = 'p', long)]
    pub pattern: Option<String>,

    /// Only show found media, don't download.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the providers command.
#[derive(Debug, Parser)]
pub struct ProvidersArgs {
    /// Show only available providers.
    #[arg(short, long)]
    pub available: bool,

    /// Show detailed information.
    #[arg(short, long)]
    pub detailed: bool,
}

/// Media type argument.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MediaTypeArg {
    /// Images (photos, illustrations).
    Image,
    /// Videos.
    Video,
    /// Audio files.
    Audio,
    /// Animated GIFs.
    Gif,
    /// Vector graphics (SVG).
    Vector,
    /// Documents (PDF, Word, etc).
    Document,
    /// 3D models (OBJ, FBX, GLTF).
    #[value(name = "3d")]
    Model3D,
    /// All types.
    All,
}

impl From<MediaTypeArg> for Option<crate::types::MediaType> {
    fn from(arg: MediaTypeArg) -> Self {
        match arg {
            MediaTypeArg::Image => Some(crate::types::MediaType::Image),
            MediaTypeArg::Video => Some(crate::types::MediaType::Video),
            MediaTypeArg::Audio => Some(crate::types::MediaType::Audio),
            MediaTypeArg::Gif => Some(crate::types::MediaType::Gif),
            MediaTypeArg::Vector => Some(crate::types::MediaType::Vector),
            MediaTypeArg::Document => Some(crate::types::MediaType::Document),
            MediaTypeArg::Model3D => Some(crate::types::MediaType::Model3D),
            MediaTypeArg::All => None,
        }
    }
}

/// Orientation filter argument.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OrientationArg {
    /// Landscape (wider than tall).
    Landscape,
    /// Portrait (taller than wide).
    Portrait,
    /// Square (equal dimensions).
    Square,
}

impl From<OrientationArg> for crate::types::Orientation {
    fn from(arg: OrientationArg) -> Self {
        match arg {
            OrientationArg::Landscape => crate::types::Orientation::Landscape,
            OrientationArg::Portrait => crate::types::Orientation::Portrait,
            OrientationArg::Square => crate::types::Orientation::Square,
        }
    }
}

/// Output format.
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output.
    #[default]
    Text,
    /// JSON output.
    Json,
    /// Compact JSON (single line).
    JsonCompact,
    /// Tab-separated values.
    Tsv,
}

/// Search mode argument.
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum SearchModeArg {
    /// Fast mode: Early exit after gathering enough results (3x count).
    /// Skips slow providers for speed. DEFAULT mode.
    #[default]
    Quantity,
    /// Thorough mode: Wait for ALL providers to respond (or timeout).
    /// Gets comprehensive results from every source.
    Quality,
}

impl From<SearchModeArg> for crate::types::SearchMode {
    fn from(arg: SearchModeArg) -> Self {
        match arg {
            SearchModeArg::Quantity => crate::types::SearchMode::Quantity,
            SearchModeArg::Quality => crate::types::SearchMode::Quality,
        }
    }
}

impl Args {
    /// Parse command-line arguments.
    #[must_use]
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_help_text_is_credential_honest() {
        let combined = format!("{CLI_ABOUT}\n{CLI_LONG_ABOUT}");

        assert!(!combined.contains("no API keys required"));
        assert!(!combined.contains("FREE APIs"));
        assert!(combined.contains("credential_status"));
        assert!(combined.contains("requires_credentials"));
        assert!(combined.contains("source_kind"));
    }

    #[test]
    fn test_search_query_string() {
        let args = SearchArgs {
            query: vec!["sunset".to_string(), "mountains".to_string()],
            media_type: None,
            count: 10,
            page: 1,
            providers: vec![],
            orientation: None,
            color: None,
            download: false,
            output: None,
            all: false,
            mode: SearchModeArg::Quantity,
        };

        assert_eq!(args.query_string(), "sunset mountains");
    }
}
