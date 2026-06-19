//! GitHub Code Search provider implementation.
//!
//! [GitHub Search API](https://docs.github.com/en/rest/search)
//!
//! Provides access to data files (JSON, CSV, PDF, Excel) hosted on GitHub repositories.
//! Unauthenticated: 10 requests/minute. Authenticated: 30 requests/minute.

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::{DxError, Result};
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// GitHub Code Search provider for data files.
/// Search JSON, CSV, PDF, Excel files in public repositories.
#[derive(Debug)]
pub struct GitHubProvider {
    client: HttpClient,
}

impl GitHubProvider {
    /// Create a new GitHub provider.
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

    /// Rate limit: 10 requests/minute unauthenticated
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(10, 60);

    /// Base URL for GitHub API
    const BASE_URL: &'static str = "https://api.github.com";

    /// Get the file extension filter for the media type
    fn get_extension_filter(media_type: Option<MediaType>) -> Vec<&'static str> {
        match media_type {
            Some(MediaType::Data) => vec!["json", "csv", "xml", "yaml", "yml", "toml"],
            Some(MediaType::Document) => vec!["pdf", "xlsx", "xls", "doc", "docx", "md", "txt"],
            Some(MediaType::Code) => vec!["rs", "py", "js", "ts", "go", "java", "c", "cpp", "rb"],
            _ => vec!["json", "csv", "pdf", "xlsx", "md"], // Default: data files
        }
    }

    /// Parse media type from file extension
    fn parse_media_type(filename: &str) -> MediaType {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

        match ext.as_str() {
            "json" | "csv" | "xml" | "yaml" | "yml" | "toml" => MediaType::Data,
            "pdf" | "xlsx" | "xls" | "doc" | "docx" => MediaType::Document,
            "md" | "txt" | "rst" => MediaType::Text,
            "svg" => MediaType::Vector,
            "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "rb" | "sh" => {
                MediaType::Code
            }
            _ => MediaType::Data,
        }
    }

    /// Construct raw download URL from html_url
    fn to_raw_url(html_url: &str) -> String {
        // Convert: https://github.com/owner/repo/blob/branch/path
        // To:      https://raw.githubusercontent.com/owner/repo/branch/path
        html_url
            .replace("github.com", "raw.githubusercontent.com")
            .replace("/blob/", "/")
    }

    fn github_token() -> Option<String> {
        std::env::var("GITHUB_TOKEN")
            .ok()
            .map(|token| token.trim().to_string())
            .filter(|token| !token.is_empty())
    }

    fn request_headers(token: &str) -> Vec<(&'static str, String)> {
        vec![
            ("Accept", "application/vnd.github+json".to_string()),
            ("X-GitHub-Api-Version", "2022-11-28".to_string()),
            ("User-Agent", "dx-media/0.1.0".to_string()),
            ("Authorization", format!("Bearer {token}")),
        ]
    }

    fn provider_metadata(item: &GitHubCodeItem, download_url: &str) -> HashMap<String, String> {
        HashMap::from([
            (
                "github.repository_full_name".to_string(),
                item.repository.full_name.clone(),
            ),
            (
                "github.repository_id".to_string(),
                item.repository.id.to_string(),
            ),
            (
                "github.repository_name".to_string(),
                item.repository.name.clone(),
            ),
            (
                "github.repository_url".to_string(),
                item.repository.html_url.clone(),
            ),
            (
                "github.owner".to_string(),
                item.repository.owner.login.clone(),
            ),
            (
                "github.owner_id".to_string(),
                item.repository.owner.id.to_string(),
            ),
            ("github.path".to_string(), item.path.clone()),
            ("github.blob_sha".to_string(), item.sha.clone()),
            ("github.api_url".to_string(), item.url.clone()),
            ("github.git_url".to_string(), item.git_url.clone()),
            ("github.html_url".to_string(), item.html_url.clone()),
            ("github.raw_url".to_string(), download_url.to_string()),
            (
                "github.raw_url_derivation".to_string(),
                "derived-from-html-blob-url".to_string(),
            ),
            (
                "github.license_evidence".to_string(),
                "repository-license-not-fetched".to_string(),
            ),
        ])
    }
}

#[async_trait]
impl Provider for GitHubProvider {
    fn name(&self) -> &'static str {
        "github"
    }

    fn display_name(&self) -> &'static str {
        "GitHub"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[
            MediaType::Data,
            MediaType::Document,
            MediaType::Code,
            MediaType::Text,
        ]
    }

    fn requires_api_key(&self) -> bool {
        // GitHub Code Search API requires authentication
        true
    }

    fn rate_limit(&self) -> RateLimitConfig {
        Self::RATE_LIMIT
    }

    fn is_available(&self) -> bool {
        // Check if GITHUB_TOKEN environment variable is configured
        Self::github_token().is_some()
    }

    fn base_url(&self) -> &'static str {
        Self::BASE_URL
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let Some(token) = Self::github_token() else {
            return Err(DxError::MissingApiKey {
                provider: "github".to_string(),
                env_var: "GITHUB_TOKEN".to_string(),
            });
        };

        let url = format!("{}/search/code", Self::BASE_URL);

        // Build search query with file extension filters
        let extensions = Self::get_extension_filter(query.media_type);
        let ext_query = extensions
            .iter()
            .map(|ext| format!("extension:{}", ext))
            .collect::<Vec<_>>()
            .join(" ");

        // GitHub search requires at least one search term AND qualifier
        // Format: "query extension:json extension:csv..."
        let search_query = format!("{} {}", query.query, ext_query);

        let page_str = query.page.to_string();
        let per_page_str = query.count.min(100).to_string();

        let params = [
            ("q", search_query.as_str()),
            ("per_page", &per_page_str),
            ("page", &page_str),
        ];

        let header_values = Self::request_headers(&token);
        let headers: Vec<(&str, &str)> = header_values
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .collect();

        let response = self.client.get_with_query(&url, &params, &headers).await?;

        let api_response: GitHubSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .items
            .into_iter()
            .map(|item| {
                let media_type = Self::parse_media_type(&item.name);
                let download_url = Self::to_raw_url(&item.html_url);
                let provider_metadata = Self::provider_metadata(&item, &download_url);

                // Create a unique ID from repo + path
                let id = format!(
                    "{}-{}",
                    item.repository.full_name.replace('/', "-"),
                    item.sha.chars().take(8).collect::<String>()
                );

                let title = format!("{} ({})", item.name, item.repository.full_name);
                let author = item.repository.owner.login.clone();

                MediaAsset::builder()
                    .id(id)
                    .provider("github")
                    .media_type(media_type)
                    .title(title)
                    .direct_download_url(download_url)
                    .preview_url(item.html_url.clone())
                    .source_url(item.html_url)
                    .author(author)
                    .license(License::Other("Various (check repository)".to_string()))
                    .provider_metadata(provider_metadata)
                    .build_or_log()
            })
            .flatten()
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.total_count.min(1000) as usize, // GitHub caps at 1000
            assets,
            providers_searched: vec!["github".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for GitHubProvider {
    fn description(&self) -> &'static str {
        "Data files (JSON, CSV, PDF, Excel) from public GitHub repositories"
    }

    fn api_key_url(&self) -> &'static str {
        "https://github.com/settings/tokens"
    }

    fn default_license(&self) -> &'static str {
        "Various (check repository license)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

// Fields are read during serde deserialization from API response
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GitHubSearchResponse {
    total_count: u64,
    incomplete_results: bool,
    items: Vec<GitHubCodeItem>,
}

// Fields are read during serde deserialization from API response
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GitHubCodeItem {
    name: String,
    path: String,
    sha: String,
    url: String,
    html_url: String,
    git_url: String,
    repository: GitHubRepository,
    #[serde(default)]
    score: f64,
}

// Fields are read during serde deserialization
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GitHubRepository {
    id: u64,
    name: String,
    full_name: String,
    owner: GitHubOwner,
    html_url: String,
    description: Option<String>,
    #[serde(default)]
    stargazers_count: u32,
}

// Fields are read during serde deserialization
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GitHubOwner {
    login: String,
    id: u64,
    avatar_url: String,
    html_url: String,
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
        let provider = GitHubProvider::new(&config);

        assert_eq!(provider.name(), "github");
        assert_eq!(provider.display_name(), "GitHub");
        // GitHub Code Search requires authentication
        assert!(provider.requires_api_key());
        // is_available() depends on GITHUB_TOKEN env var
        // We don't assert a specific value since it depends on environment
    }

    #[test]
    fn test_is_available_checks_github_token() {
        let config = Config::default();
        let provider = GitHubProvider::new(&config);

        // Save current value if any
        let original = std::env::var("GITHUB_TOKEN").ok();

        // SAFETY: These env var operations are safe in single-threaded test context.
        // We restore the original value at the end.
        unsafe {
            // Test without token
            std::env::remove_var("GITHUB_TOKEN");
            assert!(
                !provider.is_available(),
                "Should be unavailable without GITHUB_TOKEN"
            );

            // Test with token
            std::env::set_var("GITHUB_TOKEN", "test_token");
            assert!(
                provider.is_available(),
                "Should be available with GITHUB_TOKEN"
            );

            std::env::set_var("GITHUB_TOKEN", "   ");
            assert!(
                !provider.is_available(),
                "Blank GITHUB_TOKEN must not make credential-backed provider available"
            );

            // Restore original value
            match original {
                Some(val) => std::env::set_var("GITHUB_TOKEN", val),
                None => std::env::remove_var("GITHUB_TOKEN"),
            }
        }
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default();
        let provider = GitHubProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Data));
        assert!(types.contains(&MediaType::Document));
        assert!(types.contains(&MediaType::Code));
        assert!(types.contains(&MediaType::Text));
    }

    #[test]
    fn test_parse_media_type() {
        assert_eq!(
            GitHubProvider::parse_media_type("data.json"),
            MediaType::Data
        );
        assert_eq!(
            GitHubProvider::parse_media_type("data.csv"),
            MediaType::Data
        );
        assert_eq!(
            GitHubProvider::parse_media_type("report.pdf"),
            MediaType::Document
        );
        assert_eq!(
            GitHubProvider::parse_media_type("sheet.xlsx"),
            MediaType::Document
        );
        assert_eq!(
            GitHubProvider::parse_media_type("README.md"),
            MediaType::Text
        );
        assert_eq!(GitHubProvider::parse_media_type("main.rs"), MediaType::Code);
        assert_eq!(
            GitHubProvider::parse_media_type("script.py"),
            MediaType::Code
        );
    }

    #[test]
    fn test_to_raw_url() {
        let html_url = "https://github.com/owner/repo/blob/main/data.json";
        let raw_url = GitHubProvider::to_raw_url(html_url);
        assert_eq!(
            raw_url,
            "https://raw.githubusercontent.com/owner/repo/main/data.json"
        );
    }

    #[test]
    fn request_headers_include_bearer_token() {
        let headers = GitHubProvider::request_headers("test_token");

        assert!(
            headers
                .iter()
                .any(|(key, value)| { *key == "Authorization" && value == "Bearer test_token" })
        );
    }

    #[test]
    fn provider_metadata_preserves_repository_path_and_sha_evidence() {
        let item = GitHubCodeItem {
            name: "data.json".to_string(),
            path: "fixtures/data.json".to_string(),
            sha: "abc123def456".to_string(),
            url: "https://api.github.com/search/result".to_string(),
            html_url: "https://github.com/owner/repo/blob/main/fixtures/data.json".to_string(),
            git_url: "https://api.github.com/repos/owner/repo/git/blobs/abc123".to_string(),
            repository: GitHubRepository {
                id: 42,
                name: "repo".to_string(),
                full_name: "owner/repo".to_string(),
                owner: GitHubOwner {
                    login: "owner".to_string(),
                    id: 7,
                    avatar_url: "https://avatars.githubusercontent.com/u/7".to_string(),
                    html_url: "https://github.com/owner".to_string(),
                },
                html_url: "https://github.com/owner/repo".to_string(),
                description: Some("test repo".to_string()),
                stargazers_count: 12,
            },
            score: 1.0,
        };
        let raw_url = GitHubProvider::to_raw_url(&item.html_url);
        let metadata = GitHubProvider::provider_metadata(&item, &raw_url);

        assert_eq!(
            metadata
                .get("github.repository_full_name")
                .map(String::as_str),
            Some("owner/repo")
        );
        assert_eq!(
            metadata.get("github.path").map(String::as_str),
            Some("fixtures/data.json")
        );
        assert_eq!(
            metadata.get("github.blob_sha").map(String::as_str),
            Some("abc123def456")
        );
        assert_eq!(
            metadata.get("github.license_evidence").map(String::as_str),
            Some("repository-license-not-fetched")
        );
    }

    #[test]
    fn test_extension_filter() {
        let data_exts = GitHubProvider::get_extension_filter(Some(MediaType::Data));
        assert!(data_exts.contains(&"json"));
        assert!(data_exts.contains(&"csv"));

        let doc_exts = GitHubProvider::get_extension_filter(Some(MediaType::Document));
        assert!(doc_exts.contains(&"pdf"));
        assert!(doc_exts.contains(&"xlsx"));
    }
}
