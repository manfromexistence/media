//! Rijksmuseum provider implementation.
//!
//! [Rijksmuseum API](https://data.rijksmuseum.nl)
//!
//! Provides access to 700,000+ CC0 licensed artworks from the Dutch National Museum.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Rijksmuseum provider for Dutch masterpieces.
/// Access to 700K+ CC0 licensed artworks including Rembrandt, Vermeer, and more.
#[derive(Debug)]
pub struct RijksmuseumProvider {
    client: HttpClient,
}

impl RijksmuseumProvider {
    /// Create a new Rijksmuseum provider.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let client = HttpClient::with_config(
            Self::RATE_LIMIT,
            config.retry_attempts,
            Duration::from_secs(config.timeout_secs),
        )
        .unwrap_or_default();

        Self { client }
    }

    /// Rate limit: 10000 requests per day
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(400, 3600);
}

#[async_trait]
impl Provider for RijksmuseumProvider {
    fn name(&self) -> &'static str {
        "rijksmuseum"
    }

    fn display_name(&self) -> &'static str {
        "Rijksmuseum"
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
        "https://www.rijksmuseum.nl/api/en/collection"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = self.base_url().to_string();

        let count = query.count.min(100).to_string();
        let page = query.page.to_string();

        // Rijksmuseum provides a free demo API key
        let params = [
            ("key", "0fiuZFh4"),
            ("q", query.query.as_str()),
            ("ps", count.as_str()),
            ("p", page.as_str()),
            ("imgonly", "true"),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: RijksmuseumResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .artObjects
            .into_iter()
            .filter_map(|obj| {
                let web_image = obj.webImage?;

                MediaAsset::builder()
                    .id(obj.objectNumber.clone())
                    .provider("rijksmuseum")
                    .media_type(MediaType::Image)
                    .title(obj.title)
                    .direct_download_url(web_image.url.clone())
                    .preview_url(web_image.url)
                    .source_url(obj.links.web.unwrap_or_default())
                    .author(obj.principalOrFirstMaker)
                    .license(License::Cc0)
                    .dimensions(
                        web_image.width.unwrap_or(0) as u32,
                        web_image.height.unwrap_or(0) as u32,
                    )
                    .build_or_log()
            })
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.count.unwrap_or(0),
            assets,
            providers_searched: vec!["rijksmuseum".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for RijksmuseumProvider {
    fn description(&self) -> &'static str {
        "Rijksmuseum - 700K+ CC0 licensed Dutch masterpieces (Rembrandt, Vermeer, etc.)"
    }

    fn api_key_url(&self) -> &'static str {
        "https://data.rijksmuseum.nl/object-metadata/api/"
    }

    fn default_license(&self) -> &'static str {
        "CC0 (Public Domain)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct RijksmuseumResponse {
    count: Option<usize>,
    artObjects: Vec<RijksmuseumArtObject>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct RijksmuseumArtObject {
    objectNumber: String,
    title: String,
    principalOrFirstMaker: String,
    webImage: Option<RijksmuseumImage>,
    links: RijksmuseumLinks,
}

#[derive(Debug, Deserialize)]
struct RijksmuseumImage {
    url: String,
    width: Option<i32>,
    height: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct RijksmuseumLinks {
    web: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default();
        let provider = RijksmuseumProvider::new(&config);

        assert_eq!(provider.name(), "rijksmuseum");
        assert_eq!(provider.display_name(), "Rijksmuseum");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default();
        let provider = RijksmuseumProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
    }
}
