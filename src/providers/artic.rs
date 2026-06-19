//! Art Institute of Chicago provider implementation.
//!
//! [Art Institute of Chicago API](https://api.artic.edu/docs)
//!
//! Provides access to 50,000+ CC0 licensed artworks from the Art Institute of Chicago.

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Art Institute of Chicago provider for American art.
/// Access to 50K+ CC0 licensed artworks including masterpieces of American and European art.
#[derive(Debug)]
pub struct ArtInstituteChicagoProvider {
    client: HttpClient,
}

impl ArtInstituteChicagoProvider {
    /// Create a new Art Institute of Chicago provider.
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
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(60, 60);

    /// IIIF image base URL for the Art Institute
    const IIIF_BASE: &'static str = "https://www.artic.edu/iiif/2";

    fn rights_status(is_public_domain: Option<bool>) -> &'static str {
        match is_public_domain {
            Some(true) => "public-domain-confirmed",
            Some(false) => "not-public-domain",
            None => "not-provided",
        }
    }

    fn license_from_rights(
        is_public_domain: Option<bool>,
        copyright_notice: Option<&str>,
    ) -> License {
        match is_public_domain {
            Some(true) => License::Cc0,
            Some(false) => copyright_notice
                .filter(|notice| !notice.trim().is_empty())
                .map(|notice| License::Other(notice.to_string()))
                .unwrap_or_else(|| License::Other("Rights status not verified".to_string())),
            None => License::Other("Rights status not verified".to_string()),
        }
    }

    fn provider_metadata(artwork: &ArticArtwork, image_id: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::from([
            ("artic.image_id".to_string(), image_id.to_string()),
            (
                "artic.rights_status".to_string(),
                Self::rights_status(artwork.is_public_domain).to_string(),
            ),
        ]);

        if let Some(is_public_domain) = artwork.is_public_domain {
            metadata.insert(
                "artic.is_public_domain".to_string(),
                is_public_domain.to_string(),
            );
        }
        if let Some(notice) = artwork
            .copyright_notice
            .as_ref()
            .filter(|notice| !notice.trim().is_empty())
        {
            metadata.insert("artic.copyright_notice".to_string(), notice.clone());
        }
        metadata
    }
}

#[async_trait]
impl Provider for ArtInstituteChicagoProvider {
    fn name(&self) -> &'static str {
        "artic"
    }

    fn display_name(&self) -> &'static str {
        "Art Institute of Chicago"
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
        "https://api.artic.edu/api/v1"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = format!("{}/artworks/search", self.base_url());

        let limit = query.count.min(100).to_string();
        let page = query.page.to_string();

        let params = [
            ("q", query.query.as_str()),
            ("limit", limit.as_str()),
            ("page", page.as_str()),
            (
                "fields",
                "id,title,artist_title,image_id,thumbnail,dimensions,is_public_domain,copyright_notice",
            ),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: ArticSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .data
            .into_iter()
            .filter_map(|artwork| {
                let image_id = artwork.image_id.clone()?;
                let download_url =
                    format!("{}/{}/full/843,/0/default.jpg", Self::IIIF_BASE, image_id);
                let preview_url =
                    format!("{}/{}/full/200,/0/default.jpg", Self::IIIF_BASE, image_id);
                let license = Self::license_from_rights(
                    artwork.is_public_domain,
                    artwork.copyright_notice.as_deref(),
                );
                let metadata = Self::provider_metadata(&artwork, &image_id);

                Some(
                    MediaAsset::builder()
                        .id(artwork.id.to_string())
                        .provider("artic")
                        .media_type(MediaType::Image)
                        .title(artwork.title.unwrap_or_else(|| "Untitled".to_string()))
                        .direct_download_url(download_url)
                        .preview_url(preview_url)
                        .source_url(format!("https://www.artic.edu/artworks/{}", artwork.id))
                        .author(artwork.artist_title.unwrap_or_default())
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
            providers_searched: vec!["artic".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for ArtInstituteChicagoProvider {
    fn description(&self) -> &'static str {
        "Art Institute of Chicago artworks with API-provided public-domain flags when available"
    }

    fn api_key_url(&self) -> &'static str {
        "https://api.artic.edu/docs"
    }

    fn default_license(&self) -> &'static str {
        "CC0 when is_public_domain=true; otherwise rights status not verified"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct ArticSearchResponse {
    data: Vec<ArticArtwork>,
    pagination: ArticPagination,
}

#[derive(Debug, Deserialize)]
struct ArticArtwork {
    id: i64,
    title: Option<String>,
    artist_title: Option<String>,
    image_id: Option<String>,
    #[serde(default)]
    is_public_domain: Option<bool>,
    #[serde(default)]
    copyright_notice: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArticPagination {
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
        let provider = ArtInstituteChicagoProvider::new(&config);

        assert_eq!(provider.name(), "artic");
        assert_eq!(provider.display_name(), "Art Institute of Chicago");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default();
        let provider = ArtInstituteChicagoProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Image));
    }

    #[test]
    fn test_license_from_rights_uses_api_public_domain_flag() {
        assert!(matches!(
            ArtInstituteChicagoProvider::license_from_rights(Some(true), None),
            License::Cc0
        ));
        assert!(matches!(
            ArtInstituteChicagoProvider::license_from_rights(Some(false), Some("Copyright holder")),
            License::Other(value) if value == "Copyright holder"
        ));
        assert!(matches!(
            ArtInstituteChicagoProvider::license_from_rights(None, None),
            License::Other(value) if value == "Rights status not verified"
        ));
    }
}
