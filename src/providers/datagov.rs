//! Data.gov provider implementation.
//!
//! [Data.gov](https://data.gov) - US Government Open Data Portal
//!
//! Provides free access to 300,000+ datasets from the US Government.
//! No API key required. Includes JSON, CSV, XML, and other data formats.

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

/// Data.gov provider for US Government open data.
/// Access 300,000+ datasets including JSON, CSV, XML files.
#[derive(Debug)]
pub struct DataGovProvider {
    client: HttpClient,
}

impl DataGovProvider {
    /// Create a new Data.gov provider.
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

    /// Rate limit: Be respectful - 30 requests/minute
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(30, 60);

    /// Base URL for Data.gov CKAN API
    const BASE_URL: &'static str = "https://catalog.data.gov/api/3";

    /// Parse format to media type
    fn format_to_media_type(format: &str) -> MediaType {
        match format.to_lowercase().as_str() {
            "json" | "geojson" | "api" => MediaType::Data,
            "csv" | "tsv" | "xlsx" | "xls" => MediaType::Data,
            "xml" | "rss" | "atom" => MediaType::Data,
            "pdf" | "doc" | "docx" => MediaType::Document,
            "txt" | "html" | "htm" => MediaType::Text,
            "zip" | "gz" | "tar" => MediaType::Data,
            "kml" | "kmz" | "shp" => MediaType::Data, // Geographic data
            _ => MediaType::Data,
        }
    }

    fn mime_to_media_type(mime: &str) -> Option<MediaType> {
        let mime = mime.to_ascii_lowercase();
        if mime.contains("json")
            || mime.contains("csv")
            || mime.contains("xml")
            || mime.contains("spreadsheet")
        {
            Some(MediaType::Data)
        } else if mime == "application/pdf"
            || mime.contains("msword")
            || mime.contains("officedocument")
        {
            Some(MediaType::Document)
        } else if mime.starts_with("text/") {
            Some(MediaType::Text)
        } else {
            None
        }
    }

    fn download_url_kind_for_resource(
        url: &str,
        mimetype: Option<&str>,
        format: Option<&str>,
    ) -> DownloadUrlKind {
        let mime = mimetype.unwrap_or_default().trim().to_ascii_lowercase();
        let format = format.unwrap_or_default().trim().to_ascii_lowercase();
        let extension = url
            .split('?')
            .next()
            .and_then(|value| value.rsplit('.').next())
            .unwrap_or_default()
            .to_ascii_lowercase();

        if mime == "text/html" || matches!(format.as_str(), "html" | "htm") {
            return DownloadUrlKind::LandingPage;
        }

        if !mime.is_empty()
            && (Self::mime_to_media_type(&mime).is_some()
                || mime == "application/zip"
                || mime == "application/gzip")
        {
            return DownloadUrlKind::DirectFile;
        }

        match extension.as_str() {
            "json" | "geojson" | "csv" | "tsv" | "xlsx" | "xls" | "xml" | "pdf" | "doc"
            | "docx" | "txt" | "zip" | "gz" | "tar" | "kml" | "kmz" | "shp" => {
                DownloadUrlKind::DirectFile
            }
            _ => DownloadUrlKind::Unknown,
        }
    }

    fn parse_license(title: Option<&str>, license_id: Option<&str>) -> License {
        let value = title.or(license_id).unwrap_or("Unknown").trim();
        let combined = [title, license_id]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");

        parse_known_license_label(&combined).unwrap_or_else(|| License::Other(value.to_string()))
    }

    fn metadata(
        package: &DataGovPackage,
        resource: &DataGovResource,
        source_url: &str,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("datagov.package_name".to_string(), package.name.clone());
        metadata.insert("datagov.resource_id".to_string(), resource.id.clone());
        metadata.insert("datagov.source_url".to_string(), source_url.to_string());
        if let Some(format) = resource.format.as_ref().filter(|value| !value.is_empty()) {
            metadata.insert("datagov.resource_format".to_string(), format.clone());
        }
        if let Some(mimetype) = resource.mimetype.as_ref().filter(|value| !value.is_empty()) {
            metadata.insert("datagov.resource_mimetype".to_string(), mimetype.clone());
        }
        if let Some(license_id) = package
            .license_id
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            metadata.insert("datagov.license_id".to_string(), license_id.clone());
        }
        if let Some(license_title) = package
            .license_title
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            metadata.insert("datagov.license_title".to_string(), license_title.clone());
        }
        if let Some(license_url) = package
            .license_url
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            metadata.insert("datagov.license_url".to_string(), license_url.clone());
        }
        metadata
    }
}

#[async_trait]
impl Provider for DataGovProvider {
    fn name(&self) -> &'static str {
        "datagov"
    }

    fn display_name(&self) -> &'static str {
        "Data.gov"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Data, MediaType::Document, MediaType::Text]
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
        Self::BASE_URL
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let url = format!("{}/action/package_search", Self::BASE_URL);

        let rows = query.count.min(100).to_string();
        let start = ((query.page - 1) * query.count).to_string();

        let params = [
            ("q", query.query.as_str()),
            ("rows", &rows),
            ("start", &start),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;
        let api_response: DataGovResponse = response.json_or_error().await?;

        let mut assets: Vec<MediaAsset> = Vec::new();

        for package in api_response.result.results {
            // Each package can have multiple resources (files)
            for resource in &package.resources {
                // Skip resources without URLs
                let download_url = match &resource.url {
                    Some(url) if !url.is_empty() => url.clone(),
                    _ => continue,
                };

                let format = resource.format.as_deref().unwrap_or("unknown");
                let format_media_type = Self::format_to_media_type(format);
                let media_type = resource
                    .mimetype
                    .as_deref()
                    .and_then(Self::mime_to_media_type)
                    .unwrap_or(format_media_type);

                // Filter by media type if specified
                if let Some(requested_type) = query.media_type {
                    if media_type != requested_type {
                        continue;
                    }
                }

                let id = format!("datagov_{}", resource.id);
                let title = resource
                    .name
                    .clone()
                    .or_else(|| resource.description.clone())
                    .unwrap_or_else(|| package.title.clone());

                let author = package
                    .organization
                    .as_ref()
                    .map(|o| o.title.clone())
                    .unwrap_or_else(|| "US Government".to_string());

                let source_url = format!(
                    "https://catalog.data.gov/dataset/{}",
                    package.name.replace(' ', "-").to_lowercase()
                );
                let license = Self::parse_license(
                    package.license_title.as_deref(),
                    package.license_id.as_deref(),
                );
                let mut metadata = Self::metadata(&package, resource, &source_url);
                if media_type != format_media_type {
                    metadata.insert(
                        "datagov.type_resolution".to_string(),
                        "mime-preferred-over-format".to_string(),
                    );
                }
                let download_url_kind = Self::download_url_kind_for_resource(
                    &download_url,
                    resource.mimetype.as_deref(),
                    resource.format.as_deref(),
                );
                metadata.insert(
                    "datagov.download_url_kind".to_string(),
                    download_url_kind.as_str().to_string(),
                );
                // Build tags from package tags
                let mut tags: Vec<String> = package.tags.iter().map(|t| t.name.clone()).collect();

                if let Some(fmt) = &resource.format {
                    tags.push(fmt.to_lowercase());
                }
                tags.push("government".to_string());
                tags.push("open-data".to_string());

                let asset = MediaAsset::builder()
                    .id(id)
                    .provider("datagov")
                    .media_type(media_type)
                    .title(title)
                    .download_url(download_url)
                    .download_url_kind(download_url_kind)
                    .source_url(source_url)
                    .author(author)
                    .license(license)
                    .maybe_mime_type(resource.mimetype.clone())
                    .provider_metadata(metadata)
                    .tags(tags)
                    .maybe_file_size(resource.size)
                    .build_or_log();

                if let Some(asset) = asset {
                    assets.push(asset);
                }

                // Limit results per package to avoid too many from one dataset
                if assets.len() >= query.count {
                    break;
                }
            }

            if assets.len() >= query.count {
                break;
            }
        }

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.result.count,
            assets,
            providers_searched: vec!["datagov".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for DataGovProvider {
    fn description(&self) -> &'static str {
        "300,000+ US Government open datasets (JSON, CSV, XML, PDF)"
    }

    fn api_key_url(&self) -> &'static str {
        "https://data.gov" // No API key needed
    }

    fn default_license(&self) -> &'static str {
        "Varies by dataset/resource metadata"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// These structs are used by serde for JSON deserialization.
// Fields may appear unused but are read during deserialization.
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct DataGovResponse {
    #[allow(dead_code)] // Read by serde during deserialization
    success: bool,
    result: DataGovResult,
}

#[derive(Debug, Deserialize)]
struct DataGovResult {
    count: usize,
    results: Vec<DataGovPackage>,
}

#[derive(Debug, Deserialize)]
struct DataGovPackage {
    #[allow(dead_code)] // Read by serde during deserialization
    id: String,
    name: String,
    title: String,
    #[serde(default)]
    license_id: Option<String>,
    #[serde(default)]
    license_title: Option<String>,
    #[serde(default)]
    license_url: Option<String>,
    #[serde(default)]
    #[allow(dead_code)] // Read by serde during deserialization
    notes: Option<String>,
    #[serde(default)]
    organization: Option<DataGovOrganization>,
    #[serde(default)]
    resources: Vec<DataGovResource>,
    #[serde(default)]
    tags: Vec<DataGovTag>,
}

#[derive(Debug, Deserialize)]
struct DataGovOrganization {
    #[serde(default)]
    #[allow(dead_code)] // Read by serde during deserialization
    id: String,
    #[serde(default)]
    #[allow(dead_code)] // Read by serde during deserialization
    name: String,
    #[serde(default)]
    title: String,
}

#[derive(Debug, Deserialize)]
struct DataGovResource {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    #[allow(dead_code)] // Read by serde during deserialization
    size: Option<u64>,
    #[serde(default)]
    mimetype: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DataGovTag {
    #[serde(default)]
    #[allow(dead_code)] // Read by serde during deserialization
    id: String,
    name: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let config = Config::default();
        let provider = DataGovProvider::new(&config);

        assert_eq!(provider.name(), "datagov");
        assert_eq!(provider.display_name(), "Data.gov");
        assert!(!provider.requires_api_key());
        assert!(provider.is_available());
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default();
        let provider = DataGovProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Data));
        assert!(types.contains(&MediaType::Document));
        assert!(types.contains(&MediaType::Text));
    }

    #[test]
    fn test_format_to_media_type() {
        assert_eq!(
            DataGovProvider::format_to_media_type("json"),
            MediaType::Data
        );
        assert_eq!(
            DataGovProvider::format_to_media_type("CSV"),
            MediaType::Data
        );
        assert_eq!(
            DataGovProvider::format_to_media_type("xml"),
            MediaType::Data
        );
        assert_eq!(
            DataGovProvider::format_to_media_type("pdf"),
            MediaType::Document
        );
        assert_eq!(
            DataGovProvider::format_to_media_type("txt"),
            MediaType::Text
        );
        assert_eq!(
            DataGovProvider::format_to_media_type("geojson"),
            MediaType::Data
        );
    }

    #[test]
    fn mime_to_media_type_distinguishes_text_from_structured_data() {
        assert_eq!(
            DataGovProvider::mime_to_media_type("text/plain"),
            Some(MediaType::Text)
        );
        assert_eq!(
            DataGovProvider::mime_to_media_type("text/html"),
            Some(MediaType::Text)
        );
        assert_eq!(
            DataGovProvider::mime_to_media_type("text/csv"),
            Some(MediaType::Data)
        );
        assert_eq!(
            DataGovProvider::mime_to_media_type("application/json"),
            Some(MediaType::Data)
        );
    }

    #[test]
    fn data_gov_resource_url_kind_distinguishes_files_from_landing_pages() {
        assert_eq!(
            DataGovProvider::download_url_kind_for_resource(
                "https://example.gov/data.csv",
                None,
                Some("CSV")
            ),
            crate::types::DownloadUrlKind::DirectFile
        );
        assert_eq!(
            DataGovProvider::download_url_kind_for_resource(
                "https://example.gov/metadata",
                Some("text/html"),
                Some("HTML")
            ),
            crate::types::DownloadUrlKind::LandingPage
        );
        assert_eq!(
            DataGovProvider::download_url_kind_for_resource(
                "https://api.example.gov/query",
                None,
                Some("API")
            ),
            crate::types::DownloadUrlKind::Unknown
        );
    }

    #[test]
    fn test_license_parsing_does_not_default_to_public_domain() {
        assert!(matches!(
            DataGovProvider::parse_license(None, None),
            License::Other(value) if value == "Unknown"
        ));
        assert!(matches!(
            DataGovProvider::parse_license(Some("Creative Commons Attribution"), Some("cc-by")),
            License::CcBy
        ));
        assert!(matches!(
            DataGovProvider::parse_license(None, Some("cc-by-nd")),
            License::Other(value) if value == "cc-by-nd"
        ));
        assert!(matches!(
            DataGovProvider::parse_license(None, Some("cc-by-nc-sa")),
            License::Other(value) if value == "cc-by-nc-sa"
        ));
    }
}
