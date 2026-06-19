//! dx-font - A comprehensive font search and download library
//!
//! This crate provides a production-ready solution for searching and downloading fonts
//! from multiple providers. Access 50k+ commercial-free fonts from 100+ sources including:
//!
//! - Google Fonts (1,562 fonts)
//! - Bunny Fonts (1,478 fonts)
//! - Fontsource (1,562 fonts)
//! - Font Squirrel (1,082 fonts)
//! - DaFont, FontSpace, and many more!
//!
//! ## Features
//!
//! - **Parallel Search**: Blazing fast concurrent search across all providers
//! - **Progress Indication**: Real-time download progress with ETA
//! - **CDN URLs**: Generate CDN URLs for font preview and usage
//! - **Multiple Formats**: Support for TTF, OTF, WOFF, WOFF2
//! - **Caching**: Built-in response caching with configurable TTL
//! - **Rate Limiting**: Automatic rate limiting to prevent API abuse
//! - **Retry Logic**: Exponential backoff for transient failures
//! - **File Verification**: Magic byte and archive validation
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use dx_font::{FontSearch, FontDownloader, FontProvider};
//! use dx_font::models::DownloadOptions;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> dx_font::FontResult<()> {
//!     // Search for fonts
//!     let search = FontSearch::new()?;
//!     let results = search.search("roboto").await?;
//!     
//!     println!("Found {} fonts", results.total);
//!     for font in results.fonts.iter().take(5) {
//!         println!("  - {} ({})", font.name, font.provider.name());
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! All operations return [`FontResult<T>`], which is an alias for `Result<T, FontError>`.
//! The [`FontError`] enum provides detailed error information:
//!
//! ```rust,no_run
//! use dx_font::{FontSearch, FontError};
//!
//! #[tokio::main]
//! async fn main() {
//!     let search = FontSearch::new().unwrap();
//!     
//!     match search.search("roboto").await {
//!         Ok(results) => println!("Found {} fonts", results.total),
//!         Err(FontError::Network { url, .. }) => {
//!             eprintln!("Network error accessing {}", url);
//!         }
//!         Err(FontError::AllProvidersFailed { errors }) => {
//!             eprintln!("All providers failed:");
//!             for (provider, error) in errors {
//!                 eprintln!("  - {}: {}", provider, error);
//!             }
//!         }
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```
//!
//! ## Configuration
//!
//! Use [`ConfigBuilder`] to customize behavior:
//!
//! ```rust,no_run
//! use dx_font::Config;
//! use std::path::PathBuf;
//!
//! let config = Config::builder()
//!     .output_dir(PathBuf::from("./fonts"))
//!     .timeout_seconds(60)
//!     .max_retries(5)
//!     .build()
//!     .unwrap();
//! ```

/// Cache management for API responses and font metadata.
pub mod cache;

/// ZIP extraction utilities
pub mod extract;

/// Figlet fonts for ASCII art text rendering in CLI applications.
pub mod figlet;

/// CDN URL generation for font preview and usage.
pub mod cdn;

/// Command-line interface implementation.
pub mod cli;

/// Configuration and validation.
pub mod config;

/// Font download functionality with progress indication.
pub mod download;

/// Error types and result aliases.
pub mod error;

/// HTTP client with retry logic and rate limiting.
pub mod http;

/// Data models for fonts, providers, and search results.
pub mod models;

/// Prelude module for convenient imports.
pub mod prelude;

/// Font provider implementations.
pub mod providers;

/// Rate limiting for API requests.
pub mod rate_limit;

/// Font search functionality.
pub mod search;

/// File verification for downloaded fonts.
pub mod verify;

pub use cache::CacheManager;
pub use cdn::{CdnProvider, CdnUrlGenerator, FontCdnUrls};
pub use config::{Config, ConfigBuilder};
pub use download::FontDownloader;
pub use error::{FontError, FontResult};
pub use http::RetryClient;
pub use models::{Font, FontFamily, FontProvider, FontStyle, FontWeight};
pub use rate_limit::RateLimiter;
pub use search::FontSearch;
pub use verify::FileVerifier;
