//! Provider trait definition.
//!
//! All media providers must implement this trait to be used with DX Media.

use async_trait::async_trait;

use crate::error::Result;
use crate::types::{MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Trait for media asset providers.
///
/// Each provider (Openverse, Wikimedia, NASA, etc.) implements this trait to provide
/// a unified interface for searching and downloading assets.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Returns the provider's unique identifier (lowercase).
    fn name(&self) -> &'static str;

    /// Returns the provider's display name.
    fn display_name(&self) -> &'static str;

    /// Returns the media types this provider supports.
    fn supported_media_types(&self) -> &[MediaType];

    /// Returns whether this provider requires an API key.
    fn requires_api_key(&self) -> bool;

    /// Returns the rate limit configuration for this provider.
    fn rate_limit(&self) -> RateLimitConfig;

    /// Check if the provider is available (has required API key, etc.).
    fn is_available(&self) -> bool;

    /// Search for assets matching the query.
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails.
    async fn search(&self, query: &SearchQuery) -> Result<SearchResult>;

    /// Get a specific asset by ID (optional - not all providers support this).
    ///
    /// Default implementation returns None. Providers that support direct ID lookup
    /// should override this method.
    ///
    /// # Errors
    ///
    /// Returns an error if the fetch fails.
    async fn get_by_id(&self, _id: &str) -> Result<Option<crate::types::MediaAsset>> {
        Ok(None)
    }

    /// Get the provider's base URL.
    fn base_url(&self) -> &'static str;
}

/// Extension trait for provider metadata.
pub trait ProviderInfo {
    /// Get provider description.
    fn description(&self) -> &'static str;

    /// Get the URL to obtain an API key.
    fn api_key_url(&self) -> &'static str;

    /// Get the license type for assets from this provider.
    fn default_license(&self) -> &'static str;
}
