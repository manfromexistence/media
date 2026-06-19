//! Met Museum Open Access provider implementation.
//!
//! [Met Museum API](https://metmuseum.github.io/)
//!
//! Provides access to 500,000+ CC0 licensed artworks from the Metropolitan Museum of Art.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Met Museum Open Access provider for artwork images.
/// Access to 500K+ CC0 licensed images of artworks.
#[derive(Debug)]
pub struct MetMuseumProvider {
    client: HttpClient,
}

impl MetMuseumProvider {
    /// Create a new Met Museum provider.
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

    /// Rate limit: Unlimited
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(100, 60);

    /// Fetch object details by ID
    async fn fetch_object(&self, object_id: u64) -> Result<Option<MetObject>> {
        let url = format!("{}/objects/{}", self.base_url(), object_id);

        let response = self.client.get(&url).await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let obj: MetObject = response.json_or_error().await?;

        // Only return objects that have images and are public domain
        if obj.is_public_domain && obj.primary_image.is_some() {
            Ok(Some(obj))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl Provider for MetMuseumProvider {
    fn name(&self) -> &'static str {
        "met"
    }

    fn display_name(&self) -> &'static str {
        "Met Museum"
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
        "https://collectionapi.metmuseum.org/public/collection/v1"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        // First, search for object IDs
        let search_url = format!("{}/search", self.base_url());

        let params = [
            ("q", query.query.as_str()),
            ("hasImages", "true"),
            ("isPublicDomain", "true"),
        ];

        let response = self
            .client
            .get_with_query(&search_url, &params, &[])
            .await?;

        let search_response: MetSearchResponse = response.json_or_error().await?;

        // Get paginated subset of object IDs
        let start = (query.page - 1) * query.count;
        let end =
            (start + query.count).min(search_response.object_ids.as_ref().map_or(0, |v| v.len()));

        let object_ids = search_response
            .object_ids
            .as_ref()
            .map(|ids| ids[start..end].to_vec())
            .unwrap_or_default();

        // Fetch details for each object (limit concurrent requests)
        let mut assets = Vec::new();
        for object_id in object_ids.into_iter().take(query.count) {
            if let Ok(Some(obj)) = self.fetch_object(object_id).await {
                let tags: Vec<String> = obj
                    .tags
                    .unwrap_or_default()
                    .into_iter()
                    .map(|t| t.term)
                    .collect();

                let asset = MediaAsset::builder()
                    .id(obj.object_id.to_string())
                    .provider("met")
                    .media_type(MediaType::Image)
                    .title(
                        obj.title
                            .unwrap_or_else(|| "Met Museum Artwork".to_string()),
                    )
                    .direct_download_url(obj.primary_image.clone().unwrap_or_default())
                    .preview_url(
                        obj.primary_image_small
                            .unwrap_or_else(|| obj.primary_image.unwrap_or_default()),
                    )
                    .source_url(obj.object_url)
                    .author(
                        obj.artist_display_name
                            .unwrap_or_else(|| "Unknown Artist".to_string()),
                    )
                    .license(License::Cc0)
                    .tags(tags)
                    .build_or_log();

                if let Some(asset) = asset {
                    assets.push(asset);
                }
            }
        }

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: search_response.total,
            assets,
            providers_searched: vec!["met".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for MetMuseumProvider {
    fn description(&self) -> &'static str {
        "Metropolitan Museum of Art's 500K+ CC0 licensed artwork images"
    }

    fn api_key_url(&self) -> &'static str {
        "https://metmuseum.github.io/"
    }

    fn default_license(&self) -> &'static str {
        "CC0 (Public Domain)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MetSearchResponse {
    total: usize,
    #[serde(rename = "objectIDs")]
    object_ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MetObject {
    #[serde(rename = "objectID")]
    object_id: u64,
    #[serde(rename = "isPublicDomain")]
    is_public_domain: bool,
    #[serde(rename = "primaryImage")]
    primary_image: Option<String>,
    #[serde(rename = "primaryImageSmall")]
    primary_image_small: Option<String>,
    title: Option<String>,
    #[serde(rename = "artistDisplayName")]
    artist_display_name: Option<String>,
    #[serde(rename = "artistDisplayBio")]
    artist_display_bio: Option<String>,
    #[serde(rename = "objectURL")]
    object_url: String,
    department: Option<String>,
    culture: Option<String>,
    period: Option<String>,
    dynasty: Option<String>,
    #[serde(default)]
    tags: Option<Vec<MetTag>>,
}

#[derive(Debug, Deserialize)]
struct MetTag {
    term: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = MetMuseumProvider::new(&config);

        assert_eq!(provider.name(), "met");
        assert_eq!(provider.display_name(), "Met Museum");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default_for_testing();
        let provider = MetMuseumProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
    }
}
