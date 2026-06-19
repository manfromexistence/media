//! Pixabay provider implementation.
//!
//! [Pixabay API Documentation](https://pixabay.com/api/docs/)

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Pixabay provider for free images and videos.
#[derive(Debug)]
pub struct PixabayProvider {
    api_key: Option<String>,
    client: HttpClient,
}

impl PixabayProvider {
    /// Create a new Pixabay provider.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let client = HttpClient::with_config(
            Self::RATE_LIMIT,
            config.retry_attempts,
            Duration::from_secs(config.timeout_secs),
        )
        .unwrap_or_default();

        Self {
            api_key: config.pixabay_api_key.clone(),
            client,
        }
    }

    /// Rate limit: 100 requests per minute (generous for free tier).
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(100, 60);

    fn asset_from_image_hit(hit: PixabayHit) -> Option<MediaAsset> {
        let media_type = if hit.hit_type == "vector/svg" {
            MediaType::Vector
        } else {
            MediaType::Image
        };
        let download_url = hit
            .large_image_url
            .clone()
            .unwrap_or_else(|| hit.web_format_url.clone());
        let tags: Vec<String> = hit.tags.split(", ").map(|s| s.trim().to_string()).collect();
        let provider_metadata = Self::image_metadata(&hit, &download_url);

        MediaAsset::builder()
            .id(hit.id.to_string())
            .provider("pixabay")
            .media_type(media_type)
            .title(
                tags.first()
                    .cloned()
                    .unwrap_or_else(|| format!("Pixabay Image {}", hit.id)),
            )
            .direct_download_url(download_url.clone())
            .preview_url(hit.preview_url.clone())
            .source_url(hit.page_url.clone())
            .author(hit.user.clone())
            .author_url(format!(
                "https://pixabay.com/users/{}-{}/",
                hit.user, hit.user_id
            ))
            .license(License::Pixabay)
            .dimensions(hit.image_width, hit.image_height)
            .file_size(hit.image_size)
            .maybe_url_inferred_mime_type(Self::mime_type_from_url(&download_url))
            .provider_metadata(provider_metadata)
            .tags(tags)
            .build_or_log()
    }

    fn asset_from_video_hit(hit: PixabayVideoHit) -> Option<MediaAsset> {
        let (rendition, video) = hit
            .videos
            .large
            .as_ref()
            .map(|video| ("large", video))
            .or_else(|| hit.videos.medium.as_ref().map(|video| ("medium", video)))
            .or_else(|| hit.videos.small.as_ref().map(|video| ("small", video)))?;

        let download_url = video.url.clone();
        let preview_url = hit
            .videos
            .tiny
            .as_ref()
            .map(|video| video.url.clone())
            .unwrap_or_else(|| download_url.clone());
        let tags: Vec<String> = hit.tags.split(", ").map(|s| s.trim().to_string()).collect();
        let provider_metadata = Self::video_metadata(&hit, rendition, video, &download_url);

        MediaAsset::builder()
            .id(hit.id.to_string())
            .provider("pixabay")
            .media_type(MediaType::Video)
            .title(
                tags.first()
                    .cloned()
                    .unwrap_or_else(|| format!("Pixabay Video {}", hit.id)),
            )
            .direct_download_url(download_url.clone())
            .preview_url(preview_url)
            .source_url(hit.page_url.clone())
            .author(hit.user.clone())
            .author_url(format!(
                "https://pixabay.com/users/{}-{}/",
                hit.user, hit.user_id
            ))
            .license(License::Pixabay)
            .dimensions(video.width, video.height)
            .file_size(video.size)
            .maybe_url_inferred_mime_type(Self::mime_type_from_url(&download_url))
            .provider_metadata(provider_metadata)
            .tags(tags)
            .build_or_log()
    }

    fn image_metadata(hit: &PixabayHit, download_url: &str) -> HashMap<String, String> {
        let mut metadata = Self::base_metadata(
            hit.id,
            &hit.page_url,
            &hit.hit_type,
            &hit.user,
            hit.user_id,
            &hit.user_image_url,
            download_url,
        );
        metadata.insert("pixabay.preview_url".to_string(), hit.preview_url.clone());
        metadata.insert(
            "pixabay.webformat_url".to_string(),
            hit.web_format_url.clone(),
        );
        metadata.insert("pixabay.image_size".to_string(), hit.image_size.to_string());
        metadata.insert("pixabay.views".to_string(), hit.views.to_string());
        metadata.insert("pixabay.downloads".to_string(), hit.downloads.to_string());
        metadata.insert("pixabay.likes".to_string(), hit.likes.to_string());
        metadata
    }

    fn video_metadata(
        hit: &PixabayVideoHit,
        rendition: &str,
        video: &PixabayVideoSize,
        download_url: &str,
    ) -> HashMap<String, String> {
        let mut metadata = Self::base_metadata(
            hit.id,
            &hit.page_url,
            &hit.hit_type,
            &hit.user,
            hit.user_id,
            &hit.user_image_url,
            download_url,
        );
        metadata.insert(
            "pixabay.duration_seconds".to_string(),
            hit.duration.to_string(),
        );
        metadata.insert(
            "pixabay.selected_rendition".to_string(),
            rendition.to_string(),
        );
        metadata.insert(
            "pixabay.selected_rendition_size".to_string(),
            video.size.to_string(),
        );
        metadata.insert("pixabay.views".to_string(), hit.views.to_string());
        metadata.insert("pixabay.downloads".to_string(), hit.downloads.to_string());
        metadata.insert("pixabay.likes".to_string(), hit.likes.to_string());
        metadata
    }

    fn base_metadata(
        id: u64,
        page_url: &str,
        api_media_type: &str,
        user: &str,
        user_id: u64,
        user_image_url: &str,
        selected_download_url: &str,
    ) -> HashMap<String, String> {
        HashMap::from([
            ("pixabay.asset_id".to_string(), id.to_string()),
            ("pixabay.page_url".to_string(), page_url.to_string()),
            (
                "pixabay.api_media_type".to_string(),
                api_media_type.to_string(),
            ),
            ("pixabay.user".to_string(), user.to_string()),
            ("pixabay.user_id".to_string(), user_id.to_string()),
            (
                "pixabay.user_image_url".to_string(),
                user_image_url.to_string(),
            ),
            (
                "pixabay.selected_download_url".to_string(),
                selected_download_url.to_string(),
            ),
            (
                "pixabay.item_license_field".to_string(),
                "not-provided-by-api".to_string(),
            ),
            (
                "pixabay.license_scope".to_string(),
                "provider-default".to_string(),
            ),
        ])
    }

    fn mime_type_from_url(url: &str) -> Option<&'static str> {
        let url = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
        if url.ends_with(".jpg") || url.ends_with(".jpeg") {
            Some("image/jpeg")
        } else if url.ends_with(".png") {
            Some("image/png")
        } else if url.ends_with(".webp") {
            Some("image/webp")
        } else if url.ends_with(".svg") {
            Some("image/svg+xml")
        } else if url.ends_with(".mp4") {
            Some("video/mp4")
        } else if url.ends_with(".mov") {
            Some("video/quicktime")
        } else if url.ends_with(".webm") {
            Some("video/webm")
        } else {
            None
        }
    }
}

#[async_trait]
impl Provider for PixabayProvider {
    fn name(&self) -> &'static str {
        "pixabay"
    }

    fn display_name(&self) -> &'static str {
        "Pixabay"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image, MediaType::Video, MediaType::Vector]
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn rate_limit(&self) -> RateLimitConfig {
        Self::RATE_LIMIT
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    fn base_url(&self) -> &'static str {
        "https://pixabay.com/api/"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let Some(ref api_key) = self.api_key else {
            return Err(crate::error::DxError::MissingApiKey {
                provider: "pixabay".to_string(),
                env_var: "PIXABAY_API_KEY".to_string(),
            });
        };

        // Use video endpoint for video searches
        if query.media_type == Some(MediaType::Video) {
            return self.search_videos(api_key, query).await;
        }

        let image_type = match query.media_type {
            Some(MediaType::Vector) => "vector",
            Some(MediaType::Image) | None => "all",
            _ => "all",
        };

        let params = [
            ("key", api_key.as_str()),
            ("q", query.query.as_str()),
            ("page", &query.page.to_string()),
            ("per_page", &query.count.min(200).to_string()), // Pixabay max is 200
            ("image_type", image_type),
            ("safesearch", "true"),
        ];

        let response = self
            .client
            .get_with_query(self.base_url(), &params, &[])
            .await?;

        let api_response: PixabaySearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .hits
            .into_iter()
            .filter_map(Self::asset_from_image_hit)
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.total_hits,
            assets,
            providers_searched: vec!["pixabay".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl PixabayProvider {
    /// Search for videos using Pixabay video API endpoint.
    async fn search_videos(&self, api_key: &str, query: &SearchQuery) -> Result<SearchResult> {
        let video_url = "https://pixabay.com/api/videos/";

        let params = [
            ("key", api_key),
            ("q", query.query.as_str()),
            ("page", &query.page.to_string()),
            ("per_page", &query.count.min(200).to_string()),
            ("safesearch", "true"),
        ];

        let response = self.client.get_with_query(video_url, &params, &[]).await?;

        let api_response: PixabayVideoSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .hits
            .into_iter()
            .filter_map(Self::asset_from_video_hit)
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.total_hits,
            assets,
            providers_searched: vec!["pixabay".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for PixabayProvider {
    fn description(&self) -> &'static str {
        "Stunning royalty-free images & royalty-free stock"
    }

    fn api_key_url(&self) -> &'static str {
        "https://pixabay.com/api/docs/"
    }

    fn default_license(&self) -> &'static str {
        "Pixabay License (free for commercial use, no attribution required)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used for JSON deserialization
struct PixabaySearchResponse {
    total: usize,
    #[serde(rename = "totalHits")]
    total_hits: usize,
    hits: Vec<PixabayHit>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used for JSON deserialization
struct PixabayHit {
    id: u64,

    #[serde(rename = "pageURL")]
    page_url: String,

    #[serde(rename = "type", default)]
    hit_type: String,

    tags: String,

    #[serde(rename = "previewURL")]
    preview_url: String,

    #[serde(rename = "previewWidth")]
    preview_width: u32,

    #[serde(rename = "previewHeight")]
    preview_height: u32,

    #[serde(rename = "webformatURL")]
    web_format_url: String,

    #[serde(rename = "webformatWidth")]
    web_format_width: u32,

    #[serde(rename = "webformatHeight")]
    web_format_height: u32,

    #[serde(rename = "largeImageURL")]
    large_image_url: Option<String>,

    #[serde(rename = "imageWidth")]
    image_width: u32,

    #[serde(rename = "imageHeight")]
    image_height: u32,

    #[serde(rename = "imageSize")]
    image_size: u64,

    views: u64,
    downloads: u64,
    likes: u64,

    user: String,

    #[serde(rename = "user_id")]
    user_id: u64,

    #[serde(rename = "userImageURL")]
    user_image_url: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// VIDEO API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PixabayVideoSearchResponse {
    total: usize,
    #[serde(rename = "totalHits")]
    total_hits: usize,
    hits: Vec<PixabayVideoHit>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PixabayVideoHit {
    id: u64,

    #[serde(rename = "pageURL")]
    page_url: String,

    #[serde(rename = "type", default)]
    hit_type: String,

    tags: String,

    duration: u32,

    videos: PixabayVideoSizes,

    views: u64,
    downloads: u64,
    likes: u64,

    user: String,

    #[serde(rename = "user_id")]
    user_id: u64,

    #[serde(rename = "userImageURL")]
    user_image_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PixabayVideoSizes {
    large: Option<PixabayVideoSize>,
    medium: Option<PixabayVideoSize>,
    small: Option<PixabayVideoSize>,
    tiny: Option<PixabayVideoSize>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PixabayVideoSize {
    url: String,
    width: u32,
    height: u32,
    size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = PixabayProvider::new(&config);

        assert_eq!(provider.name(), "pixabay");
        assert_eq!(provider.display_name(), "Pixabay");
        assert!(provider.requires_api_key());
        assert!(!provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default_for_testing();
        let provider = PixabayProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
        assert!(types.contains(&MediaType::Video));
        assert!(types.contains(&MediaType::Vector));
    }

    #[test]
    fn pixabay_image_asset_preserves_provider_metadata_and_type_evidence() {
        let hit = PixabayHit {
            id: 12345,
            page_url: "https://pixabay.com/photos/flower-12345/".to_string(),
            hit_type: "photo".to_string(),
            tags: "flower, garden".to_string(),
            preview_url: "https://cdn.pixabay.com/photo/preview.jpg".to_string(),
            preview_width: 150,
            preview_height: 100,
            web_format_url: "https://cdn.pixabay.com/photo/webformat.jpg".to_string(),
            web_format_width: 640,
            web_format_height: 426,
            large_image_url: Some("https://cdn.pixabay.com/photo/flower_1280.jpg".to_string()),
            image_width: 1920,
            image_height: 1280,
            image_size: 1_024_000,
            views: 12_000,
            downloads: 4_000,
            likes: 250,
            user: "photographer1".to_string(),
            user_id: 99,
            user_image_url: "https://cdn.pixabay.com/user.jpg".to_string(),
        };

        let asset = PixabayProvider::asset_from_image_hit(hit).expect("valid pixabay image asset");
        let provenance = asset.provenance();

        assert_eq!(
            provenance
                .provider_metadata
                .get("pixabay.asset_id")
                .map(String::as_str),
            Some("12345")
        );
        assert_eq!(
            provenance
                .provider_metadata
                .get("pixabay.item_license_field")
                .map(String::as_str),
            Some("not-provided-by-api")
        );
        assert_eq!(
            provenance
                .provider_metadata
                .get("pixabay.license_scope")
                .map(String::as_str),
            Some("provider-default")
        );
        assert_eq!(
            provenance.download_url_kind,
            crate::types::DownloadUrlKind::DirectFile
        );
        assert_eq!(provenance.mime_type.as_deref(), Some("image/jpeg"));
        assert!(provenance.type_validation.is_valid());
    }

    #[test]
    fn pixabay_video_asset_preserves_selected_file_metadata() {
        let hit = PixabayVideoHit {
            id: 23456,
            page_url: "https://pixabay.com/videos/ocean-23456/".to_string(),
            hit_type: "film".to_string(),
            tags: "ocean, wave".to_string(),
            duration: 12,
            videos: PixabayVideoSizes {
                large: Some(PixabayVideoSize {
                    url: "https://cdn.pixabay.com/video/large.mp4".to_string(),
                    width: 1920,
                    height: 1080,
                    size: 8_000_000,
                }),
                medium: None,
                small: None,
                tiny: Some(PixabayVideoSize {
                    url: "https://cdn.pixabay.com/video/tiny.mp4".to_string(),
                    width: 480,
                    height: 270,
                    size: 500_000,
                }),
            },
            views: 6_000,
            downloads: 1_000,
            likes: 80,
            user: "videographer1".to_string(),
            user_id: 100,
            user_image_url: "https://cdn.pixabay.com/user-video.jpg".to_string(),
        };

        let asset = PixabayProvider::asset_from_video_hit(hit).expect("valid pixabay video asset");
        let provenance = asset.provenance();

        assert_eq!(
            provenance
                .provider_metadata
                .get("pixabay.selected_rendition")
                .map(String::as_str),
            Some("large")
        );
        assert_eq!(
            provenance.download_url_kind,
            crate::types::DownloadUrlKind::DirectFile
        );
        assert_eq!(asset.file_size, Some(8_000_000));
        assert_eq!(provenance.mime_type.as_deref(), Some("video/mp4"));
        assert!(provenance.type_validation.is_valid());
    }
}
