//! Victoria and Albert Museum provider implementation.
//!
//! [V&A Museum](https://www.vam.ac.uk/)
//!
//! 1.2M+ art and design objects - no API key required.

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

/// Victoria and Albert Museum provider.
/// No API key required, 1.2M+ objects.
#[derive(Debug)]
pub struct VandAMuseumProvider {
    client: HttpClient,
}

impl VandAMuseumProvider {
    /// Create a new V&A Museum provider.
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
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(30, 60);

    fn asset_from_record(record: VandARecord) -> Option<MediaAsset> {
        let image = record._images.as_ref()?.iiif_image_base_url.as_ref()?;
        let image_url = format!("{}/full/800,/0/default.jpg", image);
        let preview_url = format!("{}/full/400,/0/default.jpg", image);
        let mut metadata = direct_asset_metadata("vanda", &image_url);
        metadata.insert(
            "vanda.system_number".to_string(),
            record.system_number.clone(),
        );

        MediaAsset::builder()
            .id(format!("vanda_{}", record.system_number))
            .provider("vanda")
            .media_type(MediaType::Image)
            .title(
                record
                    ._primary_title
                    .unwrap_or_else(|| "Untitled".to_string()),
            )
            .direct_download_url(image_url.clone())
            .preview_url(preview_url)
            .source_url(format!(
                "https://collections.vam.ac.uk/item/{}",
                record.system_number
            ))
            .license(license_not_provided())
            .maybe_url_inferred_mime_type(mime_type_from_url(MediaType::Image, &image_url))
            .provider_metadata(metadata)
            .tags(record.object_type.map(|t| vec![t]).unwrap_or_default())
            .build_or_log()
    }
}

#[async_trait]
impl Provider for VandAMuseumProvider {
    fn name(&self) -> &'static str {
        "vanda"
    }

    fn display_name(&self) -> &'static str {
        "V&A Museum"
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
        "https://api.vam.ac.uk/v2"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let count = query.count.min(100);
        let page = query.page;

        let count_str = count.to_string();
        let page_str = page.to_string();

        let url = format!("{}/objects/search", self.base_url());
        let params = [
            ("q", query.query.as_str()),
            ("page", page_str.as_str()),
            ("page_size", count_str.as_str()),
            ("images_exist", "true"),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;
        let data: VandAResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = data
            .records
            .into_iter()
            .filter_map(Self::asset_from_record)
            .collect();

        let total = assets.len();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: data.info.record_count.unwrap_or(total),
            assets,
            providers_searched: vec!["vanda".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for VandAMuseumProvider {
    fn description(&self) -> &'static str {
        "Victoria and Albert Museum - 1.2M+ art and design objects"
    }

    fn api_key_url(&self) -> &'static str {
        "https://www.vam.ac.uk/api"
    }

    fn default_license(&self) -> &'static str {
        crate::providers::provenance::LICENSE_NOT_PROVIDED
    }
}

/// V&A API response structures
#[derive(Debug, Deserialize)]
struct VandAResponse {
    info: VandAInfo,
    records: Vec<VandARecord>,
}

#[derive(Debug, Deserialize)]
struct VandAInfo {
    record_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct VandARecord {
    #[serde(rename = "systemNumber")]
    system_number: String,
    #[serde(rename = "_primaryTitle")]
    _primary_title: Option<String>,
    #[serde(rename = "_primaryPlace")]
    _primary_place: Option<String>,
    #[serde(rename = "_images")]
    _images: Option<VandAImages>,
    #[serde(rename = "objectType")]
    object_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VandAImages {
    #[serde(rename = "_iiif_image_base_url")]
    iiif_image_base_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let config = Config::default();
        let provider = VandAMuseumProvider::new(&config);
        assert_eq!(provider.name(), "vanda");
        assert!(provider.is_available());
        assert!(!provider.requires_api_key());
    }

    #[test]
    fn asset_from_record_marks_license_unproven_and_preserves_source_metadata() {
        let config = Config::default();
        let provider = VandAMuseumProvider::new(&config);
        let default_license = crate::types::License::Other(provider.default_license().to_string());
        let record = VandARecord {
            system_number: "O12345".to_string(),
            _primary_title: Some("Chair".to_string()),
            _primary_place: None,
            _images: Some(VandAImages {
                iiif_image_base_url: Some(
                    "https://framemark.vam.ac.uk/collections/123".to_string(),
                ),
            }),
            object_type: Some("Furniture".to_string()),
        };

        assert_eq!(
            provider.default_license(),
            crate::providers::provenance::LICENSE_NOT_PROVIDED
        );
        assert!(
            !default_license.is_known(),
            "V&A default license should not be receipt-known without item-level rights evidence"
        );

        let asset = VandAMuseumProvider::asset_from_record(record).expect("record should map");
        let provenance = asset.provenance();

        assert_eq!(
            asset.license.as_str(),
            crate::providers::provenance::LICENSE_NOT_PROVIDED
        );
        assert!(!provenance.license_known);
        assert_eq!(asset.mime_type.as_deref(), Some("image/jpeg"));
        assert!(provenance.type_validation.is_valid());
        assert_eq!(
            provenance.source_url,
            "https://collections.vam.ac.uk/item/O12345"
        );
        assert_eq!(
            provenance
                .provider_metadata
                .get("vanda.license_evidence")
                .map(String::as_str),
            Some(crate::providers::provenance::LICENSE_EVIDENCE_NOT_PROVIDED)
        );
        assert_eq!(
            provenance
                .provider_metadata
                .get("vanda.source_url_kind")
                .map(String::as_str),
            Some("direct-asset-url")
        );
        assert_eq!(
            provenance
                .provider_metadata
                .get("vanda.system_number")
                .map(String::as_str),
            Some("O12345")
        );
    }
}
