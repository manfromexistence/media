//! Library of Congress provider implementation.
//!
//! [Library of Congress API](https://loc.gov/apis)
//!
//! Provides access to Library of Congress image records with item-level rights statements.

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Library of Congress provider for historical image media.
#[derive(Debug)]
pub struct LibraryOfCongressProvider {
    client: HttpClient,
}

impl LibraryOfCongressProvider {
    /// Create a new Library of Congress provider.
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

    fn rights_statement(value: Option<&Value>) -> Option<String> {
        fn value_text(value: &Value) -> Option<String> {
            match value {
                Value::String(text) => {
                    Some(text.trim().to_string()).filter(|text| !text.is_empty())
                }
                Value::Array(values) => {
                    let parts: Vec<String> = values.iter().filter_map(value_text).collect();
                    Some(parts.join("; ")).filter(|text| !text.is_empty())
                }
                Value::Object(map) => ["title", "text", "rights", "label"]
                    .into_iter()
                    .find_map(|key| map.get(key).and_then(value_text))
                    .or_else(|| Some(value.to_string()).filter(|text| !text.is_empty())),
                Value::Null => None,
                _ => Some(value.to_string()).filter(|text| !text.is_empty()),
            }
        }

        value.and_then(value_text)
    }

    fn license_from_rights(rights: Option<&str>) -> License {
        let Some(rights) = rights.filter(|value| !value.trim().is_empty()) else {
            return License::Other("Rights status not verified".to_string());
        };
        let normalized = rights.to_ascii_lowercase();

        if normalized.contains("cc0") {
            License::Cc0
        } else if normalized.contains("public domain") {
            License::PublicDomain
        } else {
            License::Other(rights.to_string())
        }
    }

    fn provider_metadata(rights: Option<&str>) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        match rights {
            Some(statement) if !statement.trim().is_empty() => {
                metadata.insert("loc.rights_statement".to_string(), statement.to_string());
            }
            _ => {
                metadata.insert("loc.rights_status".to_string(), "not-provided".to_string());
            }
        }
        metadata
    }
}

#[async_trait]
impl Provider for LibraryOfCongressProvider {
    fn name(&self) -> &'static str {
        "loc"
    }

    fn display_name(&self) -> &'static str {
        "Library of Congress"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image]
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
        "https://loc.gov"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        if !matches!(query.media_type, None | Some(MediaType::Image)) {
            return Ok(SearchResult {
                query: query.query.clone(),
                media_type: query.media_type,
                total_count: 0,
                assets: Vec::new(),
                providers_searched: vec!["loc".to_string()],
                provider_errors: vec![],
                duration_ms: 0,
                provider_timings: Default::default(),
            });
        }

        let url = format!("{}/search/", self.base_url());

        let format_filter = "photo,print,drawing";

        let count_str = query.count.min(100).to_string();
        let page_str = query.page.to_string();

        let params = [
            ("q", query.query.as_str()),
            ("fo", "json"),
            ("fa", format_filter),
            ("c", count_str.as_str()),
            ("sp", page_str.as_str()),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: LocSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .results
            .into_iter()
            .filter_map(|item| {
                let image_url = item.image_url.first().cloned()?;
                let rights = Self::rights_statement(item.rights.as_ref());
                let license = Self::license_from_rights(rights.as_deref());
                let metadata = Self::provider_metadata(rights.as_deref());

                Some(
                    MediaAsset::builder()
                        .id(item.id.unwrap_or_default())
                        .provider("loc")
                        .media_type(MediaType::Image)
                        .title(
                            item.title
                                .unwrap_or_else(|| "Library of Congress Item".to_string()),
                        )
                        .direct_download_url(image_url.clone())
                        .preview_url(image_url)
                        .source_url(item.url.unwrap_or_default())
                        .author(item.contributor.unwrap_or_default().join(", "))
                        .license(license)
                        .provider_metadata(metadata)
                        .build_or_log(),
                )
            })
            .flatten()
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.pagination.total.unwrap_or(0),
            assets,
            providers_searched: vec!["loc".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for LibraryOfCongressProvider {
    fn description(&self) -> &'static str {
        "The Library of Congress - images, maps, documents, audio, and video with item-level rights statements when provided"
    }

    fn api_key_url(&self) -> &'static str {
        "https://loc.gov/apis"
    }

    fn default_license(&self) -> &'static str {
        "Varies by item rights statement"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct LocSearchResponse {
    results: Vec<LocItem>,
    pagination: LocPagination,
}

#[derive(Debug, Deserialize)]
struct LocItem {
    id: Option<String>,
    title: Option<String>,
    url: Option<String>,
    #[serde(default)]
    image_url: Vec<String>,
    #[serde(default)]
    contributor: Option<Vec<String>>,
    #[serde(default)]
    rights: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct LocPagination {
    total: Option<usize>,
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
        let provider = LibraryOfCongressProvider::new(&config);

        assert_eq!(provider.name(), "loc");
        assert_eq!(provider.display_name(), "Library of Congress");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default();
        let provider = LibraryOfCongressProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
        assert!(!types.contains(&MediaType::Document));
    }

    #[test]
    fn test_license_from_rights_requires_evidence() {
        assert!(matches!(
            LibraryOfCongressProvider::license_from_rights(None),
            License::Other(value) if value == "Rights status not verified"
        ));
        assert!(matches!(
            LibraryOfCongressProvider::license_from_rights(Some("Public Domain")),
            License::PublicDomain
        ));
        assert!(matches!(
            LibraryOfCongressProvider::license_from_rights(Some("No known restrictions on publication")),
            License::Other(value) if value == "No known restrictions on publication"
        ));
    }

    #[test]
    fn test_rights_metadata_records_missing_or_supplied_statement() {
        let missing = LibraryOfCongressProvider::provider_metadata(None);
        assert_eq!(
            missing.get("loc.rights_status").map(String::as_str),
            Some("not-provided")
        );

        let supplied = LibraryOfCongressProvider::provider_metadata(Some("Public Domain"));
        assert_eq!(
            supplied.get("loc.rights_statement").map(String::as_str),
            Some("Public Domain")
        );
    }

    #[tokio::test]
    async fn test_unsupported_media_type_returns_empty_without_network() {
        let config = Config::default();
        let provider = LibraryOfCongressProvider::new(&config);
        let query = SearchQuery::for_type("manuscript", MediaType::Document);

        let result = provider
            .search(&query)
            .await
            .expect("unsupported media type should return an empty result");

        assert_eq!(result.total_count, 0);
        assert!(result.assets.is_empty());
        assert_eq!(result.providers_searched, vec!["loc".to_string()]);
    }
}
