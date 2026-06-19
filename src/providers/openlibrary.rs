//! Open Library provider implementation.
//!
//! [Open Library](https://openlibrary.org/)
//!
//! 30M+ books with covers - no API key required.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Open Library provider for book covers.
/// No API key required, 30M+ books.
#[derive(Debug)]
pub struct OpenLibraryProvider {
    client: HttpClient,
}

impl OpenLibraryProvider {
    /// Create a new Open Library provider.
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

    /// Rate limit
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(100, 60);
}

#[async_trait]
impl Provider for OpenLibraryProvider {
    fn name(&self) -> &'static str {
        "openlibrary"
    }

    fn display_name(&self) -> &'static str {
        "Open Library"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image] // Book covers
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
        "https://openlibrary.org"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let count = query.count.min(100);
        let page = query.page;

        let count_str = count.to_string();
        let page_str = page.to_string();

        let url = format!("{}/search.json", self.base_url());
        let params = [
            ("q", query.query.as_str()),
            ("limit", count_str.as_str()),
            ("page", page_str.as_str()),
            ("has_fulltext", "true"),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;
        let data: OpenLibraryResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = data
            .docs
            .into_iter()
            .filter_map(|doc| {
                // Need a cover ID to get cover image
                let cover_id = doc.cover_i?;

                // Cover URLs
                let large_url = format!("https://covers.openlibrary.org/b/id/{}-L.jpg", cover_id);
                let medium_url = format!("https://covers.openlibrary.org/b/id/{}-M.jpg", cover_id);

                let authors = doc.author_name.unwrap_or_default().join(", ");
                let title_with_author = if !authors.is_empty() {
                    format!("{} by {}", doc.title, authors)
                } else {
                    doc.title.clone()
                };

                MediaAsset::builder()
                    .id(format!("openlibrary_{}", doc.key.replace("/works/", "")))
                    .provider("openlibrary")
                    .media_type(MediaType::Image)
                    .title(title_with_author)
                    .direct_download_url(large_url)
                    .preview_url(medium_url)
                    .source_url(format!("{}{}", self.base_url(), doc.key))
                    .author(authors)
                    .license(License::Other("Open Library".to_string()))
                    .tags(
                        doc.subject
                            .unwrap_or_default()
                            .into_iter()
                            .take(5)
                            .collect(),
                    )
                    .build_or_log()
            })
            .collect();

        let total = assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: data.num_found.unwrap_or(total),
            assets,
            providers_searched: vec!["openlibrary".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for OpenLibraryProvider {
    fn description(&self) -> &'static str {
        "Open Library - 30M+ book covers, no API key required"
    }

    fn api_key_url(&self) -> &'static str {
        "https://openlibrary.org/developers/api"
    }

    fn default_license(&self) -> &'static str {
        "Various (book covers)"
    }
}

/// Open Library API response structures
#[derive(Debug, Deserialize)]
struct OpenLibraryResponse {
    num_found: Option<usize>,
    docs: Vec<OpenLibraryDoc>,
}

// Fields are read during serde deserialization
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenLibraryDoc {
    key: String,
    title: String,
    author_name: Option<Vec<String>>,
    cover_i: Option<i64>,
    first_publish_year: Option<i32>,
    subject: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let config = Config::default();
        let provider = OpenLibraryProvider::new(&config);
        assert_eq!(provider.name(), "openlibrary");
        assert!(provider.is_available());
        assert!(!provider.requires_api_key());
    }
}
