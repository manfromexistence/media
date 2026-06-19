//! Provider registry for managing all media providers.
//!
//! Supports keyless and credential-gated providers. Credential-gated providers
//! gracefully degrade when credentials are not configured.
//!
//! Includes circuit breaker integration for provider resilience.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::engine::{CircuitBreaker, CircuitState};
use crate::error::{DxError, Result};
use crate::providers::traits::Provider;
use crate::providers::{
    // Provider implementations.
    ArtInstituteChicagoProvider,
    CatApiProvider,
    ClevelandMuseumProvider,
    DataGovProvider,
    DiceBearProvider,
    DogCeoProvider,
    DplaProvider,
    EuropeanaProvider,
    FreesoundProvider,
    GiphyProvider,
    GitHubProvider,
    InternetArchiveProvider,
    LibraryOfCongressProvider,
    LoremFlickrProvider,
    LoremPicsumProvider,
    MetMuseumProvider,
    NasaImagesProvider,
    NekosBestProvider,
    OpenLibraryProvider,
    OpenverseProvider,
    PexelsProvider,
    PixabayProvider,
    PolyHavenProvider,
    RandomFoxProvider,
    RijksmuseumProvider,
    RoboHashProvider,
    ScryfallProvider,
    SmithsonianProvider,
    UnsplashProvider,
    VandAMuseumProvider,
    WaifuPicsProvider,
    WaltersArtMuseumProvider,
    WikimediaCommonsProvider,
    XkcdProvider,
};
use crate::types::{MediaType, SearchQuery, SearchResult};

/// Registry for managing and querying media providers.
///
/// Callers should use provider listing metadata instead of inferring access
/// from provider names. The listing exposes `source_kind`,
/// `requires_credentials`, `credential_status`, and unavailable reasons.
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
}

impl std::fmt::Debug for ProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderRegistry")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl ProviderRegistry {
    /// Create a new registry with all available providers.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let mut providers: HashMap<String, Arc<dyn Provider>> = HashMap::new();
        let mut circuit_breakers: HashMap<String, CircuitBreaker> = HashMap::new();

        // Helper to register a provider with its circuit breaker
        let mut register = |provider: Arc<dyn Provider>| {
            let name = provider.name().to_string();
            circuit_breakers.insert(name.clone(), CircuitBreaker::default());
            providers.insert(name, provider);
        };

        // ═══════════════════════════════════════════════════════════════════
        // TIER 1: High-Volume Providers
        // ═══════════════════════════════════════════════════════════════════

        // Openverse - 700M+ images and audio (no API key required)
        register(Arc::new(OpenverseProvider::new(config)));

        // Wikimedia Commons - 92M+ files (no API key required)
        register(Arc::new(WikimediaCommonsProvider::new(config)));

        // Europeana - 50M+ European cultural heritage items
        register(Arc::new(EuropeanaProvider::new(config)));

        // DPLA - 40M+ American cultural heritage items (requires API key now)
        register(Arc::new(DplaProvider::new(config)));

        // Library of Congress - 3M+ public domain images
        register(Arc::new(LibraryOfCongressProvider::new(config)));

        // Internet Archive - 26M+ media items (images, video, audio, docs)
        register(Arc::new(InternetArchiveProvider::new(config)));

        // ═══════════════════════════════════════════════════════════════════
        // TIER 2: Museum Providers
        // ═══════════════════════════════════════════════════════════════════

        // Rijksmuseum - 700K+ Dutch masterpieces (CC0)
        register(Arc::new(RijksmuseumProvider::new(config)));

        // Met Museum - 500K+ artworks (no API key required)
        register(Arc::new(MetMuseumProvider::new(config)));

        // NASA Images - 140K+ space images (no API key required)
        register(Arc::new(NasaImagesProvider::new(config)));

        // Cleveland Museum - 61K+ artworks (CC0)
        register(Arc::new(ClevelandMuseumProvider::new(config)));

        // Art Institute of Chicago - 50K+ artworks (CC0)
        register(Arc::new(ArtInstituteChicagoProvider::new(config)));

        // ═══════════════════════════════════════════════════════════════════
        // TIER 3: 3D & Utility Providers
        // ═══════════════════════════════════════════════════════════════════

        // Poly Haven - 3.7K+ 3D models, textures, HDRIs (CC0)
        register(Arc::new(PolyHavenProvider::new(config)));

        // Lorem Picsum - Unlimited placeholder images (no API key required)
        register(Arc::new(LoremPicsumProvider::new(config)));

        // ═══════════════════════════════════════════════════════════════════
        // TIER 3.5: Animal & Avatar Providers
        // ═══════════════════════════════════════════════════════════════════

        // Dog CEO - 20K+ dog images
        register(Arc::new(DogCeoProvider::new(config)));

        // Cat API - 60K+ cat images
        register(Arc::new(CatApiProvider::new(config)));

        // Random Fox - Unlimited fox images
        register(Arc::new(RandomFoxProvider::new(config)));

        // DiceBear - Unlimited avatar generation (25+ styles)
        register(Arc::new(DiceBearProvider::new(config)));

        // RoboHash - Unlimited robot/monster avatars
        register(Arc::new(RoboHashProvider::new(config)));

        // ═══════════════════════════════════════════════════════════════════
        // TIER 3.6: Additional Museums
        // ═══════════════════════════════════════════════════════════════════

        // V&A Museum - 1.2M+ art and design objects
        register(Arc::new(VandAMuseumProvider::new(config)));

        // Walters Art Museum - 25K+ artworks (CC0)
        register(Arc::new(WaltersArtMuseumProvider::new(config)));

        // Open Library - 30M+ book covers
        register(Arc::new(OpenLibraryProvider::new(config)));

        // ═══════════════════════════════════════════════════════════════════
        // TIER 3.7: Anime & GIFs
        // ═══════════════════════════════════════════════════════════════════

        // Waifu.pics - Unlimited anime images and GIFs
        register(Arc::new(WaifuPicsProvider::new(config)));

        // Nekos.best - High-quality anime images and GIFs
        register(Arc::new(NekosBestProvider::new(config)));

        // ═══════════════════════════════════════════════════════════════════
        // TIER 3.8: Cards, Comics & Special
        // ═══════════════════════════════════════════════════════════════════

        // Scryfall - 80K+ Magic: The Gathering cards
        register(Arc::new(ScryfallProvider::new(config)));

        // xkcd - 2,900+ webcomics
        register(Arc::new(XkcdProvider::new(config)));

        // LoremFlickr - generated image endpoints; original Flickr provenance unresolved
        register(Arc::new(LoremFlickrProvider::new(config)));

        // ═══════════════════════════════════════════════════════════════════
        // TIER 3.9: Data & Document Providers
        // ═══════════════════════════════════════════════════════════════════

        // Data.gov - 300K+ US Government datasets (JSON, CSV, XML)
        register(Arc::new(DataGovProvider::new(config)));

        // GitHub - Data files (JSON, CSV, PDF, Excel) from public repos
        register(Arc::new(GitHubProvider::new(config)));

        // ═══════════════════════════════════════════════════════════════════
        // TIER 4: Credential-gated providers
        // These providers are available in searches when credentials are configured.
        // ═══════════════════════════════════════════════════════════════════

        // Unsplash - 5M+ high-quality photos (free API key at unsplash.com/developers)
        register(Arc::new(UnsplashProvider::new(config)));

        // Pexels - 3.5M+ photos & videos (free API key at pexels.com/api)
        register(Arc::new(PexelsProvider::new(config)));

        // Pixabay - 4.2M+ images, videos, music (free API key at pixabay.com/api/docs)
        register(Arc::new(PixabayProvider::new(config)));

        // Freesound - 600K+ sound effects (free API key at freesound.org/apiv2/apply)
        register(Arc::new(FreesoundProvider::new(config)));

        // Giphy - Millions of GIFs (free API key at developers.giphy.com)
        register(Arc::new(GiphyProvider::new(config)));

        // Smithsonian - 4.5M+ CC0 images (free API key at api.si.edu)
        register(Arc::new(SmithsonianProvider::new(config)));

        Self {
            providers,
            circuit_breakers,
        }
    }

    /// Get a provider by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Arc<dyn Provider>> {
        self.providers.get(name).cloned()
    }

    /// Get all registered providers.
    #[must_use]
    pub fn all(&self) -> Vec<Arc<dyn Provider>> {
        self.providers.values().cloned().collect()
    }

    /// Get all providers available under the current configuration.
    #[must_use]
    pub fn available(&self) -> Vec<Arc<dyn Provider>> {
        self.providers
            .values()
            .filter(|p| p.is_available())
            .cloned()
            .collect()
    }

    /// Get providers that support a specific media type.
    #[must_use]
    pub fn for_media_type(&self, media_type: MediaType) -> Vec<Arc<dyn Provider>> {
        self.providers
            .values()
            .filter(|p| p.is_available() && p.supported_media_types().contains(&media_type))
            .cloned()
            .collect()
    }

    /// Get the names of all registered providers.
    #[must_use]
    pub fn provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Get the names of all available providers.
    #[must_use]
    pub fn available_provider_names(&self) -> Vec<String> {
        self.providers
            .iter()
            .filter(|(_, p)| p.is_available())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Check if a provider exists by name.
    #[must_use]
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Get the circuit breaker for a provider.
    #[must_use]
    pub fn get_circuit_breaker(&self, name: &str) -> Option<&CircuitBreaker> {
        self.circuit_breakers.get(name)
    }

    /// Check if a provider's circuit breaker is open.
    #[must_use]
    pub fn is_circuit_open(&self, name: &str) -> bool {
        self.circuit_breakers
            .get(name)
            .map(|cb| cb.state() == CircuitState::Open && !cb.allow_request())
            .unwrap_or(false)
    }

    /// Get available providers, excluding those with open circuit breakers.
    #[must_use]
    pub fn available_with_circuit_check(&self) -> Vec<Arc<dyn Provider>> {
        self.providers
            .iter()
            .filter(|(name, p)| {
                p.is_available()
                    && self
                        .circuit_breakers
                        .get(*name)
                        .map_or(true, |cb| cb.allow_request())
            })
            .map(|(_, p)| p.clone())
            .collect()
    }

    /// Search a specific provider with circuit breaker protection.
    ///
    /// Returns `DxError::CircuitBreakerOpen` if the provider's circuit is open.
    pub async fn search_provider(
        &self,
        provider_name: &str,
        query: &SearchQuery,
    ) -> Result<SearchResult> {
        let provider = self
            .get(provider_name)
            .ok_or_else(|| DxError::ProviderApi {
                provider: provider_name.to_string(),
                message: "Provider not found".to_string(),
                status_code: 404,
            })?;

        // Check circuit breaker before making request
        if let Some(cb) = self.circuit_breakers.get(provider_name) {
            if !cb.allow_request() {
                return Err(DxError::circuit_breaker_open(provider_name));
            }
        }

        // Execute the search
        let result = provider.search(query).await;

        // Record success/failure in circuit breaker
        if let Some(cb) = self.circuit_breakers.get(provider_name) {
            match &result {
                Ok(_) => cb.record_success(),
                Err(_) => cb.record_failure(),
            }
        }

        result
    }

    /// Search all available providers and aggregate results.
    ///
    /// This searches all providers **concurrently** with aggressive timeouts.
    /// Uses `FuturesUnordered` for optimal performance - results are processed
    /// as they arrive, and slow providers are timed out after 5 seconds.
    ///
    /// Circuit breakers are checked before each provider request, and
    /// success/failure is recorded to manage provider health.
    ///
    /// # Search Modes
    /// - **Quantity** (default): Early exit after 3x results - FAST but may skip slow providers
    /// - **Quality**: Waits for ALL providers to respond - thorough but slower
    pub async fn search_all(&self, query: &SearchQuery) -> Result<SearchResult> {
        use crate::types::SearchMode;
        use futures::stream::{FuturesUnordered, StreamExt};

        let providers = match query.media_type {
            Some(media_type) => self.for_media_type(media_type),
            None => self.available(),
        };

        if providers.is_empty() {
            return Err(DxError::NoResults {
                query: query.query.clone(),
            });
        }

        // AGGRESSIVE TIMEOUT: 5 seconds max per provider
        // In Quality mode, we use 8 seconds to give slow providers more time
        let provider_timeout = match query.mode {
            SearchMode::Quantity => Duration::from_secs(5),
            SearchMode::Quality => Duration::from_secs(8),
        };

        // Early exit threshold (only used in Quantity mode)
        let early_exit_threshold = query.count * 3;
        let use_early_exit = query.mode.is_quantity();

        // Filter out providers with open circuit breakers
        let providers_with_cb: Vec<_> = providers
            .iter()
            .filter_map(|p| {
                let name = p.name();
                let cb = self.circuit_breakers.get(name)?;
                if cb.allow_request() {
                    Some((Arc::clone(p), name.to_string()))
                } else {
                    None
                }
            })
            .collect();

        // Track providers skipped due to open circuits
        let circuit_open_providers: Vec<_> = providers
            .iter()
            .filter(|p| {
                self.circuit_breakers
                    .get(p.name())
                    .map(|cb| !cb.allow_request())
                    .unwrap_or(false)
            })
            .map(|p| p.name().to_string())
            .collect();

        // Create a FuturesUnordered for concurrent execution with early returns
        let circuit_breakers = &self.circuit_breakers;
        let mut futures: FuturesUnordered<_> = providers_with_cb
            .into_iter()
            .map(|(provider, name)| {
                let query = query.clone();
                async move {
                    let start = std::time::Instant::now();
                    // Wrap each provider search in a timeout
                    let result =
                        tokio::time::timeout(provider_timeout, provider.search(&query)).await;

                    let elapsed_ms = start.elapsed().as_millis() as u64;
                    let timeout_msg =
                        format!("Provider timed out (>{}s)", provider_timeout.as_secs());
                    match result {
                        Ok(search_result) => (name, search_result, elapsed_ms),
                        Err(_) => (
                            name.clone(),
                            Err(DxError::ProviderApi {
                                provider: name,
                                message: timeout_msg,
                                status_code: 408,
                            }),
                            elapsed_ms,
                        ),
                    }
                }
            })
            .collect();

        // Collect results as they complete (not waiting for all in Quantity mode)
        let mut all_assets = Vec::new();
        let mut providers_searched = Vec::new();
        let mut provider_errors = Vec::new();
        let mut provider_timings = std::collections::HashMap::new();
        let mut total_count = 0;
        let mut skipped_slow_providers = 0;

        // Add circuit breaker open errors
        for name in circuit_open_providers {
            provider_errors.push((
                name.clone(),
                "Circuit breaker open - provider temporarily disabled".to_string(),
            ));
        }

        while let Some((provider_name, result, elapsed_ms)) = futures.next().await {
            providers_searched.push(provider_name.clone());
            provider_timings.insert(provider_name.clone(), elapsed_ms);

            // Record success/failure in circuit breaker
            if let Some(cb) = circuit_breakers.get(&provider_name) {
                match &result {
                    Ok(_) => cb.record_success(),
                    Err(_) => cb.record_failure(),
                }
            }

            match result {
                Ok(search_result) => {
                    total_count += search_result.total_count;
                    all_assets.extend(search_result.assets);
                }
                Err(e) => {
                    provider_errors.push((provider_name, e.to_string()));
                }
            }

            // EARLY EXIT: Only in Quantity mode - if we have enough results, stop
            if use_early_exit && all_assets.len() >= early_exit_threshold && !futures.is_empty() {
                skipped_slow_providers = futures.len();
                // Cancel remaining futures by dropping them
                drop(futures);
                break;
            }
        }

        if skipped_slow_providers > 0 {
            provider_errors.push((
                "early_exit".to_string(),
                format!(
                    "Skipped {} slow providers (had {} results)",
                    skipped_slow_providers,
                    all_assets.len()
                ),
            ));
        }

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count,
            assets: all_assets,
            providers_searched,
            provider_errors,
            duration_ms: 0,
            provider_timings,
        })
    }

    /// Get provider count statistics.
    #[must_use]
    pub fn stats(&self) -> ProviderStats {
        let total = self.providers.len();
        let available = self.providers.values().filter(|p| p.is_available()).count();
        let unavailable = total - available;

        ProviderStats {
            total,
            available,
            unavailable,
        }
    }
}

/// Statistics about registered providers.
#[derive(Debug, Clone, Copy)]
pub struct ProviderStats {
    /// Total number of registered providers.
    pub total: usize,
    /// Number of providers available under the current configuration.
    pub available: usize,
    /// Number of unavailable providers.
    pub unavailable: usize,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let config = Config::default();
        let registry = ProviderRegistry::new(&config);

        // Keyless providers should be registered and available without credentials.
        // Tier 1: High-volume providers
        assert!(registry.has_provider("openverse"));
        assert!(registry.has_provider("wikimedia"));
        assert!(registry.has_provider("europeana"));
        assert!(registry.has_provider("dpla"));
        assert!(registry.has_provider("loc"));

        // Tier 2: Museum providers
        assert!(registry.has_provider("rijksmuseum"));
        assert!(registry.has_provider("met"));
        assert!(registry.has_provider("nasa"));
        assert!(registry.has_provider("cleveland"));
        assert!(registry.has_provider("artic"));
        assert!(registry.has_provider("archive"));

        // Tier 3: 3D & Utility providers
        assert!(registry.has_provider("polyhaven"));
        assert!(registry.has_provider("picsum"));

        // Tier 3.7: Anime & GIFs
        assert!(registry.has_provider("waifupics"));
        assert!(registry.has_provider("nekosbest"));

        // Tier 3.8: Cards, Comics & Special
        assert!(registry.has_provider("scryfall"));
        assert!(registry.has_provider("xkcd"));
        assert!(registry.has_provider("loremflickr"));

        // Credential-gated providers are registered even when credentials are missing.
        assert!(registry.has_provider("unsplash"));
        assert!(registry.has_provider("pexels"));
        assert!(registry.has_provider("pixabay"));
        assert!(registry.has_provider("freesound"));
        assert!(registry.has_provider("giphy"));
        assert!(registry.has_provider("smithsonian"));

        // Removed providers
        assert!(!registry.has_provider("nonexistent"));
    }

    #[test]
    fn test_provider_stats() {
        let config = Config::default();
        let registry = ProviderRegistry::new(&config);

        let stats = registry.stats();
        // Total registered providers in the default registry.
        assert_eq!(stats.total, 34);
        // Providers available with the default local configuration.
        assert_eq!(stats.available, 23);
        // Providers unavailable because they need credentials or are disabled.
        assert_eq!(stats.unavailable, 11);
    }

    #[test]
    fn test_get_provider() {
        let config = Config::default();
        let registry = ProviderRegistry::new(&config);

        let provider = registry.get("openverse");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name(), "openverse");
    }
}
