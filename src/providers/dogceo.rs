//! Dog CEO provider implementation.
//!
//! [Dog CEO](https://dog.ceo/)
//!
//! Free API for random dog images - 20K+ images, no API key required.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::provenance::{
    direct_asset_metadata, license_not_provided, mime_type_from_url,
};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Dog CEO provider for random dog images.
/// No API key required, 20K+ images available.
#[derive(Debug)]
pub struct DogCeoProvider {
    client: HttpClient,
}

impl DogCeoProvider {
    /// Create a new Dog CEO provider.
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

    /// Rate limit: Generous
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(100, 60);
}

#[async_trait]
impl Provider for DogCeoProvider {
    fn name(&self) -> &'static str {
        "dogceo"
    }

    fn display_name(&self) -> &'static str {
        "Dog CEO"
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
        "https://dog.ceo/api"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let count = query.count.min(50);
        let query_lower = query.query.to_lowercase();

        // Check if query is a specific breed
        let url = if query_lower.contains("husky")
            || query_lower.contains("labrador")
            || query_lower.contains("poodle")
            || query_lower.contains("bulldog")
            || query_lower.contains("beagle")
            || query_lower.contains("retriever")
            || query_lower.contains("shepherd")
            || query_lower.contains("terrier")
        {
            // Try breed-specific endpoint
            let breed = query_lower.split_whitespace().next().unwrap_or("random");
            format!(
                "{}/breed/{}/images/random/{}",
                self.base_url(),
                breed,
                count
            )
        } else {
            // Random images
            format!("{}/breeds/image/random/{}", self.base_url(), count)
        };

        let response = self.client.get(&url).await?;
        let data: DogCeoResponse = response.json_or_error().await?;

        if data.status != "success" {
            // Fallback to random if breed not found
            let fallback_url = format!("{}/breeds/image/random/{}", self.base_url(), count);
            let response = self.client.get(&fallback_url).await?;
            let data: DogCeoResponse = response.json_or_error().await?;
            return self.build_result(&data.message, query);
        }

        self.build_result(&data.message, query)
    }
}

impl DogCeoProvider {
    fn build_result(&self, images: &[String], query: &SearchQuery) -> Result<SearchResult> {
        let assets: Vec<MediaAsset> = images
            .iter()
            .enumerate()
            .map(|(idx, url)| {
                // Extract breed from URL (e.g., .../breeds/husky/image.jpg)
                let breed = url
                    .split("/breeds/")
                    .nth(1)
                    .and_then(|s| s.split('/').next())
                    .unwrap_or("dog")
                    .replace('-', " ");

                let id = format!("dog_{}", url.split('/').last().unwrap_or(&idx.to_string()));
                let mut metadata = direct_asset_metadata("dogceo", url);
                metadata.insert("dogceo.breed".to_string(), breed.clone());
                metadata.insert("dogceo.query".to_string(), query.query.clone());

                MediaAsset::builder()
                    .id(id)
                    .provider("dogceo")
                    .media_type(MediaType::Image)
                    .title(format!("{} dog photo", breed))
                    .direct_download_url(url.clone())
                    .preview_url(url.clone())
                    .source_url(url.clone())
                    .license(license_not_provided())
                    .maybe_url_inferred_mime_type(mime_type_from_url(MediaType::Image, url))
                    .provider_metadata(metadata)
                    .tags(vec!["dog".to_string(), breed.clone(), "animal".to_string()])
                    .build_or_log()
            })
            .flatten()
            .collect();

        let total = assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: total,
            assets,
            providers_searched: vec!["dogceo".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for DogCeoProvider {
    fn description(&self) -> &'static str {
        "Free random dog images API - 20K+ images, no API key required"
    }

    fn api_key_url(&self) -> &'static str {
        "https://dog.ceo/"
    }

    fn default_license(&self) -> &'static str {
        "License not provided by API response"
    }
}

/// API response from Dog CEO
#[derive(Debug, Deserialize)]
struct DogCeoResponse {
    message: Vec<String>,
    status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let config = Config::default();
        let provider = DogCeoProvider::new(&config);
        assert_eq!(provider.name(), "dogceo");
        assert!(provider.is_available());
        assert!(!provider.requires_api_key());
    }

    #[test]
    fn dog_asset_preserves_direct_source_and_unresolved_license() {
        let config = Config::default();
        let provider = DogCeoProvider::new(&config);
        let query = SearchQuery::for_type("husky", MediaType::Image).count(1);
        let result = provider
            .build_result(
                &["https://images.dog.ceo/breeds/husky/n02110185_1469.jpg".to_string()],
                &query,
            )
            .expect("fixture dog should build result");
        let asset = &result.assets[0];
        let provenance = asset.provenance();

        assert_eq!(asset.download_url, asset.source_url);
        assert_eq!(
            asset.download_url_kind,
            crate::types::DownloadUrlKind::DirectFile
        );
        assert_eq!(asset.mime_type.as_deref(), Some("image/jpeg"));
        assert_eq!(
            asset.mime_evidence_source,
            Some(crate::types::MimeEvidenceSource::UrlInferred)
        );
        assert!(provenance.type_validation.is_valid());
        assert!(!provenance.license_known);
        assert_eq!(
            asset
                .provider_metadata
                .get("dogceo.source_url_kind")
                .map(String::as_str),
            Some("direct-asset-url")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("dogceo.license_evidence")
                .map(String::as_str),
            Some("not-provided-by-api-response")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("dogceo.breed")
                .map(String::as_str),
            Some("husky")
        );
    }
}
