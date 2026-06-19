//! CLI argument definitions for unified media tool

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "media")]
#[command(author, version = "1.0.0", about = "Universal media, icon, and font CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format (json, table, simple)
    #[arg(short = 'f', long, global = true, default_value = "table")]
    pub format: OutputFormat,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for media assets (images, videos, audio)
    #[command(alias = "s")]
    Search {
        /// Search query
        query: String,

        /// Media type (image, video, audio, all)
        #[arg(short, long, default_value = "all")]
        media_type: String,

        /// Provider to search (openverse, unsplash, pexels, pixabay, etc.)
        #[arg(short, long)]
        provider: Option<String>,

        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Download media asset
    #[command(alias = "d")]
    Download {
        /// Asset ID or URL
        asset_id: String,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Provider (if using asset ID)
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// Icon operations
    Icon {
        #[command(subcommand)]
        command: IconCommands,
    },

    /// Font operations
    Font {
        #[command(subcommand)]
        command: FontCommands,
    },

    /// Media processing tools
    Tools {
        #[command(subcommand)]
        command: ToolCommands,
    },

    /// Video tools (all 11 tools)
    Video {
        #[command(subcommand)]
        command: crate::cli_unified::args_extended::VideoToolsExtended,
    },

    /// Audio tools (all 9 tools)
    Audio {
        #[command(subcommand)]
        command: crate::cli_unified::args_extended::AudioToolsExtended,
    },

    /// Image tools (all 10 tools)
    Image {
        #[command(subcommand)]
        command: crate::cli_unified::args_extended::ImageToolsExtended,
    },

    /// Archive tools (all 7 tools)
    Archive {
        #[command(subcommand)]
        command: crate::cli_unified::args_extended::ArchiveToolsExtended,
    },

    /// Document tools (all 9 tools)
    Document {
        #[command(subcommand)]
        command: crate::cli_unified::args_extended::DocumentToolsExtended,
    },

    /// Utility tools (all 14 tools)
    Utility {
        #[command(subcommand)]
        command: crate::cli_unified::args_extended::UtilityToolsExtended,
    },

    /// List available providers
    Providers {
        /// Filter by type (media, icon, font, all)
        #[arg(short = 't', long, default_value = "all")]
        provider_type: String,
    },

    /// Health check for all providers
    Health,
}

#[derive(Subcommand, Debug)]
pub enum IconCommands {
    /// Search for icons
    #[command(alias = "s")]
    Search {
        /// Search query
        query: String,

        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Filter by icon pack
        #[arg(short, long)]
        pack: Option<String>,
    },

    /// Export icons as SVG files
    #[command(alias = "e")]
    Export {
        /// Search query
        query: String,

        /// Output directory
        #[arg(short, long, default_value = "./icons")]
        output: PathBuf,

        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Filter by icon pack
        #[arg(short, long)]
        pack: Option<String>,
    },

    /// Export to desktop app
    #[command(alias = "d")]
    Desktop {
        /// Icon specifications (name:pack format)
        #[arg(required = true)]
        icons: Vec<String>,
    },

    /// List available icon packs
    #[command(alias = "p")]
    Packs,
}

#[derive(Subcommand, Debug)]
pub enum FontCommands {
    /// Search for fonts
    #[command(alias = "s")]
    Search {
        /// Search query
        query: String,

        /// Limit number of results
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Filter by provider
        #[arg(short, long)]
        provider: Option<String>,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Download a font
    #[command(alias = "d")]
    Download {
        /// Font ID or name
        font_id: String,

        /// Provider
        #[arg(short, long, default_value = "google")]
        provider: String,

        /// Output directory
        #[arg(short, long, default_value = "./fonts")]
        output: PathBuf,

        /// Font formats (ttf, woff, woff2, otf)
        #[arg(short = 'F', long, default_values = ["ttf", "woff2"])]
        formats: Vec<String>,

        /// Font subsets (latin, cyrillic, greek, etc.)
        #[arg(short = 'S', long, default_values = ["latin"])]
        subsets: Vec<String>,
    },

    /// List all available fonts
    #[command(alias = "l")]
    List {
        /// Provider to list from
        #[arg(short, long, default_value = "all")]
        provider: String,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Get detailed font information
    #[command(alias = "i")]
    Info {
        /// Font ID
        font_id: String,

        /// Provider
        #[arg(short, long, default_value = "google")]
        provider: String,
    },

    /// Show font statistics
    Stats,

    /// List available font providers
    Providers,
}

#[derive(Subcommand, Debug)]
pub enum ToolCommands {
    /// List declared tools with source, dependency, and readiness metadata
    List {
        /// Filter by category (media, image, video, audio, document, archive, utility)
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Image processing tools
    Image {
        #[command(subcommand)]
        command: ImageToolCommands,
    },

    /// Video processing tools
    Video {
        #[command(subcommand)]
        command: VideoToolCommands,
    },

    /// Audio processing tools
    Audio {
        #[command(subcommand)]
        command: AudioToolCommands,
    },

    /// Archive tools
    Archive {
        #[command(subcommand)]
        command: ArchiveToolCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ImageToolCommands {
    /// Convert image format
    Convert {
        /// Input file
        input: PathBuf,

        /// Output file
        output: PathBuf,

        /// Quality (1-100 for JPEG)
        #[arg(short, long)]
        quality: Option<u8>,
    },

    /// Resize image
    Resize {
        /// Input file
        input: PathBuf,

        /// Output file
        output: PathBuf,

        /// Width
        #[arg(short, long)]
        width: Option<u32>,

        /// Height
        #[arg(short = 'H', long)]
        height: Option<u32>,
    },

    /// Generate favicons from SVG
    Favicon {
        /// Input SVG file
        input: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "./icons")]
        output: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum VideoToolCommands {
    /// Convert video format
    Convert {
        /// Input file
        input: PathBuf,

        /// Output file
        output: PathBuf,
    },

    /// Extract audio from video
    ExtractAudio {
        /// Input video file
        input: PathBuf,

        /// Output audio file
        output: PathBuf,
    },

    /// Create GIF from video
    ToGif {
        /// Input video file
        input: PathBuf,

        /// Output GIF file
        output: PathBuf,

        /// FPS
        #[arg(short, long, default_value = "10")]
        fps: u32,
    },
}

#[derive(Subcommand, Debug)]
pub enum AudioToolCommands {
    /// Convert audio format
    Convert {
        /// Input file
        input: PathBuf,

        /// Output file
        output: PathBuf,
    },

    /// Trim audio
    Trim {
        /// Input file
        input: PathBuf,

        /// Output file
        output: PathBuf,

        /// Start time (seconds)
        #[arg(short, long)]
        start: f64,

        /// Duration (seconds)
        #[arg(short, long)]
        duration: f64,
    },
}

#[derive(Subcommand, Debug)]
pub enum ArchiveToolCommands {
    /// Create ZIP archive
    Zip {
        /// Files to archive
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output archive
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Extract archive
    Extract {
        /// Archive file
        input: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Simple,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Simple => write!(f, "simple"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands, OutputFormat};
    use crate::cli_unified::args_extended::UtilityToolsExtended;
    use clap::Parser;

    #[test]
    fn utility_convert_csv_uses_distinct_csv_format_arg() {
        let cli = Cli::try_parse_from([
            "media",
            "--format",
            "json",
            "utility",
            "convert-csv",
            "input.csv",
            "output.json",
            "--csv-format",
            "yaml",
        ])
        .expect("utility convert-csv should parse without colliding with global format");

        assert!(matches!(cli.format, OutputFormat::Json));

        match cli.command {
            Commands::Utility {
                command:
                    UtilityToolsExtended::ConvertCsv {
                        input,
                        output,
                        csv_format,
                    },
            } => {
                assert_eq!(input, std::path::PathBuf::from("input.csv"));
                assert_eq!(output, std::path::PathBuf::from("output.json"));
                assert_eq!(csv_format, "yaml");
            }
            other => panic!("expected utility convert-csv command, got {other:?}"),
        }
    }
}
