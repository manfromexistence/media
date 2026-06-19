//! Pexels provider implementation.
//!
//! [Pexels API Documentation](https://www.pexels.com/api/documentation/)

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Pexels provider for stock photos and videos.
#[derive(Debug)]
pub struct PexelsProvider {
    api_key: Option<String>,
    client: HttpClient,
}

impl PexelsProvider {
    /// Create a new Pexels provider.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let client = HttpClient::with_config(
            Self::RATE_LIMIT,
            config.retry_attempts,
            Duration::from_secs(config.timeout_secs),
        )
        .unwrap_or_default();

        Self {
            api_key: config.pexels_api_key.clone(),
            client,
        }
    }

    /// Rate limit: 200 requests per hour
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(200, 3600);

    fn asset_from_photo(photo: PexelsPhoto) -> Option<MediaAsset> {
        let download_url = photo.src.original.clone();
        let preview_url = photo.src.medium.clone();
        let provider_metadata = Self::photo_metadata(&photo, &download_url, &preview_url);

        MediaAsset::builder()
            .id(photo.id.to_string())
            .provider("pexels")
            .media_type(MediaType::Image)
            .title(
                photo
                    .alt
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| format!("Pexels Photo {}", photo.id)),
            )
            .direct_download_url(download_url.clone())
            .preview_url(preview_url)
            .source_url(photo.url)
            .author(photo.photographer)
            .author_url(photo.photographer_url)
            .license(License::Pexels)
            .dimensions(photo.width, photo.height)
            .maybe_url_inferred_mime_type(Self::mime_type_from_url(&download_url))
            .provider_metadata(provider_metadata)
            .build_or_log()
    }

    fn asset_from_video(video: PexelsVideo) -> Option<MediaAsset> {
        let best_file = video
            .video_files
            .iter()
            .filter(|file| file.quality == "hd" || file.quality == "sd")
            .max_by_key(|file| file.width.unwrap_or(0))
            .or_else(|| video.video_files.first())?;

        let download_url = best_file.link.clone();
        let preview_url = video
            .video_pictures
            .first()
            .map(|picture| picture.picture.clone());
        let provider_metadata = Self::video_metadata(&video, best_file, &download_url);
        let provider_supplied_mime = !best_file.file_type.trim().is_empty();
        let mime_type = if provider_supplied_mime {
            Some(best_file.file_type.trim().to_string())
        } else {
            Self::mime_type_from_url(&download_url).map(str::to_string)
        };
        let width = best_file.width.unwrap_or(video.width);
        let height = best_file.height.unwrap_or(video.height);

        let builder = MediaAsset::builder()
            .id(video.id.to_string())
            .provider("pexels")
            .media_type(MediaType::Video)
            .title(format!("Pexels Video {}", video.id))
            .direct_download_url(download_url)
            .maybe_preview_url(preview_url)
            .source_url(video.url)
            .author(video.user.name)
            .author_url(video.user.url)
            .license(License::Pexels)
            .dimensions(width, height)
            .provider_metadata(provider_metadata);

        if provider_supplied_mime {
            builder.maybe_provider_mime_type(mime_type).build_or_log()
        } else {
            builder
                .maybe_url_inferred_mime_type(mime_type)
                .build_or_log()
        }
    }

    fn photo_metadata(
        photo: &PexelsPhoto,
        download_url: &str,
        preview_url: &str,
    ) -> HashMap<String, String> {
        let mut metadata = Self::base_metadata(photo.id, &photo.url, download_url);
        metadata.insert(
            "pexels.photographer_url".to_string(),
            photo.photographer_url.clone(),
        );
        metadata.insert("pexels.preview_url".to_string(), preview_url.to_string());
        if photo.photographer_id > 0 {
            metadata.insert(
                "pexels.photographer_id".to_string(),
                photo.photographer_id.to_string(),
            );
        }
        if let Some(avg_color) = photo
            .avg_color
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            metadata.insert("pexels.avg_color".to_string(), avg_color.clone());
        }
        metadata
    }

    fn video_metadata(
        video: &PexelsVideo,
        selected_file: &PexelsVideoFile,
        download_url: &str,
    ) -> HashMap<String, String> {
        let mut metadata = Self::base_metadata(video.id, &video.url, download_url);
        metadata.insert("pexels.user_id".to_string(), video.user.id.to_string());
        metadata.insert("pexels.user_url".to_string(), video.user.url.clone());
        metadata.insert(
            "pexels.duration_seconds".to_string(),
            video.duration.to_string(),
        );
        metadata.insert(
            "pexels.selected_file_id".to_string(),
            selected_file.id.to_string(),
        );
        metadata.insert(
            "pexels.selected_file_quality".to_string(),
            selected_file.quality.clone(),
        );
        metadata.insert(
            "pexels.selected_file_type".to_string(),
            selected_file.file_type.clone(),
        );
        metadata
    }

    fn base_metadata(
        id: u64,
        page_url: &str,
        selected_download_url: &str,
    ) -> HashMap<String, String> {
        HashMap::from([
            ("pexels.asset_id".to_string(), id.to_string()),
            ("pexels.page_url".to_string(), page_url.to_string()),
            (
                "pexels.selected_download_url".to_string(),
                selected_download_url.to_string(),
            ),
            (
                "pexels.item_license_field".to_string(),
                "not-provided-by-api".to_string(),
            ),
            (
                "pexels.license_scope".to_string(),
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
impl Provider for PexelsProvider {
    fn name(&self) -> &'static str {
        "pexels"
    }

    fn display_name(&self) -> &'static str {
        "Pexels"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image, MediaType::Video]
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
        "https://api.pexels.com/v1"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let Some(ref api_key) = self.api_key else {
            return Err(crate::error::DxError::MissingApiKey {
                provider: "pexels".to_string(),
                env_var: "PEXELS_API_KEY".to_string(),
            });
        };

        // Use video endpoint for video searches
        if query.media_type == Some(MediaType::Video) {
            return self.search_videos(api_key, query).await;
        }

        let url = format!("{}/search", self.base_url());

        let params = [
            ("query", query.query.as_str()),
            ("page", &query.page.to_string()),
            ("per_page", &query.count.min(80).to_string()), // Pexels max is 80
        ];

        let headers = [("Authorization", api_key.as_str())];

        let response = self.client.get_with_query(&url, &params, &headers).await?;

        let api_response: PexelsSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .photos
            .into_iter()
            .filter_map(Self::asset_from_photo)
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.total_results,
            assets,
            providers_searched: vec!["pexels".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl PexelsProvider {
    /// Search for videos using Pexels video API endpoint.
    async fn search_videos(&self, api_key: &str, query: &SearchQuery) -> Result<SearchResult> {
        let url = "https://api.pexels.com/videos/search";

        let params = [
            ("query", query.query.as_str()),
            ("page", &query.page.to_string()),
            ("per_page", &query.count.min(80).to_string()),
        ];

        let headers = [("Authorization", api_key)];

        let response = self.client.get_with_query(url, &params, &headers).await?;

        let api_response: PexelsVideoSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .videos
            .into_iter()
            .filter_map(Self::asset_from_video)
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.total_results,
            assets,
            providers_searched: vec!["pexels".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}
impl ProviderInfo for PexelsProvider {
    fn description(&self) -> &'static str {
        "Free stock photos and videos shared by talented creators"
    }

    fn api_key_url(&self) -> &'static str {
        "https://www.pexels.com/api/"
    }

    fn default_license(&self) -> &'static str {
        "Pexels License (free for personal and commercial use)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used for JSON deserialization
struct PexelsSearchResponse {
    total_results: usize,
    page: usize,
    per_page: usize,
    photos: Vec<PexelsPhoto>,
    #[serde(default)]
    next_page: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used for JSON deserialization
struct PexelsPhoto {
    id: u64,
    width: u32,
    height: u32,
    url: String,
    photographer: String,
    photographer_url: String,
    #[serde(default)]
    photographer_id: u64,
    avg_color: Option<String>,
    src: PexelsPhotoSrc,
    alt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used for JSON deserialization
struct PexelsPhotoSrc {
    original: String,
    large2x: String,
    large: String,
    medium: String,
    small: String,
    portrait: String,
    landscape: String,
    tiny: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// VIDEO API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PexelsVideoSearchResponse {
    total_results: usize,
    page: usize,
    per_page: usize,
    videos: Vec<PexelsVideo>,
    #[serde(default)]
    next_page: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PexelsVideo {
    id: u64,
    width: u32,
    height: u32,
    url: String,
    duration: u32,
    user: PexelsVideoUser,
    video_files: Vec<PexelsVideoFile>,
    video_pictures: Vec<PexelsVideoPicture>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PexelsVideoUser {
    id: u64,
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PexelsVideoFile {
    id: u64,
    quality: String,
    file_type: String,
    width: Option<u32>,
    height: Option<u32>,
    link: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PexelsVideoPicture {
    id: u64,
    picture: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = PexelsProvider::new(&config);

        assert_eq!(provider.name(), "pexels");
        assert_eq!(provider.display_name(), "Pexels");
        assert!(provider.requires_api_key());
        assert!(!provider.is_available());
    }

    #[test]
    fn pexels_photo_asset_preserves_provider_metadata_and_type_evidence() {
        let photo = PexelsPhoto {
            id: 123,
            width: 2048,
            height: 1365,
            url: "https://www.pexels.com/photo/example-123/".to_string(),
            photographer: "Jane Photographer".to_string(),
            photographer_url: "https://www.pexels.com/@jane".to_string(),
            photographer_id: 456,
            avg_color: Some("#AABBCC".to_string()),
            src: PexelsPhotoSrc {
                original: "https://images.pexels.com/photos/123/example.jpeg".to_string(),
                large2x: "https://images.pexels.com/photos/123/large2x.jpeg".to_string(),
                large: "https://images.pexels.com/photos/123/large.jpeg".to_string(),
                medium: "https://images.pexels.com/photos/123/medium.jpeg".to_string(),
                small: "https://images.pexels.com/photos/123/small.jpeg".to_string(),
                portrait: "https://images.pexels.com/photos/123/portrait.jpeg".to_string(),
                landscape: "https://images.pexels.com/photos/123/landscape.jpeg".to_string(),
                tiny: "https://images.pexels.com/photos/123/tiny.jpeg".to_string(),
            },
            alt: Some("A clear provenance fixture".to_string()),
        };

        let asset = PexelsProvider::asset_from_photo(photo).expect("valid pexels photo asset");
        let provenance = asset.provenance();

        assert_eq!(
            provenance
                .provider_metadata
                .get("pexels.asset_id")
                .map(String::as_str),
            Some("123")
        );
        assert_eq!(
            provenance
                .provider_metadata
                .get("pexels.photographer_id")
                .map(String::as_str),
            Some("456")
        );
        assert_eq!(
            provenance
                .provider_metadata
                .get("pexels.item_license_field")
                .map(String::as_str),
            Some("not-provided-by-api")
        );
        assert_eq!(
            provenance.download_url_kind,
            crate::types::DownloadUrlKind::DirectFile
        );
        assert_eq!(provenance.mime_type.as_deref(), Some("image/jpeg"));
        assert!(provenance.type_validation.is_valid());
    }

    #[test]
    fn pexels_video_asset_preserves_selected_file_metadata() {
        let video = PexelsVideo {
            id: 789,
            width: 1920,
            height: 1080,
            url: "https://www.pexels.com/video/example-789/".to_string(),
            duration: 30,
            user: PexelsVideoUser {
                id: 654,
                name: "Jane Video".to_string(),
                url: "https://www.pexels.com/@jane-video".to_string(),
            },
            video_files: vec![
                PexelsVideoFile {
                    id: 1,
                    quality: "sd".to_string(),
                    file_type: "video/mp4".to_string(),
                    width: Some(640),
                    height: Some(360),
                    link: "https://videos.pexels.com/video-files/789/sd.mp4".to_string(),
                },
                PexelsVideoFile {
                    id: 2,
                    quality: "hd".to_string(),
                    file_type: "video/mp4".to_string(),
                    width: Some(1920),
                    height: Some(1080),
                    link: "https://videos.pexels.com/video-files/789/hd.mp4".to_string(),
                },
            ],
            video_pictures: vec![PexelsVideoPicture {
                id: 7,
                picture: "https://images.pexels.com/videos/789/poster.jpeg".to_string(),
            }],
        };

        let asset = PexelsProvider::asset_from_video(video).expect("valid pexels video asset");
        let provenance = asset.provenance();

        assert_eq!(
            provenance
                .provider_metadata
                .get("pexels.selected_file_id")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            provenance
                .provider_metadata
                .get("pexels.selected_file_quality")
                .map(String::as_str),
            Some("hd")
        );
        assert_eq!(
            provenance.download_url_kind,
            crate::types::DownloadUrlKind::DirectFile
        );
        assert_eq!(provenance.mime_type.as_deref(), Some("video/mp4"));
        assert!(provenance.type_validation.is_valid());
    }
}
