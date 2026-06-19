//! Giphy provider implementation.
//!
//! [Giphy API Documentation](https://developers.giphy.com/docs/api)
//!
//! Provides access to millions of GIFs and stickers.

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

/// Giphy provider for GIFs and stickers.
/// Access to millions of animated GIFs.
#[derive(Debug)]
pub struct GiphyProvider {
    api_key: Option<String>,
    client: HttpClient,
}

impl GiphyProvider {
    /// Create a new Giphy provider.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let client = HttpClient::with_config(
            Self::RATE_LIMIT,
            config.retry_attempts,
            Duration::from_secs(config.timeout_secs),
        )
        .unwrap_or_default();

        Self {
            api_key: config.giphy_api_key.clone(),
            client,
        }
    }

    /// Rate limit: 42 requests per hour for free tier, 1000 for production
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(42, 3600);

    fn asset_from_gif(gif: GiphyGif) -> Option<MediaAsset> {
        let download_url = gif.images.original.url.as_deref()?.trim();
        if download_url.is_empty() {
            return None;
        }
        let download_url = download_url.to_string();
        let source_url = non_empty_or(&gif.url, &download_url);
        let source_url_kind = if source_url == download_url {
            "direct-asset-url"
        } else {
            "provider-source-url"
        };
        let preview_url = gif
            .images
            .fixed_height
            .url
            .as_deref()
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .map(str::to_string);
        let width = parse_u32(&gif.images.original.width);
        let height = parse_u32(&gif.images.original.height);
        let file_size = parse_u64(&gif.images.original.size);

        let mut metadata = direct_asset_metadata("giphy", &download_url);
        metadata.insert("giphy.source_url".to_string(), source_url.clone());
        metadata.insert(
            "giphy.source_url_kind".to_string(),
            source_url_kind.to_string(),
        );
        if let Some(preview_url) = &preview_url {
            metadata.insert("giphy.preview_url".to_string(), preview_url.clone());
        }
        if let Some(width) = width {
            metadata.insert("giphy.original_width".to_string(), width.to_string());
        }
        if let Some(height) = height {
            metadata.insert("giphy.original_height".to_string(), height.to_string());
        }
        if let Some(file_size) = file_size {
            metadata.insert(
                "giphy.original_size_bytes".to_string(),
                file_size.to_string(),
            );
        }

        let title = gif
            .title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .unwrap_or("Giphy GIF")
            .to_string();
        let author = gif
            .username
            .as_deref()
            .map(str::trim)
            .filter(|author| !author.is_empty())
            .map(str::to_string);

        let mut builder = MediaAsset::builder()
            .id(gif.id)
            .provider("giphy")
            .media_type(MediaType::Gif)
            .title(title)
            .direct_download_url(download_url.clone())
            .source_url(source_url)
            .license(license_not_provided())
            .maybe_url_inferred_mime_type(mime_type_from_url(MediaType::Gif, &download_url))
            .provider_metadata(metadata);

        if let Some(preview_url) = preview_url {
            builder = builder.preview_url(preview_url);
        }
        if let Some(author) = author {
            builder = builder.author(author);
        }
        if let (Some(width), Some(height)) = (width, height) {
            builder = builder.dimensions(width, height);
        }
        if let Some(file_size) = file_size {
            builder = builder.file_size(file_size);
        }

        builder.build_or_log()
    }
}

fn non_empty_or(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn parse_u32(value: &Option<String>) -> Option<u32> {
    value.as_deref()?.trim().parse().ok()
}

fn parse_u64(value: &Option<String>) -> Option<u64> {
    value.as_deref()?.trim().parse().ok()
}

#[async_trait]
impl Provider for GiphyProvider {
    fn name(&self) -> &'static str {
        "giphy"
    }

    fn display_name(&self) -> &'static str {
        "Giphy"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Gif]
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
        "https://api.giphy.com/v1"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let Some(ref api_key) = self.api_key else {
            return Err(crate::error::DxError::MissingApiKey {
                provider: "giphy".to_string(),
                env_var: "GIPHY_API_KEY".to_string(),
            });
        };

        let url = format!("{}/gifs/search", self.base_url());

        let offset = ((query.page - 1) * query.count).to_string();
        let limit = query.count.min(50).to_string();

        let params = [
            ("api_key", api_key.as_str()),
            ("q", query.query.as_str()),
            ("offset", &offset),
            ("limit", &limit),
            ("rating", "g"), // Safe for work
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: GiphySearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .data
            .into_iter()
            .filter_map(Self::asset_from_gif)
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.pagination.total_count,
            assets,
            providers_searched: vec!["giphy".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for GiphyProvider {
    fn description(&self) -> &'static str {
        "World's largest library of animated GIFs and stickers"
    }

    fn api_key_url(&self) -> &'static str {
        "https://developers.giphy.com/"
    }

    fn default_license(&self) -> &'static str {
        "Giphy Terms of Service"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GiphySearchResponse {
    data: Vec<GiphyGif>,
    pagination: GiphyPagination,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GiphyGif {
    id: String,
    url: String,
    title: Option<String>,
    username: Option<String>,
    images: GiphyImages,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GiphyImages {
    original: GiphyImage,
    fixed_height: GiphyImage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GiphyImage {
    url: Option<String>,
    width: Option<String>,
    height: Option<String>,
    size: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GiphyPagination {
    total_count: usize,
    count: usize,
    offset: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = GiphyProvider::new(&config);

        assert_eq!(provider.name(), "giphy");
        assert_eq!(provider.display_name(), "Giphy");
        assert!(provider.requires_api_key());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default_for_testing();
        let provider = GiphyProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Gif));
    }

    #[test]
    fn gif_asset_preserves_source_license_mime_size_and_provider_metadata() {
        let asset = GiphyProvider::asset_from_gif(GiphyGif {
            id: "fixture-gif".to_string(),
            url: "https://giphy.com/gifs/fixture-gif".to_string(),
            title: Some("Fixture GIF".to_string()),
            username: Some("fixture-author".to_string()),
            images: GiphyImages {
                original: GiphyImage {
                    url: Some(
                        "https://media.giphy.com/media/fixture/giphy.gif?cid=test".to_string(),
                    ),
                    width: Some("320".to_string()),
                    height: Some("240".to_string()),
                    size: Some("12345".to_string()),
                },
                fixed_height: GiphyImage {
                    url: Some("https://media.giphy.com/media/fixture/200.gif".to_string()),
                    width: Some("200".to_string()),
                    height: Some("150".to_string()),
                    size: Some("4567".to_string()),
                },
            },
        })
        .expect("fixture Giphy response should build asset");
        let provenance = asset.provenance();

        assert_eq!(asset.provider, "giphy");
        assert_eq!(asset.media_type, MediaType::Gif);
        assert_eq!(asset.title, "Fixture GIF");
        assert_eq!(asset.author.as_deref(), Some("fixture-author"));
        assert_eq!(
            asset.download_url,
            "https://media.giphy.com/media/fixture/giphy.gif?cid=test"
        );
        assert_eq!(asset.source_url, "https://giphy.com/gifs/fixture-gif");
        assert_eq!(
            asset.download_url_kind,
            crate::types::DownloadUrlKind::DirectFile
        );
        assert_eq!(
            asset.preview_url.as_deref(),
            Some("https://media.giphy.com/media/fixture/200.gif")
        );
        assert_eq!(asset.mime_type.as_deref(), Some("image/gif"));
        assert_eq!(asset.file_size, Some(12345));
        assert_eq!(asset.width, Some(320));
        assert_eq!(asset.height, Some(240));
        assert!(provenance.type_validation.is_valid());
        assert!(!provenance.license_known);
        assert_eq!(
            asset
                .provider_metadata
                .get("giphy.source_url")
                .map(String::as_str),
            Some("https://giphy.com/gifs/fixture-gif")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("giphy.source_url_kind")
                .map(String::as_str),
            Some("provider-source-url")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("giphy.asset_url")
                .map(String::as_str),
            Some("https://media.giphy.com/media/fixture/giphy.gif?cid=test")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("giphy.preview_url")
                .map(String::as_str),
            Some("https://media.giphy.com/media/fixture/200.gif")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("giphy.license_evidence")
                .map(String::as_str),
            Some("not-provided-by-api-response")
        );
    }
}
