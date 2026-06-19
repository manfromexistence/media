//! LoremFlickr provider - generated image endpoint by keyword.
//!
//! LoremFlickr provides keyword-based image URLs:
//! - Keyword-based image search
//! - Unlimited requests
//! - Multiple sizes
//! - Lock images by seed
//! - No API key required
//!
//! Original Flickr attribution and license are not resolved by this provider.
//!
//! URL: <https://loremflickr.com>

use async_trait::async_trait;
use std::collections::HashMap;

use crate::config::Config;
use crate::error::Result;
use crate::providers::traits::Provider;
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// LoremFlickr provider for generated keyword image endpoints.
#[derive(Debug)]
pub struct LoremFlickrProvider {
    // Note: No HTTP client needed - LoremFlickr generates URLs directly without API calls
}

impl LoremFlickrProvider {
    /// Create a new LoremFlickr provider.
    #[must_use]
    pub fn new(_config: &Config) -> Self {
        Self {}
    }

    /// Rate limit: generous
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(100, 60);

    /// Generate image URL for a keyword search.
    fn generate_url(keywords: &str, width: u32, height: u32, seed: Option<&str>) -> String {
        let encoded_keywords = keywords.split_whitespace().collect::<Vec<_>>().join(",");

        if let Some(seed) = seed {
            format!(
                "https://loremflickr.com/{}/{}/{}?lock={}",
                width, height, encoded_keywords, seed
            )
        } else {
            format!(
                "https://loremflickr.com/{}/{}/{}",
                width, height, encoded_keywords
            )
        }
    }

    fn provider_metadata(
        query: &SearchQuery,
        seed: &str,
        download_url: &str,
    ) -> HashMap<String, String> {
        HashMap::from([
            ("loremflickr.endpoint".to_string(), download_url.to_string()),
            ("loremflickr.query".to_string(), query.query.clone()),
            ("loremflickr.lock".to_string(), seed.to_string()),
            (
                "loremflickr.source_url_kind".to_string(),
                "generated-image-endpoint".to_string(),
            ),
            (
                "loremflickr.original_source".to_string(),
                "not-resolved".to_string(),
            ),
            (
                "loremflickr.license_evidence".to_string(),
                "not-resolved".to_string(),
            ),
        ])
    }
}

#[async_trait]
impl Provider for LoremFlickrProvider {
    fn name(&self) -> &'static str {
        "loremflickr"
    }

    fn display_name(&self) -> &'static str {
        "LoremFlickr"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image]
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn rate_limit(&self) -> RateLimitConfig {
        Self::RATE_LIMIT
    }

    fn is_available(&self) -> bool {
        true
    }

    fn base_url(&self) -> &'static str {
        "https://loremflickr.com"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let count = query.count.min(50);

        // Generate multiple unique images using different seeds
        let assets: Vec<MediaAsset> = (0..count)
            .map(|i| {
                let seed = format!("{}_{}", query.query, i);
                let url = Self::generate_url(&query.query, 800, 600, Some(&seed));
                let preview_url = Self::generate_url(&query.query, 400, 300, Some(&seed));
                let metadata = Self::provider_metadata(query, &seed, &url);
                let id = format!(
                    "loremflickr_{}_{}",
                    query.query.replace(' ', "_").to_lowercase(),
                    i
                );

                MediaAsset::builder()
                    .id(id)
                    .provider(self.name().to_string())
                    .title(format!("{} photo #{}", query.query, i + 1))
                    .media_type(MediaType::Image)
                    .direct_download_url(url.clone())
                    .preview_url(preview_url)
                    .source_url(url)
                    .license(License::Other(
                        "Unknown: LoremFlickr original Flickr license not resolved".to_string(),
                    ))
                    .dimensions(800, 600)
                    .provider_metadata(metadata)
                    .build_or_log()
            })
            .flatten()
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: count, // Effectively unlimited
            assets,
            providers_searched: vec![self.name().to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let config = Config::default();
        let provider = LoremFlickrProvider::new(&config);
        assert_eq!(provider.name(), "loremflickr");
        assert_eq!(provider.display_name(), "LoremFlickr");
        assert!(provider.is_available());
        assert!(!provider.requires_api_key());
    }

    #[test]
    fn test_url_generation() {
        let url = LoremFlickrProvider::generate_url("cat dog", 800, 600, Some("seed1"));
        assert!(url.contains("cat,dog"));
        assert!(url.contains("800/600"));
        assert!(url.contains("lock=seed1"));
    }

    #[tokio::test]
    async fn test_search_marks_original_source_and_license_unresolved() {
        let config = Config::default();
        let provider = LoremFlickrProvider::new(&config);

        let result = provider
            .search(&SearchQuery::for_type("red panda", MediaType::Image).count(1))
            .await
            .expect("loremflickr URLs should be generated without network");
        let asset = &result.assets[0];
        let provenance = asset.provenance();

        assert_eq!(
            asset
                .provider_metadata
                .get("loremflickr.source_url_kind")
                .map(String::as_str),
            Some("generated-image-endpoint")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("loremflickr.original_source")
                .map(String::as_str),
            Some("not-resolved")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("loremflickr.license_evidence")
                .map(String::as_str),
            Some("not-resolved")
        );
        assert!(!provenance.license_known);
    }
}
