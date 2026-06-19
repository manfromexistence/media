//! Europeana provider implementation.
//!
//! [Europeana API](https://pro.europeana.eu/page/apis)
//!
//! Provides access to 50+ million cultural heritage items from European institutions.

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::provenance::parse_known_license_label;
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{
    DownloadUrlKind, License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult,
};

/// Europeana provider for European cultural heritage.
/// Access to 50M+ images, documents, videos, and audio from European museums and archives.
#[derive(Debug)]
pub struct EuropeanaProvider {
    client: HttpClient,
}

impl EuropeanaProvider {
    /// Create a new Europeana provider.
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

    /// Rate limit: 10000 requests per day (generous)
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(400, 3600);

    /// Parse license from rights URL
    fn parse_license(rights: Option<&str>) -> License {
        if let Some(rights) = rights {
            if rights.contains("InC") {
                return License::Other("In Copyright".to_string());
            }

            parse_known_license_label(rights).unwrap_or_else(|| License::Other(rights.to_string()))
        } else {
            License::Other("Various".to_string())
        }
    }

    fn media_type_from_europeana_type(item_type: Option<&str>, fallback: MediaType) -> MediaType {
        match item_type.unwrap_or_default().to_ascii_uppercase().as_str() {
            "IMAGE" => MediaType::Image,
            "VIDEO" => MediaType::Video,
            "SOUND" => MediaType::Audio,
            "TEXT" => MediaType::Document,
            _ => fallback,
        }
    }

    fn metadata(
        item_type: Option<&str>,
        rights: Option<&str>,
        preview: Option<&str>,
        download: Option<&str>,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        if let Some(item_type) = item_type.filter(|value| !value.is_empty()) {
            metadata.insert("europeana.type".to_string(), item_type.to_string());
        }
        if let Some(rights) = rights.filter(|value| !value.is_empty()) {
            metadata.insert("europeana.rights".to_string(), rights.to_string());
        }
        if let Some(preview) = preview.filter(|value| !value.is_empty()) {
            metadata.insert("europeana.edm_preview".to_string(), preview.to_string());
        }
        if let Some(download) = download.filter(|value| !value.is_empty()) {
            metadata.insert(
                "europeana.edm_is_shown_by".to_string(),
                download.to_string(),
            );
        }
        metadata
    }
}

#[async_trait]
impl Provider for EuropeanaProvider {
    fn name(&self) -> &'static str {
        "europeana"
    }

    fn display_name(&self) -> &'static str {
        "Europeana"
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
        true
    }

    fn base_url(&self) -> &'static str {
        "https://api.europeana.eu/record/v2"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = format!("{}/search.json", self.base_url());

        let media_filter = match query.media_type {
            Some(MediaType::Image) => "IMAGE",
            Some(MediaType::Video) => "VIDEO",
            Some(MediaType::Audio) => "SOUND",
            Some(MediaType::Document) => "TEXT",
            _ => "IMAGE",
        };

        let rows = query.count.min(100).to_string();
        let start = ((query.page - 1) * query.count + 1).to_string();

        // Europeana has a free tier API key that's publicly documented
        let params = [
            ("query", query.query.as_str()),
            ("qf", &format!("TYPE:{}", media_filter)),
            ("rows", rows.as_str()),
            ("start", start.as_str()),
            ("profile", "rich"),
            ("wskey", "api2demo"), // Public demo key
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: EuropeanaSearchResponse = response.json_or_error().await?;
        let fallback_media_type = match query.media_type {
            Some(media_type) => media_type,
            None => match media_filter {
                "VIDEO" => MediaType::Video,
                "SOUND" => MediaType::Audio,
                "TEXT" => MediaType::Document,
                _ => MediaType::Image,
            },
        };

        let assets: Vec<MediaAsset> = api_response
            .items
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                let preview = item.edmPreview.as_ref()?.first()?.clone();
                let download = item
                    .edmIsShownBy
                    .as_ref()
                    .and_then(|v| v.first())
                    .cloned()
                    .unwrap_or_else(|| preview.clone());
                let rights = item
                    .rights
                    .as_ref()
                    .and_then(|v| v.first().map(String::as_str));
                let media_type = Self::media_type_from_europeana_type(
                    item.item_type.as_deref(),
                    fallback_media_type,
                );
                let mut metadata = Self::metadata(
                    item.item_type.as_deref(),
                    rights,
                    item.edmPreview
                        .as_ref()
                        .and_then(|v| v.first().map(String::as_str)),
                    item.edmIsShownBy
                        .as_ref()
                        .and_then(|v| v.first().map(String::as_str)),
                );
                metadata.insert(
                    "europeana.download_url_kind".to_string(),
                    DownloadUrlKind::Unknown.as_str().to_string(),
                );

                Some(
                    MediaAsset::builder()
                        .id(item.id.clone())
                        .provider("europeana")
                        .media_type(media_type)
                        .title(item.title.as_ref()?.first()?.clone())
                        .download_url(download)
                        .download_url_kind(DownloadUrlKind::Unknown)
                        .preview_url(preview)
                        .source_url(item.guid?)
                        .author(item.dcCreator.unwrap_or_default().join(", "))
                        .license(Self::parse_license(rights))
                        .provider_metadata(metadata)
                        .build_or_log(),
                )
            })
            .flatten()
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.totalResults.unwrap_or(0),
            assets,
            providers_searched: vec!["europeana".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for EuropeanaProvider {
    fn description(&self) -> &'static str {
        "Europeana - 50M+ cultural heritage items from European museums and archives"
    }

    fn api_key_url(&self) -> &'static str {
        "https://pro.europeana.eu/page/get-api"
    }

    fn default_license(&self) -> &'static str {
        "Various CC licenses"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct EuropeanaSearchResponse {
    totalResults: Option<usize>,
    items: Option<Vec<EuropeanaItem>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct EuropeanaItem {
    id: String,
    #[serde(rename = "type")]
    item_type: Option<String>,
    title: Option<Vec<String>>,
    guid: Option<String>,
    edmPreview: Option<Vec<String>>,
    edmIsShownBy: Option<Vec<String>>,
    dcCreator: Option<Vec<String>>,
    rights: Option<Vec<String>>,
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
        let provider = EuropeanaProvider::new(&config);

        assert_eq!(provider.name(), "europeana");
        assert_eq!(provider.display_name(), "Europeana");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default();
        let provider = EuropeanaProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
        assert!(types.contains(&MediaType::Video));
        assert!(types.contains(&MediaType::Audio));
    }

    #[test]
    fn test_license_parsing() {
        assert!(matches!(
            EuropeanaProvider::parse_license(Some(
                "http://creativecommons.org/publicdomain/zero/1.0/"
            )),
            License::Cc0
        ));
        assert!(matches!(
            EuropeanaProvider::parse_license(Some("http://creativecommons.org/licenses/by/4.0/")),
            License::CcBy
        ));
        assert!(matches!(
            EuropeanaProvider::parse_license(Some(
                "http://creativecommons.org/licenses/by-sa/4.0/"
            )),
            License::CcBySa
        ));
    }

    #[test]
    fn unsupported_cc_variants_are_preserved_as_unmodeled() {
        assert!(matches!(
            EuropeanaProvider::parse_license(Some(
                "http://creativecommons.org/licenses/by-nd/4.0/"
            )),
            License::Other(value) if value.contains("by-nd")
        ));
        assert!(matches!(
            EuropeanaProvider::parse_license(Some(
                "http://creativecommons.org/licenses/by-nc-sa/4.0/"
            )),
            License::Other(value) if value.contains("by-nc-sa")
        ));
    }
}
