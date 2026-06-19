//! Poly Haven provider implementation.
//!
//! [Poly Haven API](https://polyhaven.com/api)
//!
//! Provides access to 1,000+ 3D models, 2,000+ textures, and 700+ HDRIs - all CC0.

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{
    DownloadUrlKind, License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult,
};

/// Poly Haven provider for 3D assets, textures, and HDRIs.
/// Access to 1K+ models, 2K+ textures, 700+ HDRIs - all CC0 licensed.
#[derive(Debug)]
pub struct PolyHavenProvider {
    client: HttpClient,
}

impl PolyHavenProvider {
    /// Create a new Poly Haven provider.
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
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(60, 60);

    /// Get asset type string for API
    /// Note: Textures are mapped to Image since MediaType doesn't have a Texture variant
    fn asset_type_for_media(media_type: Option<MediaType>) -> &'static str {
        match media_type {
            Some(MediaType::Model3D) => "models",
            Some(MediaType::Image) => "all", // Includes HDRIs and textures
            _ => "all",
        }
    }
}

#[async_trait]
impl Provider for PolyHavenProvider {
    fn name(&self) -> &'static str {
        "polyhaven"
    }

    fn display_name(&self) -> &'static str {
        "Poly Haven"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        // Note: Poly Haven has HDRIs, textures, and 3D models - textures mapped to Image
        &[MediaType::Model3D, MediaType::Image]
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
        "https://api.polyhaven.com"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        // Poly Haven doesn't have a search endpoint, but lists all assets
        // We'll fetch all and filter client-side
        let asset_type = Self::asset_type_for_media(query.media_type);
        let url = format!("{}/assets?t={}", self.base_url(), asset_type);

        let response = self.client.get(&url).await?;

        let assets_map: HashMap<String, PolyHavenAsset> = response.json_or_error().await?;

        let query_lower = query.query.to_lowercase();
        let start = (query.page - 1) * query.count;

        let filtered_assets: Vec<MediaAsset> = assets_map
            .into_iter()
            .filter(|(id, asset)| {
                if query.query.is_empty() || query.query == "*" {
                    return true;
                }
                id.to_lowercase().contains(&query_lower)
                    || asset.name.to_lowercase().contains(&query_lower)
                    || asset
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
                    || asset
                        .categories
                        .iter()
                        .any(|c| c.to_lowercase().contains(&query_lower))
            })
            .skip(start)
            .take(query.count)
            .filter_map(|(id, asset)| {
                let media_type = match asset.r#type {
                    0 => MediaType::Image, // HDRI
                    1 => MediaType::Image, // Texture (mapped to Image)
                    2 => MediaType::Model3D,
                    _ => MediaType::Image,
                };

                let preview_url = format!(
                    "https://cdn.polyhaven.com/asset_img/thumbs/{}.png?height=256",
                    id
                );
                let download_url = format!("https://polyhaven.com/a/{}", id);

                MediaAsset::builder()
                    .id(id.clone())
                    .provider("polyhaven")
                    .media_type(media_type)
                    .title(asset.name)
                    .download_url(download_url)
                    .download_url_kind(DownloadUrlKind::LandingPage)
                    .preview_url(preview_url)
                    .source_url(format!("https://polyhaven.com/a/{}", id))
                    .author(asset.authors.into_keys().collect::<Vec<_>>().join(", "))
                    .license(License::Cc0)
                    .tags(asset.tags)
                    .build_or_log()
            })
            .collect();

        let total_count = filtered_assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count,
            assets: filtered_assets,
            providers_searched: vec!["polyhaven".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for PolyHavenProvider {
    fn description(&self) -> &'static str {
        "Poly Haven - 1K+ 3D models, 2K+ textures, 700+ HDRIs - all CC0"
    }

    fn api_key_url(&self) -> &'static str {
        "https://polyhaven.com/api"
    }

    fn default_license(&self) -> &'static str {
        "CC0 (Public Domain)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct PolyHavenAsset {
    name: String,
    r#type: i32,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    authors: HashMap<String, String>,
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
        let provider = PolyHavenProvider::new(&config);

        assert_eq!(provider.name(), "polyhaven");
        assert_eq!(provider.display_name(), "Poly Haven");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default();
        let provider = PolyHavenProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Model3D));
        assert!(types.contains(&MediaType::Image)); // Textures and HDRIs mapped to Image
    }
}
