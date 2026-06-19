//! Openverse provider implementation.
//!
//! [Openverse API Documentation](https://api.openverse.engineering/v1/)
//!
//! Openverse is a search engine for openly-licensed media, providing access to
//! over 700 million images and audio files from various sources.

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Openverse provider for openly-licensed media.
/// Access to 700M+ images and audio from Creative Commons sources.
#[derive(Debug)]
pub struct OpenverseProvider {
    client: HttpClient,
    /// Base URL for API requests (configurable for testing)
    base_url: String,
}

impl OpenverseProvider {
    /// Default base URL for Openverse API.
    const DEFAULT_BASE_URL: &'static str = "https://api.openverse.org/v1";

    /// Create a new Openverse provider.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let client = HttpClient::with_config(
            Self::RATE_LIMIT,
            config.retry_attempts,
            Duration::from_secs(config.timeout_secs),
        )
        .unwrap_or_default();

        Self {
            client,
            base_url: Self::DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Create a new Openverse provider with a custom base URL.
    ///
    /// This is primarily useful for testing with mock servers.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL to use for API requests.
    #[must_use]
    pub fn with_base_url(base_url: &str) -> Self {
        let client = HttpClient::with_config(
            Self::RATE_LIMIT,
            0,                      // No retries for testing
            Duration::from_secs(5), // Short timeout for testing
        )
        .unwrap_or_default();

        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    /// Rate limit: 100 requests per day (anonymous), more with API key
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(100, 86400);

    /// Search for images
    async fn search_images(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = format!("{}/images/", self.base_url);

        // NOTE: Use format=json to force JSON response
        // The API's Accept header negotiation is unreliable
        let params = [
            ("q", query.query.as_str()),
            ("page", &query.page.to_string()),
            ("page_size", &query.count.min(500).to_string()),
            ("format", "json"),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: OpenverseSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .results
            .into_iter()
            .filter_map(|item| {
                let license = Self::parse_license(&item.license, &item.license_version);
                let metadata = Self::provider_metadata(
                    item.source.as_deref(),
                    item.license_url.as_deref(),
                    item.license_version.as_deref(),
                );
                let mime_type = Self::mime_type_from_url(&item.url);
                let media_type = Self::image_media_type_from_url(&item.url);

                if let Some(requested_type) = query.media_type {
                    if media_type != requested_type {
                        return None;
                    }
                }

                MediaAsset::builder()
                    .id(item.id)
                    .provider("openverse")
                    .media_type(media_type)
                    .title(item.title.unwrap_or_else(|| "Openverse Image".to_string()))
                    .direct_download_url(item.url)
                    .maybe_preview_url(item.thumbnail)
                    .source_url(item.foreign_landing_url)
                    .author(item.creator.unwrap_or_default())
                    .author_url(item.creator_url.unwrap_or_default())
                    .license(license)
                    .provider_metadata(metadata)
                    .dimensions(item.width.unwrap_or(0), item.height.unwrap_or(0))
                    .maybe_url_inferred_mime_type(mime_type)
                    .tags(item.tags.into_iter().map(|t| t.name).collect())
                    .build_or_log()
            })
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.result_count,
            assets,
            providers_searched: vec!["openverse".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }

    /// Search for audio
    async fn search_audio(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = format!("{}/audio/", self.base_url);

        // NOTE: Use format=json to force JSON response
        // The API's Accept header negotiation is unreliable
        let params = [
            ("q", query.query.as_str()),
            ("page", &query.page.to_string()),
            ("page_size", &query.count.min(500).to_string()),
            ("format", "json"),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: OpenverseAudioSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .results
            .into_iter()
            .filter_map(|item| {
                let license = Self::parse_license(&item.license, &item.license_version);
                let metadata = Self::provider_metadata(
                    item.source.as_deref(),
                    item.license_url.as_deref(),
                    item.license_version.as_deref(),
                );
                let mime_type = Self::mime_type_from_url(&item.url);

                MediaAsset::builder()
                    .id(item.id)
                    .provider("openverse")
                    .media_type(MediaType::Audio)
                    .title(item.title.unwrap_or_else(|| "Openverse Audio".to_string()))
                    .direct_download_url(item.url)
                    .maybe_preview_url(item.thumbnail)
                    .source_url(item.foreign_landing_url)
                    .author(item.creator.unwrap_or_default())
                    .author_url(item.creator_url.unwrap_or_default())
                    .license(license)
                    .provider_metadata(metadata)
                    .maybe_url_inferred_mime_type(mime_type)
                    .tags(item.tags.into_iter().map(|t| t.name).collect())
                    .build_or_log()
            })
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.result_count,
            assets,
            providers_searched: vec!["openverse".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }

    /// Parse license string into License enum
    fn parse_license(license: &str, _version: &Option<String>) -> License {
        match license.trim().to_ascii_lowercase().as_str() {
            "cc0" => License::Cc0,
            "by" | "cc-by" => License::CcBy,
            "by-sa" | "cc-by-sa" => License::CcBySa,
            "by-nc" | "cc-by-nc" => License::CcByNc,
            "pdm" => License::PublicDomain,
            _ => License::Other(license.to_string()),
        }
    }

    fn provider_metadata(
        source: Option<&str>,
        license_url: Option<&str>,
        license_version: Option<&str>,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        if let Some(source) = source.filter(|value| !value.trim().is_empty()) {
            metadata.insert("openverse.source".to_string(), source.to_string());
        }
        if let Some(license_url) = license_url.filter(|value| !value.trim().is_empty()) {
            metadata.insert("openverse.license_url".to_string(), license_url.to_string());
        }
        if let Some(license_version) = license_version.filter(|value| !value.trim().is_empty()) {
            metadata.insert(
                "openverse.license_version".to_string(),
                license_version.to_string(),
            );
        }
        metadata
    }

    fn mime_type_from_url(url: &str) -> Option<&'static str> {
        let url = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
        if url.ends_with(".jpg") || url.ends_with(".jpeg") {
            Some("image/jpeg")
        } else if url.ends_with(".png") {
            Some("image/png")
        } else if url.ends_with(".gif") {
            Some("image/gif")
        } else if url.ends_with(".webp") {
            Some("image/webp")
        } else if url.ends_with(".mp3") {
            Some("audio/mpeg")
        } else if url.ends_with(".wav") {
            Some("audio/wav")
        } else if url.ends_with(".ogg") {
            Some("audio/ogg")
        } else {
            None
        }
    }

    fn image_media_type_from_url(url: &str) -> MediaType {
        match Self::mime_type_from_url(url) {
            Some("image/gif") => MediaType::Gif,
            _ => MediaType::Image,
        }
    }
}

#[async_trait]
impl Provider for OpenverseProvider {
    fn name(&self) -> &'static str {
        "openverse"
    }

    fn display_name(&self) -> &'static str {
        "Openverse"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image, MediaType::Gif, MediaType::Audio]
    }

    fn requires_api_key(&self) -> bool {
        false // Works without API key (with rate limits)
    }

    fn rate_limit(&self) -> RateLimitConfig {
        Self::RATE_LIMIT
    }

    fn is_available(&self) -> bool {
        true // Always available (no API key required)
    }

    fn base_url(&self) -> &'static str {
        // Note: This returns the default static URL for trait compliance.
        // The actual search method uses self.base_url field which may be customized.
        Self::DEFAULT_BASE_URL
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        match query.media_type {
            Some(MediaType::Audio) => self.search_audio(query).await,
            _ => self.search_images(query).await,
        }
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<MediaAsset>> {
        // Try image first, then audio
        // Openverse API: GET /v1/images/{id} or /v1/audio/{id}

        // Try image endpoint
        let image_url = format!("{}/images/{}/", self.base_url, id);
        if let Ok(response) = self.client.get(&image_url).await {
            if let Ok(item) = response.json_or_error::<OpenverseImageResult>().await {
                let license = Self::parse_license(&item.license, &item.license_version);
                let metadata = Self::provider_metadata(
                    item.source.as_deref(),
                    item.license_url.as_deref(),
                    item.license_version.as_deref(),
                );
                let mime_type = Self::mime_type_from_url(&item.url);
                let media_type = Self::image_media_type_from_url(&item.url);
                return Ok(MediaAsset::builder()
                    .id(item.id)
                    .provider("openverse")
                    .media_type(media_type)
                    .title(item.title.unwrap_or_else(|| "Openverse Image".to_string()))
                    .direct_download_url(item.url)
                    .maybe_preview_url(item.thumbnail)
                    .source_url(item.foreign_landing_url)
                    .author(item.creator.unwrap_or_default())
                    .author_url(item.creator_url.unwrap_or_default())
                    .license(license)
                    .dimensions(item.width.unwrap_or(0), item.height.unwrap_or(0))
                    .provider_metadata(metadata)
                    .maybe_url_inferred_mime_type(mime_type)
                    .tags(item.tags.into_iter().map(|t| t.name).collect())
                    .build_or_log());
            }
        }

        // Try audio endpoint
        let audio_url = format!("{}/audio/{}/", self.base_url, id);
        if let Ok(response) = self.client.get(&audio_url).await {
            if let Ok(item) = response.json_or_error::<OpenverseAudioResult>().await {
                let license = Self::parse_license(&item.license, &item.license_version);
                let metadata = Self::provider_metadata(
                    item.source.as_deref(),
                    item.license_url.as_deref(),
                    item.license_version.as_deref(),
                );
                let mime_type = Self::mime_type_from_url(&item.url);
                return Ok(MediaAsset::builder()
                    .id(item.id)
                    .provider("openverse")
                    .media_type(MediaType::Audio)
                    .title(item.title.unwrap_or_else(|| "Openverse Audio".to_string()))
                    .direct_download_url(item.url)
                    .maybe_preview_url(item.thumbnail)
                    .source_url(item.foreign_landing_url)
                    .author(item.creator.unwrap_or_default())
                    .author_url(item.creator_url.unwrap_or_default())
                    .license(license)
                    .provider_metadata(metadata)
                    .maybe_url_inferred_mime_type(mime_type)
                    .tags(item.tags.into_iter().map(|t| t.name).collect())
                    .build_or_log());
            }
        }

        Ok(None)
    }
}

impl ProviderInfo for OpenverseProvider {
    fn description(&self) -> &'static str {
        "Search engine for openly-licensed media with 700M+ images and audio files"
    }

    fn api_key_url(&self) -> &'static str {
        "https://api.openverse.engineering/v1/"
    }

    fn default_license(&self) -> &'static str {
        "Creative Commons (CC0, CC-BY, CC-BY-SA, etc.)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenverseSearchResponse {
    result_count: usize,
    page_count: usize,
    results: Vec<OpenverseImageResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenverseImageResult {
    id: String,
    title: Option<String>,
    url: String,
    thumbnail: Option<String>,
    foreign_landing_url: String,
    creator: Option<String>,
    creator_url: Option<String>,
    license: String,
    license_version: Option<String>,
    license_url: Option<String>,
    source: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    #[serde(default)]
    tags: Vec<OpenverseTag>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenverseAudioSearchResponse {
    result_count: usize,
    page_count: usize,
    results: Vec<OpenverseAudioResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenverseAudioResult {
    id: String,
    title: Option<String>,
    url: String,
    thumbnail: Option<String>,
    foreign_landing_url: String,
    creator: Option<String>,
    creator_url: Option<String>,
    license: String,
    license_version: Option<String>,
    license_url: Option<String>,
    source: Option<String>,
    duration: Option<u32>,
    #[serde(default)]
    tags: Vec<OpenverseTag>,
}

#[derive(Debug, Deserialize)]
struct OpenverseTag {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = OpenverseProvider::new(&config);

        assert_eq!(provider.name(), "openverse");
        assert_eq!(provider.display_name(), "Openverse");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default_for_testing();
        let provider = OpenverseProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
        assert!(types.contains(&MediaType::Audio));
    }

    #[test]
    fn test_license_parsing() {
        assert!(matches!(
            OpenverseProvider::parse_license("cc0", &None),
            License::Cc0
        ));
        assert!(matches!(
            OpenverseProvider::parse_license("by", &None),
            License::CcBy
        ));
        assert!(matches!(
            OpenverseProvider::parse_license("pdm", &None),
            License::PublicDomain
        ));
    }

    #[test]
    fn openverse_license_parsing_accepts_fixture_shaped_cc_labels() {
        assert!(matches!(
            OpenverseProvider::parse_license("cc-by", &Some("4.0".to_string())),
            License::CcBy
        ));
        assert!(matches!(
            OpenverseProvider::parse_license("cc-by-sa", &Some("4.0".to_string())),
            License::CcBySa
        ));
        assert!(matches!(
            OpenverseProvider::parse_license("CC-BY-NC", &Some("4.0".to_string())),
            License::CcByNc
        ));
    }

    #[test]
    fn gif_image_urls_are_typed_as_gif_assets() {
        let media_type = OpenverseProvider::image_media_type_from_url(
            "https://example.com/animated.gif?token=1",
        );

        assert_eq!(media_type, MediaType::Gif);
        assert_eq!(
            OpenverseProvider::image_media_type_from_url("https://example.com/photo.jpg"),
            MediaType::Image
        );
    }
}
