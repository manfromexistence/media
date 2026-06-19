//! Random Fox provider implementation.
//!
//! [RandomFox](https://randomfox.ca/)
//!
//! Free API for random fox images - no API key required.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Random Fox provider for fox images.
/// No API key required.
#[derive(Debug)]
pub struct RandomFoxProvider {
    client: HttpClient,
}

impl RandomFoxProvider {
    /// Create a new Random Fox provider.
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

    fn fox_to_asset(&self, data: FoxResponse, index: usize) -> Option<MediaAsset> {
        let id_suffix = data
            .image
            .split('/')
            .next_back()
            .filter(|part| !part.is_empty())
            .map_or_else(|| index.to_string(), ToString::to_string);

        MediaAsset::builder()
            .id(format!("fox_{id_suffix}"))
            .provider(self.name().to_string())
            .media_type(MediaType::Image)
            .title(format!("Fox photo #{}", index + 1))
            .direct_download_url(data.image.clone())
            .preview_url(data.image.clone())
            .source_url(data.link.clone())
            .license(License::Other("RandomFox - Free".to_string()))
            .tags(vec![
                "fox".to_string(),
                "animal".to_string(),
                "wildlife".to_string(),
            ])
            .provider_metadata_entry("randomfox.api_endpoint", randomfox_api_endpoint(self))
            .provider_metadata_entry("randomfox.image_url", data.image)
            .provider_metadata_entry("randomfox.source_link", data.link)
            .provider_metadata_entry(
                "randomfox.license_status",
                "provider-stated-free-unverified",
            )
            .build_or_log()
    }
}

#[async_trait]
impl Provider for RandomFoxProvider {
    fn name(&self) -> &'static str {
        "randomfox"
    }

    fn display_name(&self) -> &'static str {
        "Random Fox"
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
        "https://randomfox.ca"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let count = query.count.min(20); // API returns one at a time, so limit requests
        let mut assets = Vec::with_capacity(count);

        // Make multiple requests to get more images
        for i in 0..count {
            let endpoint = randomfox_api_endpoint(self);
            let response = self.client.get(&endpoint).await?;
            let data: FoxResponse = response.json_or_error().await?;

            if let Some(asset) = self.fox_to_asset(data, i) {
                assets.push(asset);
            }
        }

        let total = assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: total,
            assets,
            providers_searched: vec!["randomfox".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for RandomFoxProvider {
    fn description(&self) -> &'static str {
        "Random fox images - no API key required"
    }

    fn api_key_url(&self) -> &'static str {
        "https://randomfox.ca/"
    }

    fn default_license(&self) -> &'static str {
        "Free for any use"
    }
}

/// API response from RandomFox
#[derive(Debug, Deserialize)]
struct FoxResponse {
    image: String,
    link: String,
}

fn randomfox_api_endpoint(provider: &RandomFoxProvider) -> String {
    format!("{}/floof/", provider.base_url())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let config = Config::default();
        let provider = RandomFoxProvider::new(&config);
        assert_eq!(provider.name(), "randomfox");
        assert!(provider.is_available());
        assert!(!provider.requires_api_key());
    }

    #[test]
    fn randomfox_asset_preserves_provider_provenance() {
        let config = Config::default();
        let provider = RandomFoxProvider::new(&config);
        let asset = provider
            .fox_to_asset(
                FoxResponse {
                    image: "https://randomfox.ca/images/42.jpg".to_string(),
                    link: "https://randomfox.ca/?i=42".to_string(),
                },
                0,
            )
            .expect("fixture should build a randomfox asset");

        let provenance = asset.provenance();

        assert_eq!(asset.id, "fox_42.jpg");
        assert_eq!(provenance.source_url, "https://randomfox.ca/?i=42");
        assert_eq!(
            provenance.provider_metadata.get("randomfox.api_endpoint"),
            Some(&"https://randomfox.ca/floof/".to_string())
        );
        assert_eq!(
            provenance.provider_metadata.get("randomfox.image_url"),
            Some(&"https://randomfox.ca/images/42.jpg".to_string())
        );
        assert_eq!(
            provenance.provider_metadata.get("randomfox.source_link"),
            Some(&"https://randomfox.ca/?i=42".to_string())
        );
        assert_eq!(
            provenance.provider_metadata.get("randomfox.license_status"),
            Some(&"provider-stated-free-unverified".to_string())
        );
        assert!(!provenance.license_known);
        assert!(provenance.type_validation.has_evidence());
        assert!(provenance.type_validation.is_valid());
    }
}
