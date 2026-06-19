//! # DX Media
//!
//! **Universal Digital Asset Acquisition Engine**
//!
//! One command. Any media. From anywhere.
//!
//! DX Media provides a unified interface to search and download digital assets:
//! - provider-backed media sources with visible source metadata
//! - keyless and credential-gated providers with explicit availability metadata
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use dx_media::{DxMedia, MediaType};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let dx = DxMedia::new()?;
//!     
//!     // Search for images
//!     let results = dx.search("sunset")
//!         .media_type(MediaType::Image)
//!         .execute()
//!         .await?;
//!     println!("Found {} assets", results.total_count);
//!     
//!     // Download the first result
//!     if let Some(asset) = results.assets.first() {
//!         let path = dx.download(asset).await?;
//!         println!("Downloaded to: {:?}", path);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Universal Search**: One query syntax for all providers
//! - **Smart Downloads**: Async, parallel, with progress tracking
//! - **Rate Limiting**: Automatic throttling per provider
//! - **Dual Mode**: Use as CLI (`dx`) or Rust library
//! - **Graceful Degradation**: Credential-gated providers work when credentials are set
//! - **Source Honesty**: Provider metadata exposes source kind and credential status

// ═══════════════════════════════════════════════════════════════════════════════
// LINT CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════
//
// This crate uses clippy::pedantic for high code quality. Only truly necessary
// suppressions are applied at the crate level. All other issues are fixed at source.
//
// Crate-level suppressions are limited to:
// 1. Semantic choices (module naming, similar variable names)
// 2. Documentation style (technical terms without backticks)
// 3. Trait conformance (async trait methods that don't need async)
//
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// Semantic naming - these are intentional design choices
#![allow(clippy::module_name_repetitions)] // MediaAsset in media module is clear
#![allow(clippy::similar_names)]
// url/urls, item/items are meaningful

// Documentation style - technical terms are industry standard
#![allow(clippy::doc_markdown)] // FFmpeg, WebM don't need backticks
#![allow(clippy::missing_errors_doc)] // Errors documented via DxError enum

// Media processing requires numeric conversions
#![allow(clippy::cast_possible_truncation)] // Media dimensions, quality values
#![allow(clippy::cast_sign_loss)] // Unsigned media properties
#![allow(clippy::cast_precision_loss)] // Acceptable for progress percentages
#![allow(clippy::cast_lossless)] // Explicit casts for clarity

// Builder patterns and functional style
#![allow(clippy::missing_const_for_fn)] // Many functions can't be const due to side effects
#![allow(clippy::must_use_candidate)] // Not all builders need must_use
#![allow(clippy::return_self_not_must_use)] // Builder methods are self-documenting

// Code organization
#![allow(clippy::too_many_lines)] // Some functions handle complex media operations
#![allow(clippy::redundant_closure_for_method_calls)] // Explicit closures for clarity
#![allow(clippy::missing_panics_doc)] // Functions designed not to panic

// Trait conformance - Provider trait requires async
#![allow(clippy::unused_async)]
// Trait methods must be async

// Pedantic lints - legitimate cases
#![allow(clippy::unreadable_literal)] // Binary/hex literals in tests and data
#![allow(clippy::many_single_char_names)] // Math/algorithm code uses standard variable names
#![allow(clippy::uninlined_format_args)] // Readability preference

// ═══════════════════════════════════════════════════════════════════════════════
// MODULE DECLARATIONS
// ═══════════════════════════════════════════════════════════════════════════════

pub mod binary;
pub mod config;
pub mod constants;
pub mod core;
pub mod deps;
pub mod engine;
pub mod error;
pub mod http;
pub mod providers;
pub mod scraping;
pub mod tools;
pub mod types;

// CLI modules
#[cfg(feature = "cli")]
pub mod cli; // Original CLI
#[cfg(feature = "cli")]
#[allow(missing_docs)]
pub mod cli_unified; // Unified CLI for media/icon/font

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC RE-EXPORTS
// ═══════════════════════════════════════════════════════════════════════════════

pub use config::Config;
pub use engine::DxMedia;
pub use error::{DxError, Result};
pub use types::{
    DownloadUrlKind, HealthCheckResult, HealthReport, License, MediaAsset, MediaType,
    MimeEvidenceSource, SearchMode, SearchQuery, SearchResult,
};

// Re-export engine components
pub use engine::{
    CircuitBreaker, CircuitState, Downloader, FileManager, ScrapeOptions, ScrapeResult, Scraper,
    SearchEngine,
};

// Re-export primary provider types.
pub use providers::{
    ClevelandMuseumProvider,
    DplaProvider,
    EuropeanaProvider,
    LibraryOfCongressProvider,
    LoremPicsumProvider,
    MetMuseumProvider,
    NasaImagesProvider,
    // Tier 1: High-volume providers (700M+)
    OpenverseProvider,
    // Tier 3: 3D & Utility providers
    PolyHavenProvider,
    // Registry
    ProviderRegistry,
    // Tier 2: Museum providers
    RijksmuseumProvider,
    WikimediaCommonsProvider,
};

// Re-export credential-gated provider types.
// These gracefully degrade when credentials are not configured.
pub use providers::{
    FreesoundProvider,   // 600K+ sound effects
    GiphyProvider,       // Millions of GIFs
    PexelsProvider,      // 3.5M+ photos & videos
    PixabayProvider,     // 4.2M+ images, videos, music
    SmithsonianProvider, // 4.5M+ CC0 images
    UnsplashProvider,    // 5M+ high-quality photos
};

// Re-export tools module for media processing
pub use tools::{
    ArchiveTools, AudioTools, DocumentTools, ImageTools, ToolOutput, UtilityTools, VideoTools,
};

// Re-export core infrastructure (Binary Dawn architecture)
pub use core::{
    ConversionCache, CoreConfig, CoreError, CoreResult, MediaBuffer, MediaPipeline, PipelineStage,
    ProgressTracker,
};

// Re-export binary format utilities
pub use binary::{BinaryCache, BinaryCacheConfig, BinaryFormat, FormatDetector};

// Re-export HTTP security utilities
pub use http::{sanitize_filename, validate_url, verify_content_type};

// Re-export dependency checking utilities
pub use deps::{
    DEPENDENCIES, DependencyCheckResult, DependencyInfo, DependencyReport, check_all_dependencies,
    check_dependency, check_tool_dependency, find_dependency_for_tool,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// User agent for API requests - honest identification for responsible API usage
pub const USER_AGENT: &str = concat!(
    "dx-media/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/paiml/dx)"
);
