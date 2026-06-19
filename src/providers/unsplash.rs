//! Unsplash provider implementation.
//!
//! [Unsplash API Documentation](https://unsplash.com/documentation)

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Unsplash provider for high-resolution photography.
#[derive(Debug)]
pub struct UnsplashProvider {
    api_key: Option<String>,
    client: HttpClient,
}

impl UnsplashProvider {
    /// Create a new Unsplash provider.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let client = HttpClient::with_config(
            Self::RATE_LIMIT,
            config.retry_attempts,
            Duration::from_secs(config.timeout_secs),
        )
        .unwrap_or_default();

        Self {
            api_key: config.unsplash_api_key.clone(),
            client,
        }
    }

    /// Rate limit: 50 requests per hour
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(50, 3600);

    /// Build authorization header.
    #[allow(dead_code)] // May be used in future methods
    fn auth_header(&self) -> Option<(&'static str, String)> {
        self.api_key
            .as_ref()
            .map(|key| ("Authorization", format!("Client-ID {key}")))
    }

    fn asset_from_photo(photo: UnsplashPhoto) -> Option<MediaAsset> {
        let download_url = photo.urls.full.clone();
        let preview_url = photo.urls.small.clone();
        let provider_metadata = Self::photo_metadata(&photo, &download_url, &preview_url);

        MediaAsset::builder()
            .id(photo.id.clone())
            .provider("unsplash")
            .media_type(MediaType::Image)
            .title(
                photo
                    .description
                    .or(photo.alt_description)
                    .unwrap_or_else(|| "Unsplash Photo".to_string()),
            )
            .direct_download_url(download_url.clone())
            .preview_url(preview_url)
            .source_url(photo.links.html)
            .author(photo.user.name)
            .author_url(photo.user.links.html)
            .license(License::Unsplash)
            .dimensions(photo.width, photo.height)
            .maybe_url_inferred_mime_type(Self::mime_type_from_url(&download_url))
            .provider_metadata(provider_metadata)
            .tags(photo.tags.into_iter().map(|tag| tag.title).collect())
            .build_or_log()
    }

    fn photo_metadata(
        photo: &UnsplashPhoto,
        selected_download_url: &str,
        preview_url: &str,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::from([
            ("unsplash.asset_id".to_string(), photo.id.clone()),
            ("unsplash.page_url".to_string(), photo.links.html.clone()),
            (
                "unsplash.api_download_url".to_string(),
                photo.links.download.clone(),
            ),
            (
                "unsplash.selected_download_url".to_string(),
                selected_download_url.to_string(),
            ),
            ("unsplash.preview_url".to_string(), preview_url.to_string()),
            ("unsplash.user_name".to_string(), photo.user.name.clone()),
            (
                "unsplash.item_license_field".to_string(),
                "not-provided-by-api".to_string(),
            ),
            (
                "unsplash.license_scope".to_string(),
                "provider-default".to_string(),
            ),
        ]);
        if let Some(download_location) = photo
            .links
            .download_location
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            metadata.insert(
                "unsplash.download_location".to_string(),
                download_location.clone(),
            );
        }
        if let Some(user_id) = photo
            .user
            .id
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            metadata.insert("unsplash.user_id".to_string(), user_id.clone());
        }
        if let Some(username) = photo
            .user
            .username
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            metadata.insert("unsplash.username".to_string(), username.clone());
        }
        metadata
    }

    fn mime_type_from_url(url: &str) -> Option<&'static str> {
        let lower = url.to_ascii_lowercase();
        let path = lower.split('?').next().unwrap_or(&lower);
        if path.ends_with(".jpg") || path.ends_with(".jpeg") {
            return Some("image/jpeg");
        }
        if path.ends_with(".png") {
            return Some("image/png");
        }
        if path.ends_with(".webp") {
            return Some("image/webp");
        }

        lower.split('?').nth(1).and_then(|query| {
            query.split('&').find_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                if key == "fm" {
                    match value {
                        "jpg" | "jpeg" => Some("image/jpeg"),
                        "png" => Some("image/png"),
                        "webp" => Some("image/webp"),
                        _ => None,
                    }
                } else {
                    None
                }
            })
        })
    }
}

#[async_trait]
impl Provider for UnsplashProvider {
    fn name(&self) -> &'static str {
        "unsplash"
    }

    fn display_name(&self) -> &'static str {
        "Unsplash"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image]
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
        "https://api.unsplash.com"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let Some(ref api_key) = self.api_key else {
            return Err(crate::error::DxError::MissingApiKey {
                provider: "unsplash".to_string(),
                env_var: "UNSPLASH_ACCESS_KEY".to_string(),
            });
        };

        let url = format!("{}/search/photos", self.base_url());

        let params = UnsplashSearchParams {
            query: &query.query,
            page: query.page,
            per_page: query.count.min(30), // Unsplash max is 30
            orientation: query.orientation.as_ref().map(|o| o.to_string()),
            color: query.color.clone(),
        };

        let headers = [("Authorization", format!("Client-ID {api_key}"))];
        let headers_ref: Vec<(&str, &str)> =
            headers.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response = self
            .client
            .get_with_query(&url, &params, &headers_ref)
            .await?;

        let api_response: UnsplashSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .results
            .into_iter()
            .filter_map(Self::asset_from_photo)
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.total,
            assets,
            providers_searched: vec!["unsplash".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for UnsplashProvider {
    fn description(&self) -> &'static str {
        "High-resolution photography from talented photographers worldwide"
    }

    fn api_key_url(&self) -> &'static str {
        "https://unsplash.com/developers"
    }

    fn default_license(&self) -> &'static str {
        "Unsplash License (free for commercial and personal use)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct UnsplashSearchParams<'a> {
    query: &'a str,
    page: usize,
    per_page: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    orientation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}

impl serde::Serialize for UnsplashSearchParams<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("query", self.query)?;
        map.serialize_entry("page", &self.page)?;
        map.serialize_entry("per_page", &self.per_page)?;
        if let Some(ref o) = self.orientation {
            map.serialize_entry("orientation", o)?;
        }
        if let Some(ref c) = self.color {
            map.serialize_entry("color", c)?;
        }
        map.end()
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used for JSON deserialization
struct UnsplashSearchResponse {
    total: usize,
    total_pages: usize,
    results: Vec<UnsplashPhoto>,
}

#[derive(Debug, Deserialize)]
struct UnsplashPhoto {
    id: String,
    width: u32,
    height: u32,
    description: Option<String>,
    alt_description: Option<String>,
    urls: UnsplashUrls,
    links: UnsplashLinks,
    user: UnsplashUser,
    #[serde(default)]
    tags: Vec<UnsplashTag>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used for JSON deserialization
struct UnsplashUrls {
    raw: String,
    full: String,
    regular: String,
    small: String,
    thumb: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used for JSON deserialization
struct UnsplashLinks {
    html: String,
    download: String,
    #[serde(default)]
    download_location: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UnsplashUser {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    username: Option<String>,
    name: String,
    links: UnsplashUserLinks,
}

#[derive(Debug, Deserialize)]
struct UnsplashUserLinks {
    html: String,
}

#[derive(Debug, Deserialize)]
struct UnsplashTag {
    title: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = UnsplashProvider::new(&config);

        assert_eq!(provider.name(), "unsplash");
        assert_eq!(provider.display_name(), "Unsplash");
        assert!(provider.requires_api_key());
        assert!(!provider.is_available()); // No API key in test config
    }

    #[test]
    fn unsplash_photo_asset_preserves_provider_metadata_and_type_evidence() {
        let photo = UnsplashPhoto {
            id: "abc123".to_string(),
            width: 3000,
            height: 2000,
            description: Some("Mountain lake".to_string()),
            alt_description: Some("A mountain lake at sunrise".to_string()),
            urls: UnsplashUrls {
                raw: "https://images.unsplash.com/photo-abc123?ixid=raw".to_string(),
                full: "https://images.unsplash.com/photo-abc123?fm=jpg&w=3000".to_string(),
                regular: "https://images.unsplash.com/photo-abc123?fm=jpg&w=1080".to_string(),
                small: "https://images.unsplash.com/photo-abc123?fm=jpg&w=400".to_string(),
                thumb: "https://images.unsplash.com/photo-abc123?fm=jpg&w=200".to_string(),
            },
            links: UnsplashLinks {
                html: "https://unsplash.com/photos/abc123".to_string(),
                download: "https://unsplash.com/photos/abc123/download".to_string(),
                download_location: Some(
                    "https://api.unsplash.com/photos/abc123/download".to_string(),
                ),
            },
            user: UnsplashUser {
                id: Some("user-1".to_string()),
                username: Some("jane".to_string()),
                name: "Jane Unsplash".to_string(),
                links: UnsplashUserLinks {
                    html: "https://unsplash.com/@jane".to_string(),
                },
            },
            tags: vec![UnsplashTag {
                title: "mountain".to_string(),
            }],
        };

        let asset = UnsplashProvider::asset_from_photo(photo).expect("valid unsplash photo asset");
        let provenance = asset.provenance();

        assert_eq!(
            provenance
                .provider_metadata
                .get("unsplash.asset_id")
                .map(String::as_str),
            Some("abc123")
        );
        assert_eq!(
            provenance
                .provider_metadata
                .get("unsplash.download_location")
                .map(String::as_str),
            Some("https://api.unsplash.com/photos/abc123/download")
        );
        assert_eq!(
            provenance
                .provider_metadata
                .get("unsplash.item_license_field")
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
}
