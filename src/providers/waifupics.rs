//! Waifu.pics provider - Anime images and GIFs.
//!
//! Waifu.pics is a free anime image API with:
//! - Unlimited anime images and GIFs
//! - Multiple categories (waifu, neko, shinobu, megumin, etc.)
//! - SFW and NSFW categories (we use SFW only)
//! - Bulk endpoint for multiple images at once
//! - No API key required
//!
//! API: <https://waifu.pics/docs>

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::provenance::{
    direct_asset_metadata, license_not_provided, mime_type_from_url,
};
use crate::providers::traits::Provider;
use crate::types::{MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Waifu.pics provider for anime images and GIFs.
#[derive(Debug)]
pub struct WaifuPicsProvider {
    client: HttpClient,
}

/// API response for bulk images.
#[derive(Debug, Deserialize)]
struct WaifuBulkResponse {
    files: Vec<String>,
}

/// SFW categories available.
const SFW_CATEGORIES: &[&str] = &[
    "waifu", "neko", "shinobu", "megumin", "bully", "cuddle", "cry", "hug", "awoo", "kiss", "lick",
    "pat", "smug", "bonk", "yeet", "blush", "smile", "wave", "highfive", "handhold", "nom", "bite",
    "glomp", "slap", "kill", "kick", "happy", "wink", "poke", "dance", "cringe",
];

impl WaifuPicsProvider {
    /// Create a new Waifu.pics provider.
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

    /// Rate limit: generous (no official limit)
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(100, 60);

    /// Map search query to best matching category.
    fn map_query_to_category(query: &str) -> &'static str {
        let query_lower = query.to_lowercase();

        // Direct category matches
        for &cat in SFW_CATEGORIES {
            if query_lower.contains(cat) {
                return cat;
            }
        }

        // Keyword mappings
        if query_lower.contains("cat") || query_lower.contains("kitty") {
            return "neko";
        }
        if query_lower.contains("anime") || query_lower.contains("girl") {
            return "waifu";
        }
        if query_lower.contains("gif") || query_lower.contains("react") {
            return "smile";
        }
        if query_lower.contains("cute") {
            return "pat";
        }
        if query_lower.contains("love") || query_lower.contains("heart") {
            return "hug";
        }

        // Default
        "waifu"
    }

    /// Determine if URL is a GIF.
    fn is_gif(url: &str) -> bool {
        url.split('?')
            .next()
            .unwrap_or(url)
            .to_lowercase()
            .ends_with(".gif")
    }

    fn asset_from_url(category: &str, index: usize, url: String) -> Option<MediaAsset> {
        let is_gif = Self::is_gif(&url);
        let media_type = if is_gif {
            MediaType::Gif
        } else {
            MediaType::Image
        };
        let format = if is_gif { "gif" } else { "image" };
        let id = format!("waifupics_{}_{}", category, index);
        let mut metadata = direct_asset_metadata("waifupics", &url);
        metadata.insert("waifupics.category".to_string(), category.to_string());
        metadata.insert("waifupics.format".to_string(), format.to_string());
        metadata.insert("waifupics.safety_scope".to_string(), "sfw".to_string());

        MediaAsset::builder()
            .id(id)
            .provider("waifupics")
            .title(format!("{} anime {}", category, format))
            .media_type(media_type)
            .direct_download_url(url.clone())
            .preview_url(url.clone())
            .source_url(url.clone())
            .license(license_not_provided())
            .maybe_url_inferred_mime_type(mime_type_from_url(media_type, &url))
            .provider_metadata(metadata)
            .build_or_log()
    }
}

#[async_trait]
impl Provider for WaifuPicsProvider {
    fn name(&self) -> &'static str {
        "waifupics"
    }

    fn display_name(&self) -> &'static str {
        "Waifu.pics"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image, MediaType::Gif]
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
        "https://api.waifu.pics"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let category = Self::map_query_to_category(&query.query);
        let count = query.count.min(30); // API limit

        // Use bulk endpoint for multiple results
        let url = format!("{}/many/sfw/{}", self.base_url(), category);

        let response = self.client.post_json(&url, &serde_json::json!({})).await?;

        let bulk: WaifuBulkResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = bulk
            .files
            .into_iter()
            .take(count)
            .enumerate()
            .filter_map(|(i, url)| Self::asset_from_url(category, i, url))
            .collect();

        let total = assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: total,
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
        let provider = WaifuPicsProvider::new(&config);
        assert_eq!(provider.name(), "waifupics");
        assert_eq!(provider.display_name(), "Waifu.pics");
        assert_eq!(
            provider.supported_media_types(),
            &[MediaType::Image, MediaType::Gif]
        );
        assert!(provider.is_available());
        assert!(!provider.requires_api_key());
    }

    #[test]
    fn test_category_mapping() {
        assert_eq!(WaifuPicsProvider::map_query_to_category("neko cat"), "neko");
        assert_eq!(
            WaifuPicsProvider::map_query_to_category("anime girl"),
            "waifu"
        );
        assert_eq!(WaifuPicsProvider::map_query_to_category("hug love"), "hug");
        assert_eq!(WaifuPicsProvider::map_query_to_category("random"), "waifu");
    }

    #[test]
    fn test_gif_detection() {
        assert!(WaifuPicsProvider::is_gif("https://example.com/image.gif"));
        assert!(WaifuPicsProvider::is_gif("https://example.com/IMAGE.GIF"));
        assert!(WaifuPicsProvider::is_gif(
            "https://example.com/image.gif?token=fixture"
        ));
        assert!(!WaifuPicsProvider::is_gif("https://example.com/image.png"));
    }

    #[test]
    fn image_asset_preserves_direct_source_and_unresolved_license() {
        let asset = WaifuPicsProvider::asset_from_url(
            "neko",
            0,
            "https://i.waifu.pics/example.png".to_string(),
        )
        .expect("fixture image should build asset");
        let provenance = asset.provenance();

        assert_eq!(asset.media_type, MediaType::Image);
        assert_eq!(asset.download_url, asset.source_url);
        assert_eq!(
            asset.download_url_kind,
            crate::types::DownloadUrlKind::DirectFile
        );
        assert_eq!(asset.mime_type.as_deref(), Some("image/png"));
        assert!(provenance.type_validation.is_valid());
        assert!(!provenance.license_known);
        assert_eq!(
            asset
                .provider_metadata
                .get("waifupics.source_url_kind")
                .map(String::as_str),
            Some("direct-asset-url")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("waifupics.license_evidence")
                .map(String::as_str),
            Some("not-provided-by-api-response")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("waifupics.category")
                .map(String::as_str),
            Some("neko")
        );
    }

    #[test]
    fn gif_asset_uses_gif_media_type_and_mime() {
        let asset = WaifuPicsProvider::asset_from_url(
            "smile",
            1,
            "https://i.waifu.pics/example.GIF".to_string(),
        )
        .expect("fixture gif should build asset");

        assert_eq!(asset.media_type, MediaType::Gif);
        assert_eq!(asset.mime_type.as_deref(), Some("image/gif"));
        assert!(asset.validate_type_metadata().is_valid());
    }

    #[test]
    fn gif_asset_with_query_string_uses_gif_media_type_and_mime() {
        let asset = WaifuPicsProvider::asset_from_url(
            "smile",
            2,
            "https://i.waifu.pics/example.gif?token=fixture".to_string(),
        )
        .expect("fixture gif with query string should build asset");

        assert_eq!(asset.media_type, MediaType::Gif);
        assert_eq!(asset.mime_type.as_deref(), Some("image/gif"));
        assert!(asset.validate_type_metadata().is_valid());
    }
}
