//! Nekos.best provider - High-quality anime images and GIFs.
//!
//! Nekos.best offers:
//! - High-quality anime images and GIFs
//! - Multiple categories (neko, kitsune, waifu, husbando, etc.)
//! - Proper API with pagination
//! - No API key required
//! - Artist attribution included
//!
//! API: <https://docs.nekos.best/>

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::HttpClient;
use crate::providers::provenance::{
    direct_asset_metadata, license_not_provided, mime_type_from_url,
};
use crate::providers::traits::Provider;
use crate::types::{MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Nekos.best anime image provider.
#[derive(Debug)]
pub struct NekosBestProvider {
    client: HttpClient,
}

/// Individual result from API.
#[derive(Debug, Deserialize)]
struct NekoResult {
    url: String,
    #[serde(default)]
    artist_name: Option<String>,
    #[serde(default)]
    source_url: Option<String>,
}

/// API response.
#[derive(Debug, Deserialize)]
struct NekoResponse {
    results: Vec<NekoResult>,
}

/// Available image categories.
const IMAGE_CATEGORIES: &[&str] = &["neko", "kitsune", "waifu", "husbando"];

/// Available GIF categories (reactions).
const GIF_CATEGORIES: &[&str] = &[
    "baka",
    "bite",
    "blush",
    "bored",
    "cry",
    "cuddle",
    "dance",
    "facepalm",
    "feed",
    "handhold",
    "handshake",
    "happy",
    "highfive",
    "hug",
    "kick",
    "kiss",
    "laugh",
    "lurk",
    "nod",
    "nom",
    "nope",
    "pat",
    "peck",
    "poke",
    "pout",
    "punch",
    "shoot",
    "shrug",
    "slap",
    "sleep",
    "smile",
    "smug",
    "stare",
    "think",
    "thumbsup",
    "tickle",
    "wave",
    "wink",
    "yawn",
    "yeet",
];

impl NekosBestProvider {
    /// Create a new Nekos.best provider.
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

    /// Rate limit: generous
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(100, 60);

    /// Map search query to categories.
    fn map_query_to_categories(query: &str, media_type: Option<MediaType>) -> Vec<&'static str> {
        let query_lower = query.to_lowercase();
        let requested_type = match media_type {
            Some(MediaType::Video) => Some(MediaType::Gif),
            other => other,
        };
        let mut categories = Vec::new();

        // Check for direct category matches
        for &cat in IMAGE_CATEGORIES {
            if query_lower.contains(cat) {
                Self::push_allowed_category(&mut categories, cat, requested_type);
            }
        }
        for &cat in GIF_CATEGORIES {
            if query_lower.contains(cat) {
                Self::push_allowed_category(&mut categories, cat, requested_type);
            }
        }

        // Keyword mappings
        if query_lower.contains("cat") || query_lower.contains("kitty") {
            Self::push_allowed_category(&mut categories, "neko", requested_type);
        }
        if query_lower.contains("fox") {
            Self::push_allowed_category(&mut categories, "kitsune", requested_type);
        }
        if query_lower.contains("anime") || query_lower.contains("girl") {
            Self::push_allowed_category(&mut categories, "waifu", requested_type);
        }
        if query_lower.contains("boy") || query_lower.contains("guy") {
            Self::push_allowed_category(&mut categories, "husbando", requested_type);
        }
        if query_lower.contains("hug") || query_lower.contains("love") {
            Self::push_allowed_category(&mut categories, "hug", requested_type);
        }
        if query_lower.contains("cute") || query_lower.contains("happy") {
            Self::push_allowed_category(&mut categories, "happy", requested_type);
        }

        // If no matches, default based on preference
        if categories.is_empty() {
            match requested_type {
                Some(MediaType::Gif) => {
                    Self::push_allowed_category(&mut categories, "smile", requested_type);
                    Self::push_allowed_category(&mut categories, "wave", requested_type);
                }
                Some(MediaType::Image) | None => {
                    Self::push_allowed_category(&mut categories, "neko", requested_type);
                    Self::push_allowed_category(&mut categories, "waifu", requested_type);
                }
                _ => {
                    Self::push_allowed_category(&mut categories, "neko", requested_type);
                    Self::push_allowed_category(&mut categories, "waifu", requested_type);
                }
            }
        }

        categories.into_iter().take(3).collect()
    }

    /// Check if category is a GIF category.
    fn is_gif_category(category: &str) -> bool {
        GIF_CATEGORIES.contains(&category)
    }

    fn push_allowed_category(
        categories: &mut Vec<&'static str>,
        category: &'static str,
        requested_type: Option<MediaType>,
    ) {
        let allowed = match requested_type {
            Some(MediaType::Gif) => Self::is_gif_category(category),
            Some(MediaType::Image) => IMAGE_CATEGORIES.contains(&category),
            _ => true,
        };

        if allowed && !categories.contains(&category) {
            categories.push(category);
        }
    }

    fn asset_from_result(category: &str, index: usize, result: NekoResult) -> Option<MediaAsset> {
        let is_gif = Self::is_gif_category(category);
        let media_type = if is_gif {
            MediaType::Gif
        } else {
            MediaType::Image
        };
        let format = if is_gif { "gif" } else { "image" };
        let id = format!("nekosbest_{}_{}", category, index);
        let mut metadata = direct_asset_metadata("nekosbest", &result.url);
        let (source_url, source_url_kind) = match result.source_url {
            Some(source_url) => (source_url, "provider-source-url"),
            None => (result.url.clone(), "direct-asset-url"),
        };

        metadata.insert("nekosbest.category".to_string(), category.to_string());
        metadata.insert("nekosbest.format".to_string(), format.to_string());
        metadata.insert("nekosbest.source_url".to_string(), source_url.clone());
        metadata.insert(
            "nekosbest.source_url_kind".to_string(),
            source_url_kind.to_string(),
        );

        let mut builder = MediaAsset::builder()
            .id(id)
            .provider("nekosbest")
            .title(format!("{} anime {}", category, format))
            .media_type(media_type)
            .direct_download_url(result.url.clone())
            .preview_url(result.url.clone())
            .source_url(source_url)
            .license(license_not_provided())
            .maybe_url_inferred_mime_type(mime_type_from_url(media_type, &result.url))
            .provider_metadata(metadata);

        if let Some(artist) = result.artist_name {
            builder = builder.author(artist);
        }

        builder.build_or_log()
    }
}

#[async_trait]
impl Provider for NekosBestProvider {
    fn name(&self) -> &'static str {
        "nekosbest"
    }

    fn display_name(&self) -> &'static str {
        "Nekos.best"
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
        // Disabled: nekos.best has Cloudflare bot detection
        false
    }

    fn base_url(&self) -> &'static str {
        "https://nekos.best"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let categories = Self::map_query_to_categories(&query.query, query.media_type);
        let per_category = (query.count / categories.len().max(1)).max(1).min(20);

        let mut all_assets = Vec::new();

        for category in &categories {
            let url = format!(
                "{}/api/v2/{}?amount={}",
                self.base_url(),
                category,
                per_category
            );

            // Debug: log URL
            tracing::debug!("Nekos.best fetching: {}", url);

            let response = match self.client.get(&url).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!("Nekos.best error for {}: {}", category, e);
                    continue;
                }
            };

            let text = match response.text().await {
                Ok(t) => t,
                Err(e) => {
                    tracing::debug!("Nekos.best text error: {}", e);
                    continue;
                }
            };

            let data: NekoResponse = match serde_json::from_str(&text) {
                Ok(d) => d,
                Err(e) => {
                    tracing::debug!(
                        "Nekos.best parse error: {} - text: {}",
                        e,
                        &text[..text.len().min(200)]
                    );
                    continue;
                }
            };

            for (i, result) in data.results.into_iter().enumerate() {
                if let Some(asset) = Self::asset_from_result(category, i, result) {
                    all_assets.push(asset);
                }
            }
        }

        let total = all_assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: total,
            assets: all_assets.into_iter().take(query.count).collect(),
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
        let provider = NekosBestProvider::new(&config);
        assert_eq!(provider.name(), "nekosbest");
        assert_eq!(provider.display_name(), "Nekos.best");
        assert_eq!(
            provider.supported_media_types(),
            &[MediaType::Image, MediaType::Gif]
        );
        // NOTE: Provider is disabled due to Cloudflare bot detection
        assert!(!provider.is_available());
        assert!(!provider.requires_api_key());
    }

    #[test]
    fn test_category_mapping() {
        let cats = NekosBestProvider::map_query_to_categories("neko cat", None);
        assert!(cats.contains(&"neko"));

        let cats = NekosBestProvider::map_query_to_categories("anime girl", None);
        assert!(cats.contains(&"waifu"));
    }

    #[test]
    fn gif_search_preference_does_not_return_image_categories() {
        let cats = NekosBestProvider::map_query_to_categories("neko cat", Some(MediaType::Gif));

        assert!(
            !cats
                .iter()
                .any(|category| IMAGE_CATEGORIES.contains(category))
        );
        assert!(
            cats.iter()
                .all(|category| GIF_CATEGORIES.contains(category))
        );
    }

    #[test]
    fn image_search_preference_does_not_return_gif_categories() {
        let cats = NekosBestProvider::map_query_to_categories("hug love", Some(MediaType::Image));

        assert!(
            !cats
                .iter()
                .any(|category| GIF_CATEGORIES.contains(category))
        );
        assert!(
            cats.iter()
                .all(|category| IMAGE_CATEGORIES.contains(category))
        );
    }

    #[test]
    fn gif_category_asset_preserves_gif_type_mime_and_provider_metadata() {
        let asset = NekosBestProvider::asset_from_result(
            "hug",
            1,
            NekoResult {
                url: "https://nekos.best/api/v2/hug/fixture.gif".to_string(),
                artist_name: Some("Fixture Artist".to_string()),
                source_url: Some("https://example.test/source".to_string()),
            },
        )
        .expect("fixture gif should build asset");

        assert_eq!(asset.media_type, MediaType::Gif);
        assert_eq!(asset.mime_type.as_deref(), Some("image/gif"));
        assert_eq!(asset.author.as_deref(), Some("Fixture Artist"));
        assert!(asset.validate_type_metadata().is_valid());
        assert!(!asset.provenance().license_known);
        assert_eq!(
            asset
                .provider_metadata
                .get("nekosbest.category")
                .map(String::as_str),
            Some("hug")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("nekosbest.source_url")
                .map(String::as_str),
            Some("https://example.test/source")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("nekosbest.source_url_kind")
                .map(String::as_str),
            Some("provider-source-url")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("nekosbest.license_evidence")
                .map(String::as_str),
            Some(crate::providers::provenance::LICENSE_EVIDENCE_NOT_PROVIDED)
        );
    }

    #[test]
    fn image_category_asset_preserves_image_type_and_direct_source_metadata() {
        let asset = NekosBestProvider::asset_from_result(
            "neko",
            0,
            NekoResult {
                url: "https://nekos.best/api/v2/neko/fixture.png".to_string(),
                artist_name: None,
                source_url: None,
            },
        )
        .expect("fixture image should build asset");

        assert_eq!(asset.media_type, MediaType::Image);
        assert_eq!(asset.mime_type.as_deref(), Some("image/png"));
        assert_eq!(asset.source_url, asset.download_url);
        assert!(asset.validate_type_metadata().is_valid());
        assert_eq!(
            asset
                .provider_metadata
                .get("nekosbest.source_url_kind")
                .map(String::as_str),
            Some("direct-asset-url")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("nekosbest.format")
                .map(String::as_str),
            Some("image")
        );
    }
}
