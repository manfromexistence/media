//! Search engine for coordinating provider searches.

use std::sync::Arc;
use std::time::Instant;

use crate::constants::EARLY_EXIT_MULTIPLIER;
use crate::error::Result;
use crate::providers::ProviderRegistry;
use crate::types::{MediaType, SearchQuery, SearchResult};

/// Search engine for coordinating searches across providers.
#[derive(Debug)]
pub struct SearchEngine {
    registry: Arc<ProviderRegistry>,
}

impl SearchEngine {
    /// Create a new search engine with the given provider registry.
    #[must_use]
    pub fn new(registry: Arc<ProviderRegistry>) -> Self {
        Self { registry }
    }

    /// Create a search query builder.
    #[must_use]
    pub fn query(&self, terms: impl Into<String>) -> SearchQueryBuilder<'_> {
        SearchQueryBuilder {
            engine: self,
            query: SearchQuery::new(terms),
        }
    }

    /// Execute a search query.
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let start = Instant::now();

        // If specific providers requested, search only those
        let mut result = if !query.providers.is_empty() {
            self.search_specific_providers(query).await?
        } else {
            // Search all available providers that can serve the requested media type.
            let mut filtered_query = query.clone();
            if filtered_query.providers.is_empty() {
                filtered_query.providers = self.default_provider_names_for_query(query);
            }

            if filtered_query.providers.is_empty() {
                return Err(crate::error::DxError::NoResults {
                    query: query.query.clone(),
                });
            }

            self.search_specific_providers(&filtered_query).await?
        };

        result.duration_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    fn default_provider_names_for_query(&self, query: &SearchQuery) -> Vec<String> {
        let mut provider_names: Vec<_> = match query.media_type {
            Some(media_type) => self.registry.for_media_type(media_type),
            None => self.registry.available(),
        }
        .into_iter()
        .map(|provider| provider.name().to_string())
        .filter(|provider_name| !is_default_excluded_provider(provider_name))
        .collect();

        provider_names.sort();
        provider_names
    }

    /// Search specific providers by name (concurrently with timeouts).
    async fn search_specific_providers(&self, query: &SearchQuery) -> Result<SearchResult> {
        use crate::types::SearchMode;
        use futures::stream::{FuturesUnordered, StreamExt};
        use std::time::Duration;

        // Timeout varies by mode: Quantity=fast (5s), Quality=patient (8s)
        let provider_timeout = match query.mode {
            SearchMode::Quantity => Duration::from_secs(5),
            SearchMode::Quality => Duration::from_secs(8),
        };

        // Early exit only in Quantity mode
        let early_exit_threshold = query.count * EARLY_EXIT_MULTIPLIER;
        let use_early_exit = query.mode.is_quantity();

        // Create FuturesUnordered for concurrent execution
        let mut futures: FuturesUnordered<_> = query
            .providers
            .iter()
            .map(|provider_name| {
                let registry = Arc::clone(&self.registry);
                let provider_name = provider_name.clone();
                let query = query.clone();
                async move {
                    let result = tokio::time::timeout(
                        provider_timeout,
                        registry.search_provider(&provider_name, &query),
                    )
                    .await;

                    let timeout_msg =
                        format!("Provider timed out (>{}s)", provider_timeout.as_secs());
                    match result {
                        Ok(search_result) => (provider_name, search_result),
                        Err(_) => (
                            provider_name.clone(),
                            Err(crate::error::DxError::ProviderApi {
                                provider: provider_name,
                                message: timeout_msg,
                                status_code: 408,
                            }),
                        ),
                    }
                }
            })
            .collect();

        // Aggregate results as they complete
        let mut all_assets = Vec::new();
        let mut providers_searched = Vec::new();
        let mut provider_errors = Vec::new();
        let mut total_count = 0;
        let mut skipped_slow_providers = 0;

        while let Some((provider_name, result)) = futures.next().await {
            providers_searched.push(provider_name.clone());

            match result {
                Ok(search_result) => {
                    total_count += search_result.total_count;
                    all_assets.extend(search_result.assets);
                }
                Err(e) => {
                    provider_errors.push((provider_name, e.to_string()));
                }
            }

            // Early exit in Quantity mode
            if use_early_exit && all_assets.len() >= early_exit_threshold && !futures.is_empty() {
                skipped_slow_providers = futures.len();
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

        if all_assets.is_empty() && !provider_errors.is_empty() {
            // All providers failed
            let errors: Vec<String> = provider_errors
                .iter()
                .map(|(p, e)| format!("{}: {}", p, e))
                .collect();

            return Err(crate::error::DxError::ProviderApi {
                provider: "multiple".to_string(),
                message: errors.join("; "),
                status_code: 0,
            });
        }

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count,
            assets: all_assets,
            providers_searched,
            provider_errors,
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }

    /// Get the underlying provider registry.
    #[must_use]
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }
}

fn is_default_excluded_provider(provider_name: &str) -> bool {
    matches!(
        provider_name,
        "robohash" | "dicebear" | "loremflickr" | "picsum" | "placeholder"
    )
}

/// Builder for constructing search queries.
pub struct SearchQueryBuilder<'a> {
    engine: &'a SearchEngine,
    query: SearchQuery,
}

impl<'a> SearchQueryBuilder<'a> {
    /// Set the media type filter.
    #[must_use]
    pub fn media_type(mut self, media_type: MediaType) -> Self {
        self.query.media_type = Some(media_type);
        self
    }

    /// Set the number of results to return.
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
        self.engine.search(&self.query).await
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
    use crate::config::Config;

    #[test]
    fn test_search_query_builder() {
        let config = Config::default();
        let registry = Arc::new(ProviderRegistry::new(&config));
        let engine = SearchEngine::new(registry);

        let query = engine
            .query("nature")
            .media_type(MediaType::Image)
            .count(20)
            .page(2)
            .provider("openverse")
            .build();

        assert_eq!(query.query, "nature");
        assert_eq!(query.media_type, Some(MediaType::Image));
        assert_eq!(query.count, 20);
        assert_eq!(query.page, 2);
        assert_eq!(query.providers, vec!["openverse".to_string()]);
    }

    #[test]
    fn default_provider_selection_respects_media_type_and_availability() {
        let config = Config::default();
        let registry = Arc::new(ProviderRegistry::new(&config));
        let engine = SearchEngine::new(Arc::clone(&registry));
        let query = SearchQuery::for_type("apollo", MediaType::Video);

        let provider_names = engine.default_provider_names_for_query(&query);

        assert!(
            !provider_names.is_empty(),
            "video search should have at least one available provider"
        );
        assert!(
            !provider_names.iter().any(|name| name == "catapi"),
            "image-only providers should not be selected for default video search"
        );

        for provider_name in provider_names {
            let provider = registry
                .get(&provider_name)
                .expect("selected provider should be registered");

            assert!(provider.is_available(), "{provider_name}");
            assert!(
                provider.supported_media_types().contains(&MediaType::Video),
                "{provider_name}"
            );
            assert!(
                !is_default_excluded_provider(&provider_name),
                "{provider_name}"
            );
        }
    }
}
