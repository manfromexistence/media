//! DxMedia - Main facade for the media acquisition library.
//!
//! This is the primary entry point for using dx_media as a library.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::Config;
use crate::engine::{Downloader, FileManager, ScrapeOptions, Scraper, SearchEngine};
use crate::error::Result;
use crate::providers::ProviderRegistry;
use crate::tools::ToolOutput;
use crate::types::{MediaAsset, MediaType, SearchQuery, SearchResult};

/// Main facade for the DX Media library.
///
/// # Example
///
/// ```no_run
/// use dx_media::{DxMedia, MediaType};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let dx = DxMedia::new()?;
///     
///     // Search for images
///     let results = dx.search("sunset mountains")
///         .media_type(MediaType::Image)
///         .count(10)
///         .execute()
///         .await?;
///
///     // Download the first result
///     if let Some(asset) = results.assets.first() {
///         let path = dx.download(asset).await?;
///         println!("Downloaded to: {}", path.display());
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct DxMedia {
    config: Config,
    registry: Arc<ProviderRegistry>,
    search_engine: SearchEngine,
    downloader: Downloader,
    file_manager: FileManager,
}

impl DxMedia {
    /// Create a new DxMedia instance with default configuration.
    ///
    /// Loads configuration from environment variables and .env file.
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Self::with_config(config)
    }

    /// Create a new DxMedia instance with the given configuration.
    pub fn with_config(config: Config) -> Result<Self> {
        let registry = Arc::new(ProviderRegistry::new(&config));
        let search_engine = SearchEngine::new(Arc::clone(&registry));
        let downloader = Downloader::new(&config);
        let file_manager = FileManager::new(&config.download_dir);

        Ok(Self {
            config,
            registry,
            search_engine,
            downloader,
            file_manager,
        })
    }

    /// Create a search query builder.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dx_media::{DxMedia, MediaType};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dx = DxMedia::new()?;
    /// let results = dx.search("nature")
    ///     .media_type(MediaType::Image)
    ///     .count(20)
    ///     .provider("openverse")
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn search(&self, query: impl Into<String>) -> SearchBuilder<'_> {
        SearchBuilder {
            dx: self,
            query: SearchQuery::new(query),
        }
    }

    /// Execute a search query directly.
    pub async fn search_query(&self, query: &SearchQuery) -> Result<SearchResult> {
        self.search_engine.search(query).await
    }

    /// Download a media asset to the default download directory.
    pub async fn download(&self, asset: &MediaAsset) -> Result<PathBuf> {
        self.downloader.download(asset).await
    }

    /// Download a media asset to the default download directory with a provenance receipt.
    pub async fn download_with_receipt(&self, asset: &MediaAsset) -> Result<ToolOutput> {
        self.downloader.download_with_receipt(asset).await
    }

    /// Download a media asset to a specific directory.
    pub async fn download_to(&self, asset: &MediaAsset, dir: &Path) -> Result<PathBuf> {
        self.downloader.download_to(dir, asset).await
    }

    /// Download a media asset to a specific directory with a provenance receipt.
    pub async fn download_to_with_receipt(
        &self,
        asset: &MediaAsset,
        dir: &Path,
    ) -> Result<ToolOutput> {
        self.downloader.download_to_with_receipt(dir, asset).await
    }

    /// Get the provider registry.
    #[must_use]
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    /// Get available provider names.
    #[must_use]
    pub fn available_providers(&self) -> Vec<String> {
        self.registry.available_provider_names()
    }

    /// Get all provider names (including unavailable).
    #[must_use]
    pub fn all_providers(&self) -> Vec<String> {
        self.registry.provider_names()
    }

    /// Check if a specific provider is available.
    #[must_use]
    pub fn is_provider_available(&self, name: &str) -> bool {
        self.registry.get(name).map_or(false, |p| p.is_available())
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the file manager.
    #[must_use]
    pub fn file_manager(&self) -> &FileManager {
        &self.file_manager
    }

    /// Get the downloader.
    #[must_use]
    pub fn downloader(&self) -> &Downloader {
        &self.downloader
    }

    /// Get the download directory.
    #[must_use]
    pub fn download_dir(&self) -> &Path {
        self.downloader.download_dir()
    }

    /// Perform a health check on all providers.
    ///
    /// Tests connectivity to each provider with a timeout and returns
    /// a comprehensive health report including latency and circuit breaker state.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dx_media::DxMedia;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dx = DxMedia::new()?;
    /// let report = dx.health_check().await;
    /// println!("Healthy: {}/{}", report.healthy_count, report.total_providers);
    /// for provider in report.unhealthy_providers() {
    ///     println!("  {} - {}", provider.provider, provider.error.as_deref().unwrap_or("unknown"));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn health_check(&self) -> crate::types::HealthReport {
        use crate::engine::CircuitState;
        use crate::types::{HealthCheckResult, HealthReport, SearchQuery};
        use futures::stream::{FuturesUnordered, StreamExt};
        use std::time::{Duration, Instant};

        let start = Instant::now();
        let timeout = Duration::from_secs(5);

        // Get all registered providers
        let providers = self.registry.all();

        // Create health check futures for each provider
        let mut futures: FuturesUnordered<_> = providers
            .iter()
            .map(|provider| {
                let provider = std::sync::Arc::clone(provider);
                let registry = std::sync::Arc::clone(&self.registry);
                async move {
                    let name = provider.name().to_string();
                    let provider_start = Instant::now();

                    // Get circuit breaker state
                    let circuit_state = registry
                        .get_circuit_breaker(&name)
                        .map(|cb| match cb.state() {
                            CircuitState::Closed => "closed",
                            CircuitState::Open => "open",
                            CircuitState::HalfOpen => "half-open",
                        })
                        .unwrap_or("unknown");

                    // Check if provider is available (has API key, etc.)
                    if !provider.is_available() {
                        return HealthCheckResult::failure(
                            &name,
                            "Provider not available (missing API key or disabled)",
                            circuit_state,
                        );
                    }

                    // Try a minimal search to test connectivity
                    let test_query = SearchQuery::new("test").count(1);
                    let result = tokio::time::timeout(timeout, provider.search(&test_query)).await;

                    let latency = provider_start.elapsed().as_millis() as u64;

                    match result {
                        Ok(Ok(_)) => HealthCheckResult::success(&name, latency, circuit_state),
                        Ok(Err(e)) => {
                            HealthCheckResult::failure(&name, e.to_string(), circuit_state)
                        }
                        Err(_) => HealthCheckResult::failure(
                            &name,
                            format!("Timeout after {}ms", timeout.as_millis()),
                            circuit_state,
                        ),
                    }
                }
            })
            .collect();

        // Collect all results
        let mut results = Vec::new();
        while let Some(result) = futures.next().await {
            results.push(result);
        }

        // Sort by provider name for consistent output
        results.sort_by(|a, b| a.provider.cmp(&b.provider));

        HealthReport::new(results, start.elapsed().as_millis() as u64)
    }

    /// Search all providers AND scrapers concurrently.
    ///
    /// This is the main unified search function that:
    /// 1. Searches all available API providers concurrently
    /// 2. Scrapes configured free image sites concurrently
    /// 3. Returns combined results from all sources
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dx_media::DxMedia;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dx = DxMedia::new()?;
    /// let results = dx.search_all("sunset mountains", 10).await?;
    /// println!("Found {} total results from {} sources",
    ///     results.assets.len(),
    ///     results.providers_searched.len()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_all(&self, query: &str, count_per_source: usize) -> Result<SearchResult> {
        use crate::types::SearchMode;
        self.search_all_with_mode(query, count_per_source, SearchMode::default())
            .await
    }

    /// Search all providers AND scrapers concurrently with explicit search mode.
    ///
    /// # Search Modes
    /// - **Quantity** (default): Fast early-exit after 3x results. Skips slow sources.
    /// - **Quality**: Waits for ALL providers/scrapers to respond (or timeout).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dx_media::{DxMedia, SearchMode};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let dx = DxMedia::new()?;
    /// // Quality mode - wait for all providers
    /// let results = dx.search_all_with_mode("sunset", 10, SearchMode::Quality).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_all_with_mode(
        &self,
        query: &str,
        count_per_source: usize,
        mode: crate::types::SearchMode,
    ) -> Result<SearchResult> {
        use crate::types::SearchMode;
        use futures::stream::{FuturesUnordered, StreamExt};
        use std::time::{Duration, Instant};

        let start = Instant::now();

        // Timeout varies by mode: Quantity=fast, Quality=patient
        let scraper_timeout = match mode {
            SearchMode::Quantity => Duration::from_secs(3),
            SearchMode::Quality => Duration::from_secs(6),
        };

        // Build search query for providers with mode
        let search_query = SearchQuery::new(query).count(count_per_source).mode(mode);

        // Create scraper for web scraping
        let scraper = Scraper::new()?;

        // Define scraping targets (sites that work without Cloudflare protection)
        let scrape_urls = self.get_scrape_search_urls(query);

        // Create FuturesUnordered for scraper searches with timeouts
        let mut scrape_futures: FuturesUnordered<_> = scrape_urls
            .into_iter()
            .map(|(name, url)| {
                let scraper = scraper.clone();
                let options = ScrapeOptions {
                    max_depth: 0,
                    pattern: None,
                    media_types: vec![MediaType::Image],
                    max_assets: count_per_source,
                };
                async move {
                    let result =
                        tokio::time::timeout(scraper_timeout, scraper.scrape(&url, &options)).await;

                    match result {
                        Ok(r) => (name, r),
                        Err(_) => (
                            name.clone(),
                            Err(crate::error::DxError::ProviderApi {
                                provider: format!("scraper:{}", name),
                                message: "Scraper timed out (>5s)".to_string(),
                                status_code: 408,
                            }),
                        ),
                    }
                }
            })
            .collect();

        // Execute provider search and scraper searches concurrently
        let provider_future = self.search_engine.search(&search_query);

        // Start collecting scraper results in parallel
        let scraper_collector = async {
            let mut results = Vec::new();
            while let Some(result) = scrape_futures.next().await {
                results.push(result);
            }
            results
        };

        // Run both in parallel
        let (provider_result, scrape_results) = tokio::join!(provider_future, scraper_collector);

        // Start with provider results
        let mut result = provider_result.unwrap_or_else(|_| SearchResult::new(query));

        // Add scraper results
        for (name, scrape_result) in scrape_results {
            match scrape_result {
                Ok(sr) => {
                    result.providers_searched.push(format!("scraper:{}", name));
                    result.total_count += sr.assets.len();
                    result.assets.extend(sr.assets);
                }
                Err(e) => {
                    result
                        .provider_errors
                        .push((format!("scraper:{}", name), e.to_string()));
                }
            }
        }

        result.duration_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Get search URLs for scraping targets.
    ///
    /// Returns a list of (name, url) pairs for sites that support search
    /// and don't have Cloudflare protection.
    fn get_scrape_search_urls(&self, _query: &str) -> Vec<(String, String)> {
        // Only include sites that:
        // 1. Support search via URL parameter
        // 2. Don't have Cloudflare/bot protection
        // 3. Return useful results without JavaScript
        vec![
            // Flickr explore (no search, but popular images)
            (
                "flickr".to_string(),
                "https://www.flickr.com/explore".to_string(),
            ),
            // NASA image gallery
            (
                "nasa-gallery".to_string(),
                "https://www.nasa.gov/multimedia/imagegallery/".to_string(),
            ),
        ]
    }
}

/// Builder for constructing and executing searches.
pub struct SearchBuilder<'a> {
    dx: &'a DxMedia,
    query: SearchQuery,
}

impl<'a> SearchBuilder<'a> {
    /// Filter by media type.
    #[must_use]
    pub fn media_type(mut self, media_type: MediaType) -> Self {
        self.query.media_type = Some(media_type);
        self
    }

    /// Set the number of results per provider.
    #[must_use]
    pub fn count(mut self, count: usize) -> Self {
        self.query.count = count;
        self
    }

    /// Set the page number for pagination.
    #[must_use]
    pub fn page(mut self, page: usize) -> Self {
        self.query.page = page;
        self
    }

    /// Limit search to specific providers.
    #[must_use]
    pub fn providers(mut self, providers: Vec<String>) -> Self {
        self.query.providers = providers;
        self
    }

    /// Add a single provider to search.
    #[must_use]
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.query.providers.push(provider.into());
        self
    }

    /// Execute the search.
    pub async fn execute(self) -> Result<SearchResult> {
        self.dx.search_query(&self.query).await
    }

    /// Get the built query without executing.
    #[must_use]
    pub fn build(self) -> SearchQuery {
        self.query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_builder() {
        let config = Config::default();
        let dx = DxMedia::with_config(config).unwrap();

        let query = dx
            .search("test query")
            .media_type(MediaType::Image)
            .count(25)
            .page(2)
            .provider("openverse")
            .provider("wikimedia")
            .build();

        assert_eq!(query.query, "test query");
        assert_eq!(query.media_type, Some(MediaType::Image));
        assert_eq!(query.count, 25);
        assert_eq!(query.page, 2);
        assert!(query.providers.contains(&"openverse".to_string()));
        assert!(query.providers.contains(&"wikimedia".to_string()));
    }

    #[test]
    fn test_provider_listing() {
        let config = Config::default();
        let dx = DxMedia::with_config(config).unwrap();

        let all = dx.all_providers();
        // Check representative providers.
        assert!(all.contains(&"openverse".to_string()));
        assert!(all.contains(&"wikimedia".to_string()));
        assert!(all.contains(&"nasa".to_string()));
        assert!(all.contains(&"met".to_string()));
        assert!(all.contains(&"picsum".to_string()));
        assert!(all.contains(&"cleveland".to_string()));
        assert!(all.contains(&"artic".to_string())); // Added back
        assert!(all.contains(&"archive".to_string())); // Registered but unavailable
    }
}
