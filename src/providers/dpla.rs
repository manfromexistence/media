//! Digital Public Library of America (DPLA) provider implementation.
//!
//! [DPLA API](https://pro.dp.la/developers)
//!
//! Provides access to 40+ million items from US libraries, archives, and museums.

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

/// DPLA provider for American cultural heritage.
/// Access to 40M+ items from US libraries, archives, and museums.
#[derive(Debug)]
pub struct DplaProvider {
    client: HttpClient,
}

impl DplaProvider {
    /// Create a new DPLA provider.
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

    /// Parse license from rights field
    fn parse_license(rights: Option<&str>) -> License {
        rights
            .and_then(parse_known_license_label)
            .unwrap_or_else(|| {
                License::Other(
                    rights
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or("Various")
                        .to_string(),
                )
            })
    }

    fn media_type_from_source_type(types: Option<&[String]>) -> MediaType {
        let normalized = types
            .and_then(|values| values.first())
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_default();

        if normalized.contains("image") {
            MediaType::Image
        } else if normalized.contains("sound") || normalized.contains("audio") {
            MediaType::Audio
        } else if normalized.contains("moving image") || normalized.contains("video") {
            MediaType::Video
        } else if normalized.contains("text") || normalized.contains("book") {
            MediaType::Document
        } else {
            MediaType::Image
        }
    }

    fn metadata(
        source_type: Option<&[String]>,
        rights: Option<&str>,
        object: Option<&str>,
        shown_at: Option<&str>,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        if let Some(source_type) = source_type.and_then(|values| values.first()) {
            metadata.insert("dpla.source_resource_type".to_string(), source_type.clone());
        }
        if let Some(rights) = rights.filter(|value| !value.is_empty()) {
            metadata.insert("dpla.rights".to_string(), rights.to_string());
        }
        if let Some(object) = object.filter(|value| !value.is_empty()) {
            metadata.insert("dpla.object".to_string(), object.to_string());
        }
        if let Some(shown_at) = shown_at.filter(|value| !value.is_empty()) {
            metadata.insert("dpla.is_shown_at".to_string(), shown_at.to_string());
        }
        metadata
    }
}

#[async_trait]
impl Provider for DplaProvider {
    fn name(&self) -> &'static str {
        "dpla"
    }

    fn display_name(&self) -> &'static str {
        "Digital Public Library of America"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[
            MediaType::Image,
            MediaType::Document,
            MediaType::Audio,
            MediaType::Video,
        ]
    }

    fn requires_api_key(&self) -> bool {
        true // DPLA requires API key (free to obtain at https://pro.dp.la/developers/policies)
    }

    fn rate_limit(&self) -> RateLimitConfig {
        Self::RATE_LIMIT
    }

    fn is_available(&self) -> bool {
        // Requires DPLA_API_KEY environment variable
        std::env::var("DPLA_API_KEY").is_ok()
    }

    fn base_url(&self) -> &'static str {
        "https://api.dp.la/v2"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let api_key =
            std::env::var("DPLA_API_KEY").map_err(|_| crate::error::DxError::MissingApiKey {
                provider: "dpla".to_string(),
                env_var: "DPLA_API_KEY".to_string(),
            })?;

        let url = format!("{}/items", self.base_url());

        let page_size = query.count.min(500).to_string();
        let page_str = query.page.to_string();

        let params = [
            ("q", query.query.as_str()),
            ("page_size", page_size.as_str()),
            ("page", page_str.as_str()),
            ("api_key", api_key.as_str()),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: DplaSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .docs
            .into_iter()
            .filter_map(|doc| {
                let preview = doc.object.as_ref()?.clone();
                let title = doc.sourceResource.title.as_ref()?.first()?.clone();
                let media_type =
                    Self::media_type_from_source_type(doc.sourceResource.item_type.as_deref());
                if let Some(requested_type) = query.media_type {
                    if media_type != requested_type {
                        return None;
                    }
                }
                let source_url = doc.isShownAt.clone()?;
                let rights = doc
                    .sourceResource
                    .rights
                    .as_ref()
                    .and_then(|v| v.first().map(String::as_str));
                let mut metadata = Self::metadata(
                    doc.sourceResource.item_type.as_deref(),
                    rights,
                    doc.object.as_deref(),
                    doc.isShownAt.as_deref(),
                );
                metadata.insert(
                    "dpla.download_url_kind".to_string(),
                    DownloadUrlKind::Unknown.as_str().to_string(),
                );

                Some(
                    MediaAsset::builder()
                        .id(doc.id.clone())
                        .provider("dpla")
                        .media_type(media_type)
                        .title(title)
                        .download_url(preview.clone())
                        .download_url_kind(DownloadUrlKind::Unknown)
                        .preview_url(preview)
                        .source_url(source_url)
                        .author(doc.sourceResource.creator.unwrap_or_default().join(", "))
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
            total_count: api_response.count.unwrap_or(0),
            assets,
            providers_searched: vec!["dpla".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for DplaProvider {
    fn description(&self) -> &'static str {
        "Digital Public Library of America - 40M+ items from US libraries and museums"
    }

    fn api_key_url(&self) -> &'static str {
        "https://pro.dp.la/developers/policies#get-a-key"
    }

    fn default_license(&self) -> &'static str {
        "Various (Public Domain, CC)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct DplaSearchResponse {
    count: Option<usize>,
    docs: Vec<DplaDoc>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DplaDoc {
    id: String,
    object: Option<String>,
    isShownAt: Option<String>,
    sourceResource: DplaSourceResource,
}

#[derive(Debug, Deserialize)]
struct DplaSourceResource {
    title: Option<Vec<String>>,
    creator: Option<Vec<String>>,
    rights: Option<Vec<String>>,
    #[serde(rename = "type")]
    item_type: Option<Vec<String>>,
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
        let provider = DplaProvider::new(&config);

        assert_eq!(provider.name(), "dpla");
        assert_eq!(provider.display_name(), "Digital Public Library of America");
        assert!(provider.requires_api_key());
        // Without API key, provider is not available
        assert!(!provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default();
        let provider = DplaProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
        assert!(types.contains(&MediaType::Document));
    }

    #[test]
    fn test_license_parsing() {
        assert!(matches!(
            DplaProvider::parse_license(Some("Public Domain")),
            License::PublicDomain
        ));
        assert!(matches!(
            DplaProvider::parse_license(Some("CC0")),
            License::Cc0
        ));
        assert!(matches!(
            DplaProvider::parse_license(Some("CC BY 4.0")),
            License::CcBy
        ));
    }

    #[test]
    fn unsupported_cc_variants_are_preserved_as_unmodeled() {
        assert!(matches!(
            DplaProvider::parse_license(Some("CC BY-ND 4.0")),
            License::Other(value) if value.contains("CC BY-ND")
        ));
        assert!(matches!(
            DplaProvider::parse_license(Some("CC BY-NC-SA 4.0")),
            License::Other(value) if value.contains("CC BY-NC-SA")
        ));
    }

    #[test]
    fn test_media_type_from_source_type() {
        assert_eq!(
            DplaProvider::media_type_from_source_type(Some(&["text".to_string()])),
            MediaType::Document
        );
        assert_eq!(
            DplaProvider::media_type_from_source_type(Some(&["sound".to_string()])),
            MediaType::Audio
        );
    }
}
