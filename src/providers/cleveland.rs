//! Cleveland Museum of Art provider implementation.
//!
//! [Cleveland Museum of Art Open Access API](https://openaccess-api.clevelandart.org)
//!
//! Provides access to 61,000+ CC0 licensed artworks.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Cleveland Museum of Art provider for CC0 artworks.
/// Access to 61K+ CC0 licensed artworks spanning 6,000 years of human history.
#[derive(Debug)]
pub struct ClevelandMuseumProvider {
    client: HttpClient,
}

impl ClevelandMuseumProvider {
    /// Create a new Cleveland Museum of Art provider.
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

    /// Rate limit: Unlimited but be respectful
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(100, 60);
}

#[async_trait]
impl Provider for ClevelandMuseumProvider {
    fn name(&self) -> &'static str {
        "cleveland"
    }

    fn display_name(&self) -> &'static str {
        "Cleveland Museum of Art"
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
        "https://openaccess-api.clevelandart.org/api/artworks"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = self.base_url().to_string();

        let limit = query.count.min(100).to_string();
        let skip = ((query.page - 1) * query.count).to_string();

        let params = [
            ("q", query.query.as_str()),
            ("limit", limit.as_str()),
            ("skip", skip.as_str()),
            ("has_image", "1"),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: ClevelandResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .data
            .into_iter()
            .filter_map(|artwork| {
                let images = artwork.images?;
                let web_image = images.web?;

                Some(
                    MediaAsset::builder()
                        .id(artwork.id.to_string())
                        .provider("cleveland")
                        .media_type(MediaType::Image)
                        .title(artwork.title.unwrap_or_else(|| "Untitled".to_string()))
                        .direct_download_url(web_image.url.clone())
                        .preview_url(web_image.url)
                        .source_url(artwork.url.unwrap_or_default())
                        .author(
                            artwork
                                .creators
                                .map(|c| {
                                    c.into_iter()
                                        .map(|cr| cr.description)
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                })
                                .unwrap_or_default(),
                        )
                        .license(License::Cc0)
                        .dimensions(web_image.width.unwrap_or(0), web_image.height.unwrap_or(0))
                        .build_or_log(),
                )
            })
            .flatten()
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.info.total.unwrap_or(0),
            assets,
            providers_searched: vec!["cleveland".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for ClevelandMuseumProvider {
    fn description(&self) -> &'static str {
        "Cleveland Museum of Art - 61K+ CC0 licensed artworks spanning 6,000 years"
    }

    fn api_key_url(&self) -> &'static str {
        "https://openaccess-api.clevelandart.org/"
    }

    fn default_license(&self) -> &'static str {
        "CC0 (Public Domain)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct ClevelandResponse {
    info: ClevelandInfo,
    data: Vec<ClevelandArtwork>,
}

#[derive(Debug, Deserialize)]
struct ClevelandInfo {
    total: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ClevelandArtwork {
    id: i64,
    title: Option<String>,
    url: Option<String>,
    creators: Option<Vec<ClevelandCreator>>,
    images: Option<ClevelandImages>,
}

#[derive(Debug, Deserialize)]
struct ClevelandCreator {
    description: String,
}

#[derive(Debug, Deserialize)]
struct ClevelandImages {
    web: Option<ClevelandImage>,
}

#[derive(Debug, Deserialize)]
struct ClevelandImage {
    url: String,
    #[serde(deserialize_with = "deserialize_dimension")]
    width: Option<u32>,
    #[serde(deserialize_with = "deserialize_dimension")]
    height: Option<u32>,
}

fn deserialize_dimension<'de, D>(deserializer: D) -> std::result::Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match value {
        Some(serde_json::Value::Number(n)) => Ok(n.as_u64().map(|v| v as u32)),
        Some(serde_json::Value::String(s)) => Ok(s.parse().ok()),
        _ => Ok(None),
    }
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
        let provider = ClevelandMuseumProvider::new(&config);

        assert_eq!(provider.name(), "cleveland");
        assert_eq!(provider.display_name(), "Cleveland Museum of Art");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default();
        let provider = ClevelandMuseumProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
    }
}
