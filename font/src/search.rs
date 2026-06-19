//! Font search functionality
//!
//! Provides unified search across all font providers with optimized
//! concurrent fetching for maximum performance.

use rayon::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{Instrument, info_span, instrument};

use crate::cdn::{CdnUrlGenerator, FontCdnUrls};
use crate::error::{FontError, FontResult};
use crate::models::{FontCategory, FontFamily, FontProvider, SearchQuery, SearchResults};
use crate::providers::ProviderRegistry;

/// Main font search engine with performance optimizations
pub struct FontSearch {
    registry: Arc<ProviderRegistry>,
}

impl FontSearch {
    /// Create a new font search engine with default providers
    pub fn new() -> FontResult<Self> {
        let registry = ProviderRegistry::with_defaults()?;
        Ok(Self {
            registry: Arc::new(registry),
        })
    }

    /// Create with a custom registry
    pub fn with_registry(registry: ProviderRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
        }
    }

    /// Search for fonts matching the query across all providers (concurrent)
    #[instrument(skip(self), fields(query = %query))]
    pub async fn search(&self, query: &str) -> FontResult<SearchResults> {
        tracing::info!(query = %query, "Starting font search");

        let search_query = SearchQuery {
            query: query.to_string(),
            ..Default::default()
        };

        let result = self
            .registry
            .search_all(&search_query)
            .instrument(info_span!("provider_search_all"))
            .await;

        match &result {
            Ok(results) => {
                tracing::info!(
                    query = %query,
                    total_fonts = results.total,
                    providers_searched = results.providers_searched.len(),
                    "Search completed successfully"
                );
            }
            Err(e) => {
                tracing::warn!(query = %query, error = %e, "Search failed");
            }
        }

        result
    }

    /// Search with timing information
    pub async fn search_timed(&self, query: &str) -> FontResult<(SearchResults, Duration)> {
        let start = Instant::now();
        let results = self.search(query).await?;
        let elapsed = start.elapsed();
        Ok((results, elapsed))
    }

    /// Search with advanced options
    #[instrument(skip(self), fields(query = %query.query))]
    pub async fn search_advanced(&self, query: SearchQuery) -> FontResult<SearchResults> {
        self.registry
            .search_all(&query)
            .instrument(info_span!("provider_search_all"))
            .await
    }

    /// Search for fonts by category
    #[instrument(skip(self), fields(category = ?category))]
    pub async fn search_by_category(&self, category: FontCategory) -> FontResult<SearchResults> {
        let query = SearchQuery {
            query: String::new(),
            category: Some(category.clone()),
            ..Default::default()
        };

        let mut results = self
            .registry
            .search_all(&query)
            .instrument(info_span!("provider_search_all"))
            .await?;

        // Filter by category
        results
            .fonts
            .retain(|f| f.category.as_ref() == Some(&category));
        results.total = results.fonts.len();

        Ok(results)
    }

    /// List all available fonts from all providers (concurrent)
    #[instrument(skip(self))]
    pub async fn list_all(&self) -> FontResult<SearchResults> {
        tracing::info!("Starting to list all fonts from all providers");

        let result = self
            .registry
            .list_all_concurrent()
            .instrument(info_span!("provider_list_all"))
            .await;

        match &result {
            Ok(results) => {
                tracing::info!(
                    total_fonts = results.total,
                    providers_searched = results.providers_searched.len(),
                    "List all completed successfully"
                );
            }
            Err(e) => {
                tracing::warn!(error = %e, "List all failed");
            }
        }

        result
    }

    /// List all with timing information
    pub async fn list_all_timed(&self) -> FontResult<(SearchResults, Duration)> {
        let start = Instant::now();
        let results = self.list_all().await?;
        let elapsed = start.elapsed();
        Ok((results, elapsed))
    }

    /// Get detailed information about a specific font
    #[instrument(skip(self), fields(provider = %provider.name(), font_id = %font_id))]
    pub async fn get_font_details(
        &self,
        provider: &FontProvider,
        font_id: &str,
    ) -> FontResult<FontFamily> {
        for p in self.registry.providers() {
            if p.name() == provider.name() {
                return p.get_font_family(font_id).await;
            }
        }

        Err(FontError::provider(
            provider.name(),
            format!("Provider not found: {:?}", provider),
        ))
    }

    /// Get CDN URLs for a font for preview/usage
    pub fn get_cdn_urls(
        &self,
        font_id: &str,
        font_name: &str,
        provider: &FontProvider,
    ) -> FontCdnUrls {
        match provider {
            FontProvider::GoogleFonts => CdnUrlGenerator::for_google_font(font_id, font_name),
            FontProvider::BunnyFonts => CdnUrlGenerator::for_bunny_font(font_id, font_name),
            FontProvider::Fontsource => CdnUrlGenerator::for_fontsource_font(font_id),
            _ => CdnUrlGenerator::for_google_font(font_id, font_name),
        }
    }

    /// Check health of all providers (concurrent)
    pub async fn health_check(&self) -> Vec<(String, bool)> {
        self.registry
            .health_check_all()
            .await
            .into_iter()
            .map(|(name, healthy, _)| (name, healthy))
            .collect()
    }

    /// Check health with timing information
    pub async fn health_check_timed(&self) -> Vec<(String, bool, Duration)> {
        self.registry.health_check_all().await
    }

    /// Get statistics about available fonts
    pub async fn get_stats(&self) -> FontResult<FontStats> {
        let (results, elapsed) = self.list_all_timed().await?;

        let mut stats = FontStats {
            total_fonts: results.total,
            providers_count: results.providers_searched.len(),
            providers: results.providers_searched,
            fetch_time_ms: elapsed.as_millis() as u64,
            ..Default::default()
        };

        // Count by category using parallel processing (Rayon)
        let category_counts: Vec<(Option<FontCategory>, usize)> = results
            .fonts
            .par_iter()
            .fold(std::collections::HashMap::new, |mut acc, font| {
                *acc.entry(font.category.clone()).or_insert(0) += 1;
                acc
            })
            .reduce(std::collections::HashMap::new, |mut a, b| {
                for (k, v) in b {
                    *a.entry(k).or_insert(0) += v;
                }
                a
            })
            .into_iter()
            .collect();

        for (category, count) in category_counts {
            match category {
                Some(FontCategory::Serif) => stats.serif_count = count,
                Some(FontCategory::SansSerif) => stats.sans_serif_count = count,
                Some(FontCategory::Display) => stats.display_count = count,
                Some(FontCategory::Handwriting) => stats.handwriting_count = count,
                Some(FontCategory::Monospace) => stats.monospace_count = count,
                None => stats.uncategorized_count = count,
            }
        }

        Ok(stats)
    }
}

/// Font statistics with performance metrics
#[derive(Debug, Default, serde::Serialize)]
pub struct FontStats {
    pub total_fonts: usize,
    pub providers_count: usize,
    pub providers: Vec<String>,
    pub serif_count: usize,
    pub sans_serif_count: usize,
    pub display_count: usize,
    pub handwriting_count: usize,
    pub monospace_count: usize,
    pub uncategorized_count: usize,
    pub fetch_time_ms: u64,
}
