//! DiceBear Avatars provider implementation.
//!
//! [DiceBear](https://www.dicebear.com/)
//!
//! Free avatar generation API - unlimited SVG/PNG avatars, no API key required.
//! Avatar style licenses vary; preserve the DiceBear license overview URL in metadata.

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::HttpClient;
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// DiceBear Avatars provider for generated avatars.
/// No API key required, unlimited generation.
#[derive(Debug)]
pub struct DiceBearProvider {
    #[allow(dead_code)]
    client: HttpClient,
}

impl DiceBearProvider {
    /// Create a new DiceBear provider.
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

    /// Rate limit: Generous
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(1000, 60);

    /// Available avatar styles
    const STYLES: &'static [&'static str] = &[
        "adventurer",
        "adventurer-neutral",
        "avataaars",
        "avataaars-neutral",
        "big-ears",
        "big-ears-neutral",
        "big-smile",
        "bottts",
        "bottts-neutral",
        "croodles",
        "croodles-neutral",
        "fun-emoji",
        "icons",
        "identicon",
        "initials",
        "lorelei",
        "lorelei-neutral",
        "micah",
        "miniavs",
        "notionists",
        "notionists-neutral",
        "open-peeps",
        "personas",
        "pixel-art",
        "pixel-art-neutral",
        "rings",
        "shapes",
        "thumbs",
    ];

    const LICENSE_OVERVIEW_URL: &'static str = "https://www.dicebear.com/licenses/";

    fn provider_license() -> License {
        License::Other("Varies by DiceBear style license".to_string())
    }
}

#[async_trait]
impl Provider for DiceBearProvider {
    fn name(&self) -> &'static str {
        "dicebear"
    }

    fn display_name(&self) -> &'static str {
        "DiceBear Avatars"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image, MediaType::Vector]
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
        "https://api.dicebear.com/9.x"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        if !matches!(
            query.media_type,
            None | Some(MediaType::Image | MediaType::Vector)
        ) {
            return Ok(SearchResult {
                query: query.query.clone(),
                media_type: query.media_type,
                total_count: 0,
                assets: Vec::new(),
                providers_searched: vec!["dicebear".to_string()],
                provider_errors: vec![],
                duration_ms: 0,
                provider_timings: Default::default(),
            });
        }

        let count = query.count.min(50);
        let seed_base = &query.query;
        let requested_type = query.media_type.unwrap_or(MediaType::Image);

        // Generate avatars using different styles and seeds
        let mut assets = Vec::with_capacity(count);

        for i in 0..count {
            let style_idx = i % Self::STYLES.len();
            let style = Self::STYLES[style_idx];
            let seed = format!("{}_{}", seed_base, i);

            let svg_url = format!("{}/{}/svg?seed={}", self.base_url(), style, seed);
            let png_url = format!("{}/{}/png?seed={}&size=200", self.base_url(), style, seed);
            let (media_type, download_url, mime_type, format) = match requested_type {
                MediaType::Vector => (MediaType::Vector, svg_url.clone(), "image/svg+xml", "svg"),
                _ => (MediaType::Image, png_url.clone(), "image/png", "png"),
            };
            let provider_metadata = HashMap::from([
                ("dicebear.style".to_string(), style.to_string()),
                ("dicebear.seed".to_string(), seed.clone()),
                ("dicebear.download_format".to_string(), format.to_string()),
                ("dicebear.svg_url".to_string(), svg_url),
                ("dicebear.png_url".to_string(), png_url.clone()),
                (
                    "dicebear.license_status".to_string(),
                    "varies-by-style".to_string(),
                ),
                (
                    "dicebear.license_overview_url".to_string(),
                    Self::LICENSE_OVERVIEW_URL.to_string(),
                ),
                (
                    "dicebear.style_license_mapped".to_string(),
                    "false".to_string(),
                ),
            ]);

            if let Some(asset) = MediaAsset::builder()
                .id(format!("dicebear_{}_{}", style, i))
                .provider("dicebear")
                .media_type(media_type)
                .title(format!("{} avatar - {}", style, seed))
                .direct_download_url(download_url)
                .preview_url(png_url)
                .source_url(format!("https://www.dicebear.com/styles/{}", style))
                .license(Self::provider_license())
                .url_inferred_mime_type(mime_type)
                .provider_metadata(provider_metadata)
                .tags(vec![
                    "avatar".to_string(),
                    style.to_string(),
                    "generated".to_string(),
                    format.to_string(),
                ])
                .build_or_log()
            {
                assets.push(asset);
            }
        }

        let total = assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: total,
            assets,
            providers_searched: vec!["dicebear".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for DiceBearProvider {
    fn description(&self) -> &'static str {
        "Free avatar generation - 25+ styles, unlimited SVG/PNG avatars"
    }

    fn api_key_url(&self) -> &'static str {
        "https://www.dicebear.com/"
    }

    fn default_license(&self) -> &'static str {
        "Varies by DiceBear style license"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MimeEvidenceSource;

    #[test]
    fn test_provider_info() {
        let config = Config::default();
        let provider = DiceBearProvider::new(&config);
        assert_eq!(provider.name(), "dicebear");
        assert!(provider.is_available());
        assert!(!provider.requires_api_key());
    }

    #[test]
    fn test_styles_available() {
        assert!(!DiceBearProvider::STYLES.is_empty());
        assert!(DiceBearProvider::STYLES.contains(&"avataaars"));
    }

    #[tokio::test]
    async fn test_search_uses_mime_consistent_download_urls() {
        let config = Config::default();
        let provider = DiceBearProvider::new(&config);

        let image_result = provider
            .search(&SearchQuery::for_type("ada", MediaType::Image).count(1))
            .await
            .expect("image search should be generated locally");
        let image = &image_result.assets[0];
        assert_eq!(image.media_type, MediaType::Image);
        assert_eq!(image.mime_type.as_deref(), Some("image/png"));
        assert_eq!(
            image.mime_evidence_source,
            Some(MimeEvidenceSource::UrlInferred)
        );
        assert!(image.download_url.contains("/png?"));
        assert!(image.tags.contains(&"png".to_string()));
        assert!(!image.tags.contains(&"svg".to_string()));
        assert!(image.validate_type_metadata().is_valid());

        let vector_result = provider
            .search(&SearchQuery::for_type("ada", MediaType::Vector).count(1))
            .await
            .expect("vector search should be generated locally");
        let vector = &vector_result.assets[0];
        assert_eq!(vector.media_type, MediaType::Vector);
        assert_eq!(vector.mime_type.as_deref(), Some("image/svg+xml"));
        assert_eq!(
            vector.mime_evidence_source,
            Some(MimeEvidenceSource::UrlInferred)
        );
        assert!(vector.download_url.contains("/svg?"));
        assert!(vector.tags.contains(&"svg".to_string()));
        assert_eq!(
            vector
                .provider_metadata
                .get("dicebear.download_format")
                .map(String::as_str),
            Some("svg")
        );
        assert!(vector.validate_type_metadata().is_valid());
    }

    #[tokio::test]
    async fn test_search_does_not_claim_blanket_cc0_for_style_licenses() {
        let config = Config::default();
        let provider = DiceBearProvider::new(&config);

        let result = provider
            .search(&SearchQuery::for_type("license", MediaType::Image).count(1))
            .await
            .expect("image search should be generated locally");
        let asset = &result.assets[0];
        let provenance = asset.provenance();

        assert!(!matches!(asset.license, License::Cc0));
        assert!(!provenance.license_known);
        assert_eq!(
            asset
                .provider_metadata
                .get("dicebear.license_status")
                .map(String::as_str),
            Some("varies-by-style")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("dicebear.license_overview_url")
                .map(String::as_str),
            Some("https://www.dicebear.com/licenses/")
        );
    }
}
