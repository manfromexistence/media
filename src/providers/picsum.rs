//! Lorem Picsum provider implementation.
//!
//! [Lorem Picsum](https://picsum.photos)
//!
//! Provides random placeholder images from Unsplash - no API key required.

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{
    License, MediaAsset, MediaType, MimeEvidenceSource, RateLimitConfig, SearchQuery, SearchResult,
};

/// Lorem Picsum provider for placeholder images.
/// No API key required, unlimited access to random images.
#[derive(Debug)]
pub struct LoremPicsumProvider {
    client: HttpClient,
}

impl LoremPicsumProvider {
    /// Create a new Lorem Picsum provider.
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
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(1000, 60);

    const LICENSE_STATEMENT: &'static str =
        "Not provided by Lorem Picsum API; verify original photo rights at source";

    fn provider_license() -> License {
        License::Other(Self::LICENSE_STATEMENT.to_string())
    }

    fn asset_from_api_image(img: PicsumImage) -> Option<MediaAsset> {
        let preview_url = format!("https://picsum.photos/id/{}/400/300", img.id);
        let provider_page = format!("https://picsum.photos/id/{}/info", img.id);
        let provider_metadata = HashMap::from([
            ("picsum.id".to_string(), img.id.clone()),
            ("picsum.author".to_string(), img.author.clone()),
            ("picsum.provider_page".to_string(), provider_page),
            (
                "picsum.api_download_url".to_string(),
                img.download_url.clone(),
            ),
            ("picsum.source_url".to_string(), img.url.clone()),
            (
                "picsum.license_status".to_string(),
                "not-provided-by-api".to_string(),
            ),
        ]);

        MediaAsset::builder()
            .id(img.id)
            .provider("picsum")
            .media_type(MediaType::Image)
            .title(format!("Photo by {}", img.author))
            .direct_download_url(img.download_url)
            .preview_url(preview_url)
            .source_url(img.url)
            .author(img.author)
            .license(Self::provider_license())
            .mime_type_with_evidence("image/jpeg", MimeEvidenceSource::Defaulted)
            .provider_metadata(provider_metadata)
            .dimensions(img.width, img.height)
            .build_or_log()
    }
}

#[async_trait]
impl Provider for LoremPicsumProvider {
    fn name(&self) -> &'static str {
        "picsum"
    }

    fn display_name(&self) -> &'static str {
        "Lorem Picsum"
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
        "https://picsum.photos"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = format!("{}/v2/list", self.base_url());

        let limit = query.count.min(100).to_string();
        let page = query.page.to_string();

        let params = [("limit", limit.as_str()), ("page", page.as_str())];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let images: Vec<PicsumImage> = response.json_or_error().await?;

        // Filter by query if provided (search by author name since that's all we have)
        let query_lower = query.query.to_lowercase();
        let filtered_images: Vec<_> = if query.query.is_empty() || query.query == "*" {
            images
        } else {
            images
                .into_iter()
                .filter(|img| img.author.to_lowercase().contains(&query_lower))
                .collect()
        };

        let assets: Vec<MediaAsset> = filtered_images
            .into_iter()
            .filter_map(Self::asset_from_api_image)
            .collect();

        let total = assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: total,
            assets,
            providers_searched: vec!["picsum".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for LoremPicsumProvider {
    fn description(&self) -> &'static str {
        "Beautiful placeholder images with source metadata from Lorem Picsum, no API key required"
    }

    fn api_key_url(&self) -> &'static str {
        "https://picsum.photos/"
    }

    fn default_license(&self) -> &'static str {
        Self::LICENSE_STATEMENT
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PicsumImage {
    id: String,
    author: String,
    width: u32,
    height: u32,
    url: String,
    download_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = LoremPicsumProvider::new(&config);

        assert_eq!(provider.name(), "picsum");
        assert_eq!(provider.display_name(), "Lorem Picsum");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default_for_testing();
        let provider = LoremPicsumProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
    }

    #[test]
    fn provider_info_does_not_claim_verified_unsplash_license() {
        let config = Config::default_for_testing();
        let provider = LoremPicsumProvider::new(&config);

        assert_ne!(provider.default_license(), "Unsplash License");
        assert!(
            provider
                .default_license()
                .to_ascii_lowercase()
                .contains("not provided")
        );
    }

    #[test]
    fn asset_from_api_image_preserves_picsum_download_and_unverified_license() {
        let image = PicsumImage {
            id: "42".to_string(),
            author: "Ada Lovelace".to_string(),
            width: 1200,
            height: 800,
            url: "https://unsplash.com/photos/example".to_string(),
            download_url: "https://picsum.photos/id/42/1200/800".to_string(),
        };

        let asset = LoremPicsumProvider::asset_from_api_image(image)
            .expect("fixture image should map to an asset");
        let provenance = asset.provenance();

        assert_eq!(asset.download_url, "https://picsum.photos/id/42/1200/800");
        assert_eq!(asset.source_url, "https://unsplash.com/photos/example");
        assert_eq!(asset.mime_type.as_deref(), Some("image/jpeg"));
        assert_eq!(
            asset.mime_evidence_source,
            Some(crate::types::MimeEvidenceSource::Defaulted)
        );
        assert!(!provenance.license_known);
        assert_eq!(
            asset
                .provider_metadata
                .get("picsum.license_status")
                .map(String::as_str),
            Some("not-provided-by-api")
        );
    }
}
