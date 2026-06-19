//! Smithsonian Open Access provider implementation.
//!
//! [Smithsonian Open Access API](https://api.si.edu/)
//!
//! Provides access to 4.5+ million CC0 licensed images from the Smithsonian.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Smithsonian Open Access provider for museum media.
/// Access to 4.5M+ CC0 licensed images and 3D models.
#[derive(Debug)]
pub struct SmithsonianProvider {
    api_key: Option<String>,
    client: HttpClient,
}

impl SmithsonianProvider {
    /// Create a new Smithsonian provider.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let client = HttpClient::with_config(
            Self::RATE_LIMIT,
            config.retry_attempts,
            Duration::from_secs(config.timeout_secs),
        )
        .unwrap_or_default();

        Self {
            api_key: std::env::var("SMITHSONIAN_API_KEY").ok(),
            client,
        }
    }

    /// Rate limit: Unlimited but be respectful
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(1000, 3600);
}

#[async_trait]
impl Provider for SmithsonianProvider {
    fn name(&self) -> &'static str {
        "smithsonian"
    }

    fn display_name(&self) -> &'static str {
        "Smithsonian Open Access"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image, MediaType::Model3D]
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
        "https://api.si.edu/openaccess/api/v1.0"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let Some(ref api_key) = self.api_key else {
            return Err(crate::error::DxError::MissingApiKey {
                provider: "smithsonian".to_string(),
                env_var: "SMITHSONIAN_API_KEY".to_string(),
            });
        };

        let url = format!("{}/search", self.base_url());

        let start = ((query.page - 1) * query.count).to_string();
        let rows = query.count.min(100).to_string();

        let params = [
            ("q", query.query.as_str()),
            ("start", &start),
            ("rows", &rows),
            ("api_key", api_key.as_str()),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: SmithsonianSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .response
            .rows
            .into_iter()
            .filter_map(|row| {
                let content = row.content?;
                let descriptive_non_repeating = content.descriptive_non_repeating?;

                // Get the first available media
                let online_media = descriptive_non_repeating.online_media?;
                let media = online_media.media.into_iter().next()?;

                let download_url = media.content.clone();
                let thumbnail = media.thumbnail.unwrap_or_else(|| download_url.clone());

                // Determine media type from content type
                let media_type =
                    if media.media_type.contains("3d") || media.media_type.contains("model") {
                        MediaType::Model3D
                    } else {
                        MediaType::Image
                    };

                let title = descriptive_non_repeating
                    .title
                    .and_then(|t| t.content)
                    .unwrap_or_else(|| "Smithsonian Item".to_string());

                let guid = descriptive_non_repeating
                    .guid
                    .unwrap_or_else(|| row.id.clone());

                MediaAsset::builder()
                    .id(row.id)
                    .provider("smithsonian")
                    .media_type(media_type)
                    .title(title)
                    .direct_download_url(download_url)
                    .preview_url(thumbnail)
                    .source_url(format!("https://www.si.edu/object/{}", guid))
                    .author("Smithsonian Institution".to_string())
                    .license(License::Cc0)
                    .build_or_log()
            })
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.response.row_count,
            assets,
            providers_searched: vec!["smithsonian".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for SmithsonianProvider {
    fn description(&self) -> &'static str {
        "Smithsonian Institution's 4.5M+ CC0 licensed images and 3D models"
    }

    fn api_key_url(&self) -> &'static str {
        "https://api.si.edu/openaccess/api/v1.0/api_key"
    }

    fn default_license(&self) -> &'static str {
        "CC0 (Public Domain)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmithsonianSearchResponse {
    response: SmithsonianResponse,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmithsonianResponse {
    #[serde(rename = "rowCount")]
    row_count: usize,
    rows: Vec<SmithsonianRow>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmithsonianRow {
    id: String,
    content: Option<SmithsonianContent>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmithsonianContent {
    #[serde(rename = "descriptiveNonRepeating")]
    descriptive_non_repeating: Option<SmithsonianDescriptive>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmithsonianDescriptive {
    title: Option<SmithsonianTitle>,
    guid: Option<String>,
    #[serde(rename = "online_media")]
    online_media: Option<SmithsonianOnlineMedia>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmithsonianTitle {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmithsonianOnlineMedia {
    media: Vec<SmithsonianMedia>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmithsonianMedia {
    content: String,
    thumbnail: Option<String>,
    #[serde(rename = "type")]
    media_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = SmithsonianProvider::new(&config);

        assert_eq!(provider.name(), "smithsonian");
        assert_eq!(provider.display_name(), "Smithsonian Open Access");
        assert!(provider.requires_api_key());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default_for_testing();
        let provider = SmithsonianProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
        assert!(types.contains(&MediaType::Model3D));
    }
}
