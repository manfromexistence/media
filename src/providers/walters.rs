//! Walters Art Museum provider implementation.
//!
//! [Walters Art Museum](https://art.thewalters.org/)
//!
//! 25K+ artworks - no API key required, CC0 licensed.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Walters Art Museum provider.
/// No API key required, CC0 licensed.
#[derive(Debug)]
pub struct WaltersArtMuseumProvider {
    client: HttpClient,
}

impl WaltersArtMuseumProvider {
    /// Create a new Walters Art Museum provider.
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

    /// Rate limit
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(60, 60);
}

#[async_trait]
impl Provider for WaltersArtMuseumProvider {
    fn name(&self) -> &'static str {
        "walters"
    }

    fn display_name(&self) -> &'static str {
        "Walters Art Museum"
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
        // NOTE: Walters API is now behind Cloudflare - disabled until they fix it
        false
    }

    fn base_url(&self) -> &'static str {
        "https://api.thewalters.org/v1"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let count = query.count.min(100);
        let page = query.page;

        let count_str = count.to_string();
        let page_str = page.to_string();

        let url = format!("{}/objects", self.base_url());
        let params = [
            ("keyword", query.query.as_str()),
            ("pageSize", count_str.as_str()),
            ("page", page_str.as_str()),
            ("orderBy", "ObjectID"),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;
        let data: WaltersResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = data
            .items
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                let image_url = item.primary_image.large_thumb_path?;
                let preview_url = item
                    .primary_image
                    .small_thumb_path
                    .unwrap_or_else(|| image_url.clone());

                MediaAsset::builder()
                    .id(format!("walters_{}", item.object_id))
                    .provider("walters")
                    .media_type(MediaType::Image)
                    .title(item.title.unwrap_or_else(|| "Untitled".to_string()))
                    .direct_download_url(image_url)
                    .preview_url(preview_url)
                    .source_url(format!(
                        "https://art.thewalters.org/detail/{}",
                        item.object_id
                    ))
                    .author(item.creator.unwrap_or_default())
                    .license(License::Cc0)
                    .tags(
                        vec![
                            item.classification.unwrap_or_default(),
                            item.medium.unwrap_or_default(),
                        ]
                        .into_iter()
                        .filter(|s| !s.is_empty())
                        .collect(),
                    )
                    .build_or_log()
            })
            .collect();

        let total = assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: data
                .return_status
                .as_ref()
                .and_then(|s| s.total_count)
                .unwrap_or(total),
            assets,
            providers_searched: vec!["walters".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for WaltersArtMuseumProvider {
    fn description(&self) -> &'static str {
        "Walters Art Museum - 25K+ artworks, CC0 licensed"
    }

    fn api_key_url(&self) -> &'static str {
        "https://api.thewalters.org/"
    }

    fn default_license(&self) -> &'static str {
        "CC0 1.0 Public Domain"
    }
}

/// Walters API response structures
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WaltersResponse {
    items: Option<Vec<WaltersItem>>,
    return_status: Option<WaltersStatus>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WaltersStatus {
    total_count: Option<usize>,
}

// Fields are read during serde deserialization
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WaltersItem {
    object_id: i64,
    title: Option<String>,
    description: Option<String>,
    creator: Option<String>,
    classification: Option<String>,
    medium: Option<String>,
    primary_image: WaltersImage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WaltersImage {
    large_thumb_path: Option<String>,
    small_thumb_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let config = Config::default();
        let provider = WaltersArtMuseumProvider::new(&config);
        assert_eq!(provider.name(), "walters");
        // Disabled due to Cloudflare protection
        assert!(!provider.is_available());
        assert!(!provider.requires_api_key());
    }
}
