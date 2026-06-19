//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from dx-font,
//! allowing you to import everything you need with a single use statement.
//!
//! # Example
//!
//! ```rust
//! use dx_font::prelude::*;
//!
//! // Now you have access to all common types:
//! // - FontSearch, FontDownloader
//! // - Font, FontFamily, FontProvider
//! // - FontError, FontResult
//! // - Config, ConfigBuilder
//! // - And more...
//! ```

// Core search and download functionality
pub use crate::download::FontDownloader;
pub use crate::search::FontSearch;

// Error handling
pub use crate::error::{FontError, FontResult};

// Configuration
pub use crate::config::{Config, ConfigBuilder};

// Core models
pub use crate::models::{
    DownloadOptions, Font, FontCategory, FontFamily, FontLicense, FontProvider, FontStyle,
    FontVariant, FontWeight, ProviderError, ProviderErrorType, SearchQuery, SearchResults,
};

// CDN functionality
pub use crate::cdn::{CdnProvider, CdnUrlGenerator, FontCdnUrls};

// Infrastructure (for advanced users)
pub use crate::cache::CacheManager;
pub use crate::http::RetryClient;
pub use crate::rate_limit::RateLimiter;
pub use crate::verify::FileVerifier;
