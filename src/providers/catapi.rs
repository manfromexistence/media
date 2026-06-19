//! The Cat API provider implementation.
//!
//! [The Cat API](https://thecatapi.com/)
//!
//! Free cat images API - 60K+ images, no API key required.

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

/// The Cat API provider for cat images.
/// No API key required, 60K+ images.
#[derive(Debug)]
pub struct CatApiProvider {
    client: HttpClient,
}

impl CatApiProvider {
    /// Create a new Cat API provider.
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

    fn asset_from_image(cat: CatImage, idx: usize) -> Option<MediaAsset> {
        let breeds = cat.breeds.unwrap_or_default();
        let breed_name = breeds
            .first()
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "Cat".to_string());
        let breed_tags: Vec<String> = breeds.iter().map(|b| b.name.clone()).collect();
        let download_url = cat.url.clone();
        let mut metadata = direct_asset_metadata("catapi", &download_url);
        metadata.insert("catapi.asset_id".to_string(), cat.id.clone());
        metadata.insert("catapi.display_breed".to_string(), breed_name.clone());
        if !breed_tags.is_empty() {
            metadata.insert("catapi.breeds".to_string(), breed_tags.join(","));
        }
        if let Some(width) = cat.width {
            metadata.insert("catapi.width".to_string(), width.to_string());
        }
        if let Some(height) = cat.height {
            metadata.insert("catapi.height".to_string(), height.to_string());
        }

        let mut builder = MediaAsset::builder()
            .id(format!("catapi_{}", cat.id))
            .provider("catapi")
            .media_type(MediaType::Image)
            .title(format!("{} photo #{}", breed_name, idx + 1))
            .direct_download_url(download_url.clone())
            .preview_url(download_url.clone())
            .source_url(download_url.clone())
            .license(license_not_provided())
            .maybe_url_inferred_mime_type(mime_type_from_url(MediaType::Image, &download_url))
            .provider_metadata(metadata)
            .tags(
                vec!["cat".to_string(), "animal".to_string(), "pet".to_string()]
                    .into_iter()
                    .chain(breed_tags)
                    .collect(),
            );

        if let (Some(width), Some(height)) = (
            cat.width.and_then(|value| u32::try_from(value).ok()),
            cat.height.and_then(|value| u32::try_from(value).ok()),
        ) {
            builder = builder.dimensions(width, height);
        }

        builder.build_or_log()
    }
}

#[async_trait]
impl Provider for CatApiProvider {
    fn name(&self) -> &'static str {
        "catapi"
    }

    fn display_name(&self) -> &'static str {
        "The Cat API"
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
        "https://api.thecatapi.com/v1"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let count = query.count.min(25); // API max is 25 per request

        let url = format!("{}/images/search?limit={}", self.base_url(), count);

        let response = self.client.get(&url).await?;
        let cats: Vec<CatImage> = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = cats
            .into_iter()
            .enumerate()
            .filter_map(|(idx, cat)| Self::asset_from_image(cat, idx))
            .collect();

        let total = assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: total,
            assets,
            providers_searched: vec!["catapi".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for CatApiProvider {
    fn description(&self) -> &'static str {
        "The Cat API - 60K+ cat images, no API key required"
    }

    fn api_key_url(&self) -> &'static str {
        "https://thecatapi.com/"
    }

    fn default_license(&self) -> &'static str {
        "License not provided by API response"
    }
}

/// Cat API response structures
#[derive(Debug, Deserialize)]
struct CatImage {
    id: String,
    url: String,
    width: Option<i32>,
    height: Option<i32>,
    breeds: Option<Vec<CatBreed>>,
}

#[derive(Debug, Deserialize)]
struct CatBreed {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let config = Config::default();
        let provider = CatApiProvider::new(&config);
        assert_eq!(provider.name(), "catapi");
        assert!(provider.is_available());
        assert!(!provider.requires_api_key());
    }

    #[test]
    fn cat_asset_preserves_direct_source_and_unresolved_license() {
        let asset = CatApiProvider::asset_from_image(
            CatImage {
                id: "abc123".to_string(),
                url: "https://cdn2.thecatapi.com/images/abc123.jpg".to_string(),
                width: Some(640),
                height: Some(480),
                breeds: Some(vec![CatBreed {
                    name: "Tabby".to_string(),
                }]),
            },
            0,
        )
        .expect("fixture cat should build asset");
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
                .get("catapi.source_url_kind")
                .map(String::as_str),
            Some("direct-asset-url")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("catapi.license_evidence")
                .map(String::as_str),
            Some("not-provided-by-api-response")
        );
    }
}
