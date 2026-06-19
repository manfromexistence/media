//! Internet Archive provider implementation.
//!
//! [Internet Archive API](https://archive.org/developers/)
//!
//! Provides access to millions of free media items including images, audio, video, and texts.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::provenance::parse_known_license_label;
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{
    DownloadUrlKind, License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult,
};

/// Internet Archive provider for free digital content.
/// Access to millions of images, audio, video, and documents.
#[derive(Debug)]
pub struct InternetArchiveProvider {
    client: HttpClient,
}

impl InternetArchiveProvider {
    /// Create a new Internet Archive provider.
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
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(100, 60);

    /// Get the media type query filter
    fn media_type_query(media_type: Option<MediaType>) -> &'static str {
        match media_type {
            Some(MediaType::Image) => "mediatype:image",
            Some(MediaType::Video) => "mediatype:movies",
            Some(MediaType::Audio) => "mediatype:audio",
            Some(MediaType::Document) => "mediatype:texts",
            Some(MediaType::Data) => "mediatype:data",
            _ => "", // All types
        }
    }

    /// Parse media type from Internet Archive mediatype
    fn parse_media_type(mediatype: &str) -> MediaType {
        match mediatype {
            "image" => MediaType::Image,
            "movies" => MediaType::Video,
            "audio" => MediaType::Audio,
            "texts" => MediaType::Document,
            "data" => MediaType::Data,
            "software" => MediaType::Code,
            _ => MediaType::Image,
        }
    }

    /// Parse license from licenseurl or rights field
    fn parse_license(licenseurl: Option<&str>) -> License {
        licenseurl
            .and_then(parse_known_license_label)
            .unwrap_or_else(|| {
                License::Other(
                    licenseurl
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or("Various")
                        .to_string(),
                )
            })
    }
}

#[async_trait]
impl Provider for InternetArchiveProvider {
    fn name(&self) -> &'static str {
        "archive"
    }

    fn display_name(&self) -> &'static str {
        "Internet Archive"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[
            MediaType::Image,
            MediaType::Video,
            MediaType::Audio,
            MediaType::Document,
        ]
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn rate_limit(&self) -> RateLimitConfig {
        Self::RATE_LIMIT
    }

    fn is_available(&self) -> bool {
        // Temporarily disabled - API is very slow and unreliable
        false
    }

    fn base_url(&self) -> &'static str {
        "https://archive.org"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = format!("{}/advancedsearch.php", self.base_url());

        // Build search query with media type filter
        let media_filter = Self::media_type_query(query.media_type);
        let search_query = if media_filter.is_empty() {
            query.query.clone()
        } else {
            format!("{} AND {}", query.query, media_filter)
        };

        let page_str = query.page.to_string();
        let rows_str = query.count.min(100).to_string();

        let params = [
            ("q", search_query.as_str()),
            (
                "fl[]",
                "identifier,title,description,mediatype,creator,licenseurl,downloads",
            ),
            ("sort[]", "downloads desc"),
            ("rows", &rows_str),
            ("page", &page_str),
            ("output", "json"),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: ArchiveSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .response
            .docs
            .into_iter()
            .map(|doc| {
                let media_type = Self::parse_media_type(&doc.mediatype);
                let license = Self::parse_license(doc.licenseurl.as_deref());

                // Construct URLs for Internet Archive items
                let identifier = &doc.identifier;
                let source_url = format!("https://archive.org/details/{}", identifier);
                let download_url = format!("https://archive.org/download/{}", identifier);
                let preview_url = format!("https://archive.org/services/img/{}", identifier);

                MediaAsset::builder()
                    .id(identifier.clone())
                    .provider("archive")
                    .media_type(media_type)
                    .title(doc.title.unwrap_or_else(|| identifier.clone()))
                    .download_url(download_url)
                    .download_url_kind(DownloadUrlKind::AssetManifest)
                    .preview_url(preview_url)
                    .source_url(source_url)
                    .author(doc.creator.unwrap_or_else(|| "Unknown".to_string()))
                    .license(license)
                    .build_or_log()
            })
            .flatten()
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.response.num_found,
            assets,
            providers_searched: vec!["archive".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for InternetArchiveProvider {
    fn description(&self) -> &'static str {
        "Digital library with millions of free books, movies, music, and more"
    }

    fn api_key_url(&self) -> &'static str {
        "https://archive.org/developers/"
    }

    fn default_license(&self) -> &'static str {
        "Various (Public Domain, Creative Commons, etc.)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ArchiveSearchResponse {
    response: ArchiveResponse,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ArchiveResponse {
    #[serde(rename = "numFound")]
    num_found: usize,
    docs: Vec<ArchiveDoc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ArchiveDoc {
    identifier: String,
    title: Option<String>,
    description: Option<String>,
    mediatype: String,
    creator: Option<String>,
    licenseurl: Option<String>,
    downloads: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = InternetArchiveProvider::new(&config);

        assert_eq!(provider.name(), "archive");
        assert_eq!(provider.display_name(), "Internet Archive");
        assert!(!provider.requires_api_key());
        // Disabled due to slow API - is_available returns false
        assert!(!provider.is_available());
    }

    #[test]
    fn test_media_type_parsing() {
        assert_eq!(
            InternetArchiveProvider::parse_media_type("image"),
            MediaType::Image
        );
        assert_eq!(
            InternetArchiveProvider::parse_media_type("movies"),
            MediaType::Video
        );
        assert_eq!(
            InternetArchiveProvider::parse_media_type("audio"),
            MediaType::Audio
        );
        assert_eq!(
            InternetArchiveProvider::parse_media_type("texts"),
            MediaType::Document
        );
    }

    #[test]
    fn test_license_parsing() {
        assert!(matches!(
            InternetArchiveProvider::parse_license(Some(
                "https://creativecommons.org/publicdomain/zero/1.0/"
            )),
            License::Cc0
        ));
        assert!(matches!(
            InternetArchiveProvider::parse_license(Some(
                "https://creativecommons.org/licenses/by/4.0/"
            )),
            License::CcBy
        ));
    }

    #[test]
    fn unsupported_cc_variants_are_preserved_as_unmodeled() {
        assert!(matches!(
            InternetArchiveProvider::parse_license(Some(
                "https://creativecommons.org/licenses/by-nd/4.0/"
            )),
            License::Other(value) if value.contains("by-nd")
        ));
        assert!(matches!(
            InternetArchiveProvider::parse_license(Some(
                "https://creativecommons.org/licenses/by-nc-sa/4.0/"
            )),
            License::Other(value) if value.contains("by-nc-sa")
        ));
    }
}
