//! Font provider implementations
//!
//! This module contains implementations for various font providers/sources.
//! Optimized for performance with concurrent fetching and connection pooling.

pub mod bunny_fonts;
pub mod dafont;
pub mod font_library;
pub mod fonts1001;
pub mod fontshare;
pub mod fontsource;
pub mod fontspace;
pub mod fontsquirrel;
pub mod github_fonts;
pub mod google_fonts;

use crate::error::{FontError, FontResult};
use crate::models::{
    Font, FontFamily, ProviderError, ProviderErrorType, SearchQuery, SearchResults,
};
use async_trait::async_trait;
use futures::future::join_all;
use std::sync::Arc;
use tokio::time::{Duration, Instant, timeout};
use tracing::{Instrument, debug_span, info_span};

/// Trait that all font providers must implement
#[async_trait]
pub trait FontProviderTrait: Send + Sync {
    /// Get the name of this provider
    fn name(&self) -> &str;

    /// Get the base URL for this provider
    fn base_url(&self) -> &str;

    /// Search for fonts matching the query
    async fn search(&self, query: &SearchQuery) -> FontResult<Vec<Font>>;

    /// Get all available fonts from this provider
    async fn list_all(&self) -> FontResult<Vec<Font>>;

    /// Get detailed information about a specific font family
    async fn get_font_family(&self, font_id: &str) -> FontResult<FontFamily>;

    /// Get download URL for a font
    async fn get_download_url(&self, font_id: &str) -> FontResult<String>;

    /// Check if the provider is available/responding
    async fn health_check(&self) -> FontResult<bool>;

    /// Get the cache key for this provider's font list
    fn cache_key(&self) -> String {
        format!(
            "provider_{}_fonts",
            self.name().to_lowercase().replace(' ', "_")
        )
    }
}

/// Create an HTTP client with optimized settings for performance
pub fn create_http_client() -> FontResult<reqwest::Client> {
    let client = reqwest::Client::builder()
        .user_agent(format!("dx-font/{}", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        // Connection pooling for better performance
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        // Enable compression for faster transfers
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .build()
        .map_err(|e| {
            FontError::provider("HTTP Client", format!("Failed to create client: {}", e))
        })?;
    Ok(client)
}

/// Create a fast HTTP client with shorter timeouts for health checks
pub fn create_fast_http_client() -> FontResult<reqwest::Client> {
    let client = reqwest::Client::builder()
        .user_agent(format!("dx-font/{}", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(3))
        .pool_max_idle_per_host(5)
        .build()
        .map_err(|e| {
            FontError::provider("HTTP Client", format!("Failed to create client: {}", e))
        })?;
    Ok(client)
}

/// Registry of all available font providers
pub struct ProviderRegistry {
    providers: Vec<Arc<dyn FontProviderTrait>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn with_defaults() -> FontResult<Self> {
        let client = create_http_client()?;
        let mut registry = Self::new();

        // Add all providers for maximum font coverage (50k+ fonts)
        registry.register(Arc::new(google_fonts::GoogleFontsProvider::new(
            client.clone(),
        )));
        registry.register(Arc::new(bunny_fonts::BunnyFontsProvider::new(
            client.clone(),
        )));
        registry.register(Arc::new(fontsource::FontsourceProvider::new(
            client.clone(),
        )));
        registry.register(Arc::new(fontshare::FontshareProvider::new(client.clone())));
        registry.register(Arc::new(font_library::FontLibraryProvider::new(
            client.clone(),
        )));
        registry.register(Arc::new(github_fonts::GitHubFontsProvider::new(
            client.clone(),
        )));
        registry.register(Arc::new(dafont::DafontProvider::new(client.clone())));
        registry.register(Arc::new(fontspace::FontSpaceProvider::new(client.clone())));
        registry.register(Arc::new(fonts1001::Fonts1001Provider::new(client.clone())));
        registry.register(Arc::new(fontsquirrel::FontSquirrelProvider::new(
            client.clone(),
        )));

        Ok(registry)
    }

    pub fn register(&mut self, provider: Arc<dyn FontProviderTrait>) {
        self.providers.push(provider);
    }

    pub fn providers(&self) -> &[Arc<dyn FontProviderTrait>] {
        &self.providers
    }

    /// Search all providers concurrently for maximum speed
    pub async fn search_all(&self, query: &SearchQuery) -> FontResult<SearchResults> {
        let start = Instant::now();
        let providers_searched: Vec<String> = self
            .providers
            .iter()
            .map(|p| p.name().to_string())
            .collect();

        // Create search futures for all providers
        let search_futures: Vec<_> = self
            .providers
            .iter()
            .map(|provider| {
                let provider = Arc::clone(provider);
                let query = query.clone();
                async move {
                    let provider_name = provider.name().to_string();
                    // Add timeout for each provider to prevent slow providers from blocking
                    let search_result = timeout(Duration::from_secs(15), provider.search(&query))
                        .instrument(info_span!("provider_search", provider = %provider_name, query = %query.query))
                        .await;

                    match search_result {
                        Ok(Ok(fonts)) => {
                            tracing::debug!(
                                "Provider {} returned {} fonts",
                                provider_name,
                                fonts.len()
                            );
                            (fonts, None)
                        }
                        Ok(Err(e)) => {
                            tracing::warn!("Error searching {}: {}", provider_name, e);
                            let error_type = match &e {
                                FontError::Network { .. } => ProviderErrorType::Network,
                                FontError::RateLimit { .. } => ProviderErrorType::RateLimit,
                                FontError::Parse { .. } => ProviderErrorType::Parse,
                                FontError::Timeout { .. } => ProviderErrorType::Timeout,
                                _ => ProviderErrorType::NotAvailable,
                            };
                            (Vec::new(), Some(ProviderError {
                                provider: provider_name,
                                error_type,
                                message: e.to_string(),
                            }))
                        }
                        Err(_) => {
                            tracing::warn!("Timeout searching {}", provider_name);
                            (Vec::new(), Some(ProviderError {
                                provider: provider_name,
                                error_type: ProviderErrorType::Timeout,
                                message: "Request timed out".to_string(),
                            }))
                        }
                    }
                }
            })
            .collect();

        // Execute all searches concurrently
        let results = join_all(search_futures).await;

        // Separate fonts and errors
        let mut all_fonts: Vec<Font> = Vec::new();
        let mut provider_errors: Vec<ProviderError> = Vec::new();

        for (fonts, error) in results {
            all_fonts.extend(fonts);
            if let Some(err) = error {
                provider_errors.push(err);
            }
        }

        let total = all_fonts.len();

        // If ALL providers failed, return an error
        if total == 0 && provider_errors.len() == self.providers.len() {
            let errors: Vec<(String, FontError)> = provider_errors
                .iter()
                .map(|e| {
                    (
                        e.provider.clone(),
                        FontError::provider(&e.provider, &e.message),
                    )
                })
                .collect();
            return Err(FontError::all_providers_failed(errors));
        }

        // Apply limit if specified
        if let Some(limit) = query.limit {
            all_fonts.truncate(limit);
        }

        let elapsed = start.elapsed();
        tracing::info!("Search completed in {:?}, found {} fonts", elapsed, total);

        Ok(SearchResults {
            fonts: all_fonts,
            total,
            query: query.query.clone(),
            providers_searched,
            provider_errors,
            from_cache: false,
        })
    }

    /// List all fonts from all providers concurrently
    pub async fn list_all_concurrent(&self) -> FontResult<SearchResults> {
        let start = Instant::now();
        let providers_searched: Vec<String> = self
            .providers
            .iter()
            .map(|p| p.name().to_string())
            .collect();

        // Create list futures for all providers
        let list_futures: Vec<_> = self
            .providers
            .iter()
            .map(|provider| {
                let provider = Arc::clone(provider);
                async move {
                    let provider_name = provider.name().to_string();
                    let list_result = timeout(Duration::from_secs(30), provider.list_all())
                        .instrument(info_span!("provider_list", provider = %provider_name))
                        .await;

                    match list_result {
                        Ok(Ok(fonts)) => {
                            tracing::debug!(
                                "Provider {} listed {} fonts",
                                provider_name,
                                fonts.len()
                            );
                            (fonts, None)
                        }
                        Ok(Err(e)) => {
                            tracing::warn!("Error listing from {}: {}", provider_name, e);
                            let error_type = match &e {
                                FontError::Network { .. } => ProviderErrorType::Network,
                                FontError::RateLimit { .. } => ProviderErrorType::RateLimit,
                                FontError::Parse { .. } => ProviderErrorType::Parse,
                                FontError::Timeout { .. } => ProviderErrorType::Timeout,
                                _ => ProviderErrorType::NotAvailable,
                            };
                            (
                                Vec::new(),
                                Some(ProviderError {
                                    provider: provider_name,
                                    error_type,
                                    message: e.to_string(),
                                }),
                            )
                        }
                        Err(_) => {
                            tracing::warn!("Timeout listing from {}", provider_name);
                            (
                                Vec::new(),
                                Some(ProviderError {
                                    provider: provider_name,
                                    error_type: ProviderErrorType::Timeout,
                                    message: "Request timed out".to_string(),
                                }),
                            )
                        }
                    }
                }
            })
            .collect();

        // Execute all lists concurrently
        let results = join_all(list_futures).await;

        // Separate fonts and errors
        let mut all_fonts: Vec<Font> = Vec::new();
        let mut provider_errors: Vec<ProviderError> = Vec::new();

        for (fonts, error) in results {
            all_fonts.extend(fonts);
            if let Some(err) = error {
                provider_errors.push(err);
            }
        }

        let total = all_fonts.len();

        // If ALL providers failed, return an error
        if total == 0 && provider_errors.len() == self.providers.len() {
            let errors: Vec<(String, FontError)> = provider_errors
                .iter()
                .map(|e| {
                    (
                        e.provider.clone(),
                        FontError::provider(&e.provider, &e.message),
                    )
                })
                .collect();
            return Err(FontError::all_providers_failed(errors));
        }

        let elapsed = start.elapsed();
        tracing::info!("List all completed in {:?}, found {} fonts", elapsed, total);

        Ok(SearchResults {
            fonts: all_fonts,
            total,
            query: String::new(),
            providers_searched,
            provider_errors,
            from_cache: false,
        })
    }

    /// Check health of all providers concurrently
    pub async fn health_check_all(&self) -> Vec<(String, bool, Duration)> {
        let health_futures: Vec<_> = self
            .providers
            .iter()
            .map(|provider| {
                let provider = Arc::clone(provider);
                async move {
                    let provider_name = provider.name().to_string();
                    let start = Instant::now();
                    let health_result = timeout(Duration::from_secs(5), provider.health_check())
                        .instrument(debug_span!("provider_health_check", provider = %provider_name))
                        .await;
                    let is_healthy = match health_result {
                        Ok(Ok(healthy)) => healthy,
                        _ => false,
                    };
                    let elapsed = start.elapsed();
                    (provider_name, is_healthy, elapsed)
                }
            })
            .collect();

        join_all(health_futures).await
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
