//! CLI interface for dx-font
//!
//! Provides command-line interface for searching and downloading fonts.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// dx-font - Access 50k+ commercial-free fonts
#[derive(Parser, Debug)]
#[command(name = "dx-font")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format (json, table, simple)
    #[arg(short, long, global = true, default_value = "table")]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for fonts
    Search {
        /// Search query
        query: String,

        /// Limit number of results
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Filter by provider (google, bunny, fontsource, fontshare)
        #[arg(short, long)]
        provider: Option<String>,

        /// Filter by category (serif, sans-serif, display, handwriting, monospace)
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Download a font
    Download {
        /// Font ID or name
        font_id: String,

        /// Provider to download from
        #[arg(short, long, default_value = "google")]
        provider: String,

        /// Output directory
        #[arg(short, long, default_value = "./fonts")]
        output: PathBuf,

        /// Font formats to download (ttf, woff, woff2, otf)
        #[arg(short = 'F', long, default_values = ["ttf", "woff2"])]
        formats: Vec<String>,

        /// Font subsets to download (latin, cyrillic, greek, etc.)
        #[arg(short = 'S', long, default_values = ["latin"])]
        subsets: Vec<String>,
    },

    /// List all available fonts
    List {
        /// Provider to list from (google, bunny, fontsource, fontshare, all)
        #[arg(short, long, default_value = "all")]
        provider: String,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Get detailed information about a font
    Info {
        /// Font ID
        font_id: String,

        /// Provider
        #[arg(short, long, default_value = "google")]
        provider: String,
    },

    /// Show font statistics
    Stats,

    /// Check provider health status
    Health,

    /// Show available providers and their font counts
    Providers,
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
