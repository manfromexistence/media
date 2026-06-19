//! NASA Images provider implementation.
//!
//! [NASA Images API Documentation](https://images.nasa.gov/docs/images.nasa.gov_api_docs.pdf)
//!
//! Provides access to NASA's image and video library with 140,000+ public domain assets.

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

/// NASA Images provider for space and science media.
/// Access to 140K+ public domain images and videos.
#[derive(Debug)]
pub struct NasaImagesProvider {
    client: HttpClient,
    /// Base URL for API requests (configurable for testing)
    base_url: String,
}

impl NasaImagesProvider {
    /// Default base URL for NASA Images API.
    const DEFAULT_BASE_URL: &'static str = "https://images-api.nasa.gov";

    /// Create a new NASA Images provider.
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

    /// Create a new NASA Images provider with a custom base URL.
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

    /// Rate limit: Unlimited (but be respectful)
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(1000, 3600);

    /// Get the media type filter string for the API
    fn media_type_filter(media_type: Option<MediaType>) -> &'static str {
        match media_type {
            Some(MediaType::Image) => "image",
            Some(MediaType::Gif) => "image",
            Some(MediaType::Video) => "video",
            Some(MediaType::Audio) => "audio",
            _ => "image",
        }
    }

    /// Parse media type from string
    fn parse_media_type(s: &str) -> MediaType {
        match s {
            "video" => MediaType::Video,
            "audio" => MediaType::Audio,
            _ => MediaType::Image,
        }
    }

    fn mime_type_from_url(media_type: MediaType, url: &str) -> Option<&'static str> {
        let lower = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
        match media_type {
            MediaType::Image if lower.ends_with(".png") => Some("image/png"),
            MediaType::Image if lower.ends_with(".jpg") || lower.ends_with(".jpeg") => {
                Some("image/jpeg")
            }
            MediaType::Image if lower.ends_with(".tif") || lower.ends_with(".tiff") => {
                Some("image/tiff")
            }
            MediaType::Gif if lower.ends_with(".gif") => Some("image/gif"),
            MediaType::Video if lower.ends_with(".mov") => Some("video/quicktime"),
            MediaType::Video if lower.ends_with(".mp4") => Some("video/mp4"),
            MediaType::Audio if lower.ends_with(".wav") => Some("audio/wav"),
            MediaType::Audio if lower.ends_with(".mp3") => Some("audio/mpeg"),
            _ => None,
        }
    }

    fn refine_media_type_from_url(media_type: MediaType, url: &str) -> MediaType {
        let lower = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
        if media_type == MediaType::Image && lower.ends_with(".gif") {
            MediaType::Gif
        } else {
            media_type
        }
    }

    fn image_download_candidate(
        manifest_url: &str,
        preview_url: Option<&str>,
    ) -> (String, DownloadUrlKind, &'static str) {
        preview_url.map_or_else(
            || {
                (
                    manifest_url.to_string(),
                    DownloadUrlKind::AssetManifest,
                    "asset-manifest",
                )
            },
            |url| {
                let lower = url.to_ascii_lowercase();
                if lower.contains("~orig.") || lower.contains("~orig_") {
                    (url.to_string(), DownloadUrlKind::DirectFile, "direct-file")
                } else {
                    (
                        url.to_string(),
                        DownloadUrlKind::PreviewDerivative,
                        "preview-derivative",
                    )
                }
            },
        )
    }

    fn manifest_request_url(&self, manifest_url: &str) -> String {
        if self.base_url != Self::DEFAULT_BASE_URL
            && manifest_url.starts_with(Self::DEFAULT_BASE_URL)
        {
            manifest_url.replacen(Self::DEFAULT_BASE_URL, &self.base_url, 1)
        } else {
            manifest_url.to_string()
        }
    }

    async fn resolve_download_candidate(
        &self,
        manifest_url: &str,
        provider_media_type: MediaType,
        preview_url: Option<&str>,
    ) -> (String, DownloadUrlKind, &'static str) {
        if provider_media_type == MediaType::Image {
            let (candidate_url, candidate_kind, candidate_role) =
                Self::image_download_candidate(manifest_url, preview_url);
            if candidate_kind == DownloadUrlKind::DirectFile {
                return (candidate_url, candidate_kind, candidate_role);
            }
        }

        if let Some(direct_url) = self
            .direct_download_url_from_manifest(manifest_url, provider_media_type)
            .await
        {
            return (direct_url, DownloadUrlKind::DirectFile, "direct-file");
        }

        (
            manifest_url.to_string(),
            DownloadUrlKind::AssetManifest,
            "asset-manifest",
        )
    }

    async fn direct_download_url_from_manifest(
        &self,
        manifest_url: &str,
        provider_media_type: MediaType,
    ) -> Option<String> {
        let request_url = self.manifest_request_url(manifest_url);
        let response = self.client.get(&request_url).await.ok()?;
        let urls = response.json_or_error::<Vec<String>>().await.ok()?;
        Self::direct_url_from_manifest_urls(provider_media_type, urls)
    }

    fn direct_url_from_manifest_urls(
        provider_media_type: MediaType,
        urls: Vec<String>,
    ) -> Option<String> {
        urls.into_iter().find(|url| {
            let lower = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
            match provider_media_type {
                MediaType::Image => {
                    lower.contains("~orig.")
                        && (lower.ends_with(".jpg")
                            || lower.ends_with(".jpeg")
                            || lower.ends_with(".png")
                            || lower.ends_with(".gif")
                            || lower.ends_with(".tif")
                            || lower.ends_with(".tiff"))
                }
                MediaType::Video => lower.ends_with(".mp4") || lower.ends_with(".mov"),
                MediaType::Audio => lower.ends_with(".mp3") || lower.ends_with(".wav"),
                MediaType::Gif => lower.ends_with(".gif"),
                _ => false,
            }
        })
    }

    fn detail_url(nasa_id: &str) -> String {
        format!("https://images.nasa.gov/details/{nasa_id}")
    }

    fn provider_metadata(data: &NasaItemData, manifest_url: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::from([
            ("nasa.media_type".to_string(), data.media_type.clone()),
            (
                "nasa.asset_manifest_url".to_string(),
                manifest_url.to_string(),
            ),
            (
                "nasa.item_license_field".to_string(),
                "not-provided".to_string(),
            ),
            (
                "nasa.license_scope".to_string(),
                "provider-default".to_string(),
            ),
        ]);
        if let Some(center) = data
            .center
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            metadata.insert("nasa.center".to_string(), center.clone());
        }
        if let Some(date) = data
            .date_created
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            metadata.insert("nasa.date_created".to_string(), date.clone());
        }
        metadata
    }

    fn provider_default_license() -> License {
        License::Other("NASA provider default; item license not provided".to_string())
    }
}

#[async_trait]
impl Provider for NasaImagesProvider {
    fn name(&self) -> &'static str {
        "nasa"
    }

    fn display_name(&self) -> &'static str {
        "NASA Images"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[
            MediaType::Image,
            MediaType::Gif,
            MediaType::Video,
            MediaType::Audio,
        ]
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
        // Note: This returns the default static URL for trait compliance.
        // The actual search method uses self.base_url field which may be customized.
        Self::DEFAULT_BASE_URL
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = format!("{}/search", self.base_url);

        let media_type = Self::media_type_filter(query.media_type);
        let page_str = query.page.to_string();
        let count_str = query.count.min(100).to_string();

        let params = [
            ("q", query.query.as_str()),
            ("media_type", media_type),
            ("page", &page_str),
            ("page_size", &count_str),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: NasaSearchResponse = response.json_or_error().await?;

        let mut assets = Vec::new();
        for item in api_response.collection.items {
            let Some(data) = item.data.into_iter().next() else {
                continue;
            };
            let link = item.links.and_then(|l| l.into_iter().next());

            let preview_url = link.as_ref().map(|l| l.href.clone());
            let provider_media_type = Self::parse_media_type(&data.media_type);
            let (download_url, download_url_kind, download_url_role) = self
                .resolve_download_candidate(&item.href, provider_media_type, preview_url.as_deref())
                .await;
            let media_type = Self::refine_media_type_from_url(provider_media_type, &download_url);
            if let Some(requested_type) = query.media_type {
                if media_type != requested_type {
                    continue;
                }
            }
            let mime_type = Self::mime_type_from_url(media_type, &download_url);
            let mut provider_metadata = Self::provider_metadata(&data, &item.href);
            provider_metadata.insert(
                "nasa.download_url_role".to_string(),
                download_url_role.to_string(),
            );
            let source_url = Self::detail_url(&data.nasa_id);

            if let Some(asset) = MediaAsset::builder()
                .id(data.nasa_id)
                .provider("nasa")
                .media_type(media_type)
                .title(data.title)
                .download_url(download_url)
                .download_url_kind(download_url_kind)
                .maybe_preview_url(preview_url)
                .source_url(source_url)
                .author(data.center.unwrap_or_else(|| "NASA".to_string()))
                .license(Self::provider_default_license())
                .maybe_url_inferred_mime_type(mime_type)
                .provider_metadata(provider_metadata)
                .tags(data.keywords.unwrap_or_default())
                .build_or_log()
            {
                assets.push(asset);
            }
        }

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.collection.metadata.total_hits,
            assets,
            providers_searched: vec!["nasa".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<MediaAsset>> {
        // NASA API: GET /search?nasa_id={id}
        let url = format!("{}/search", self.base_url);
        let params = [("nasa_id", id)];

        let response = self.client.get_with_query(&url, &params, &[]).await?;
        let api_response: NasaSearchResponse = response.json_or_error().await?;

        // Get the first (and should be only) result
        let item = api_response.collection.items.into_iter().next();

        if let Some(item) = item {
            let data = item.data.into_iter().next();
            if let Some(data) = data {
                let provider_media_type = Self::parse_media_type(&data.media_type);
                let preview_url = item
                    .links
                    .as_ref()
                    .and_then(|l| l.first())
                    .map(|l| l.href.clone());

                let (download_url, download_url_kind, download_url_role) = self
                    .resolve_download_candidate(
                        &item.href,
                        provider_media_type,
                        preview_url.as_deref(),
                    )
                    .await;
                let media_type =
                    Self::refine_media_type_from_url(provider_media_type, &download_url);
                let mime_type = Self::mime_type_from_url(media_type, &download_url);
                let mut provider_metadata = Self::provider_metadata(&data, &item.href);
                provider_metadata.insert(
                    "nasa.download_url_role".to_string(),
                    download_url_role.to_string(),
                );
                let source_url = Self::detail_url(&data.nasa_id);

                return Ok(MediaAsset::builder()
                    .id(data.nasa_id)
                    .provider("nasa")
                    .media_type(media_type)
                    .title(data.title)
                    .download_url(download_url)
                    .download_url_kind(download_url_kind)
                    .maybe_preview_url(preview_url)
                    .source_url(source_url)
                    .author(data.center.unwrap_or_else(|| "NASA".to_string()))
                    .license(Self::provider_default_license())
                    .maybe_url_inferred_mime_type(mime_type)
                    .provider_metadata(provider_metadata)
                    .tags(data.keywords.unwrap_or_default())
                    .build_or_log());
            }
        }

        Ok(None)
    }
}

impl ProviderInfo for NasaImagesProvider {
    fn description(&self) -> &'static str {
        "NASA's official image and video library with space and science media"
    }

    fn api_key_url(&self) -> &'static str {
        "https://images.nasa.gov/docs/images.nasa.gov_api_docs.pdf"
    }

    fn default_license(&self) -> &'static str {
        "NASA provider default; item license not provided"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NasaSearchResponse {
    collection: NasaCollection,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NasaCollection {
    items: Vec<NasaItem>,
    metadata: NasaMetadata,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NasaMetadata {
    total_hits: usize,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NasaItem {
    href: String,
    data: Vec<NasaItemData>,
    links: Option<Vec<NasaLink>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NasaItemData {
    nasa_id: String,
    title: String,
    media_type: String,
    description: Option<String>,
    center: Option<String>,
    date_created: Option<String>,
    #[serde(default)]
    keywords: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NasaLink {
    href: String,
    rel: String,
    render: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = NasaImagesProvider::new(&config);

        assert_eq!(provider.name(), "nasa");
        assert_eq!(provider.display_name(), "NASA Images");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default_for_testing();
        let provider = NasaImagesProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
        assert!(types.contains(&MediaType::Video));
        assert!(types.contains(&MediaType::Audio));
    }

    #[test]
    fn test_provider_default_license_is_not_item_level_evidence() {
        assert!(matches!(
            NasaImagesProvider::provider_default_license(),
            License::Other(value) if value == "NASA provider default; item license not provided"
        ));
        assert!(!NasaImagesProvider::provider_default_license().is_known());
    }

    #[test]
    fn nasa_detail_url_is_human_source_not_asset_manifest() {
        assert_eq!(
            NasaImagesProvider::detail_url("PIA12345"),
            "https://images.nasa.gov/details/PIA12345"
        );
    }

    #[test]
    fn image_records_with_gif_direct_urls_are_typed_as_gif_assets() {
        let media_type = NasaImagesProvider::refine_media_type_from_url(
            MediaType::Image,
            "https://images-assets.nasa.gov/image/PIA12345/PIA12345~orig.gif?download=1",
        );

        assert_eq!(media_type, MediaType::Gif);
        assert_eq!(
            NasaImagesProvider::refine_media_type_from_url(
                MediaType::Image,
                "https://images-assets.nasa.gov/image/PIA12345/PIA12345~orig.jpg",
            ),
            MediaType::Image
        );
    }

    #[test]
    fn image_thumbnail_links_are_preview_derivatives() {
        let (url, kind, role) = NasaImagesProvider::image_download_candidate(
            "https://images-api.nasa.gov/asset/PIA12345",
            Some("https://images-assets.nasa.gov/image/PIA12345/PIA12345~thumb.jpg"),
        );

        assert!(url.ends_with("~thumb.jpg"));
        assert_eq!(kind, DownloadUrlKind::PreviewDerivative);
        assert_eq!(role, "preview-derivative");
    }

    #[test]
    fn image_manifest_prefers_tiff_originals() {
        let selected = NasaImagesProvider::direct_url_from_manifest_urls(
            MediaType::Image,
            vec![
                "https://images-assets.nasa.gov/image/PIA12345/PIA12345~small.jpg".to_string(),
                "https://images-assets.nasa.gov/image/PIA12345/PIA12345~orig.tif?download=1"
                    .to_string(),
            ],
        );

        assert_eq!(
            selected.as_deref(),
            Some("https://images-assets.nasa.gov/image/PIA12345/PIA12345~orig.tif?download=1")
        );
    }

    #[test]
    fn image_manifest_rejects_preview_derivatives_without_originals() {
        let selected = NasaImagesProvider::direct_url_from_manifest_urls(
            MediaType::Image,
            vec![
                "https://images-assets.nasa.gov/image/PIA12345/PIA12345~thumb.jpg".to_string(),
                "https://images-assets.nasa.gov/image/PIA12345/PIA12345~small.png".to_string(),
            ],
        );

        assert!(selected.is_none());
    }
}
