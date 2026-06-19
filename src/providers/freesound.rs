//! Freesound provider implementation.
//!
//! [Freesound API Documentation](https://freesound.org/docs/api/)
//!
//! Provides access to 600,000+ sound effects and audio samples.

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::{Provider, ProviderInfo};
use crate::types::{
    DownloadUrlKind, License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult,
};

/// Freesound provider for sound effects and audio samples.
/// Access to 600K+ Creative Commons licensed sounds.
#[derive(Debug)]
pub struct FreesoundProvider {
    api_key: Option<String>,
    client: HttpClient,
}

impl FreesoundProvider {
    /// Create a new Freesound provider.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let client = HttpClient::with_config(
            Self::RATE_LIMIT,
            config.retry_attempts,
            Duration::from_secs(config.timeout_secs),
        )
        .unwrap_or_default();

        Self {
            api_key: config
                .freesound_api_key
                .as_deref()
                .map(str::trim)
                .filter(|key| !key.is_empty())
                .map(ToOwned::to_owned),
            client,
        }
    }

    /// Rate limit: 2000 requests per day
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(2000, 86400);

    /// Parse license from Freesound license string
    fn parse_license(license: &str) -> License {
        match license {
            "Creative Commons 0" => License::Cc0,
            "Attribution" => License::CcBy,
            "Attribution Noncommercial" => License::CcByNc,
            _ => License::Other(license.to_string()),
        }
    }

    fn best_preview_url(sound: &FreesoundSound) -> Option<String> {
        sound
            .previews
            .as_ref()
            .and_then(|previews| {
                previews
                    .preview_hq_mp3
                    .clone()
                    .or_else(|| previews.preview_lq_mp3.clone())
                    .or_else(|| previews.preview_hq_ogg.clone())
                    .or_else(|| previews.preview_lq_ogg.clone())
            })
            .filter(|url| !url.trim().is_empty())
    }

    fn asset_from_sound(sound: FreesoundSound) -> Option<MediaAsset> {
        let license = Self::parse_license(&sound.license);
        let preview_url = Self::best_preview_url(&sound);
        let authenticated_download_url = sound
            .download
            .as_ref()
            .map(|url| url.trim())
            .filter(|url| !url.is_empty())
            .map(ToOwned::to_owned);

        let (download_url, download_url_kind, download_url_source, selected_requires_credentials) =
            if let Some(preview_url) = preview_url.clone() {
                (preview_url, DownloadUrlKind::DirectFile, "preview", false)
            } else if let Some(download_url) = authenticated_download_url.clone() {
                (
                    download_url,
                    DownloadUrlKind::Unknown,
                    "authenticated-download",
                    true,
                )
            } else {
                return None;
            };

        let author_url = format!("https://freesound.org/people/{}/", sound.username);
        let source_url = format!("https://freesound.org/s/{}/", sound.id);
        let mut metadata = HashMap::from([
            (
                "freesound.download_url_source".to_string(),
                download_url_source.to_string(),
            ),
            (
                "freesound.selected_download_requires_credentials".to_string(),
                selected_requires_credentials.to_string(),
            ),
            ("freesound.license_label".to_string(), sound.license.clone()),
            (
                "freesound.license_evidence".to_string(),
                "api-license-field".to_string(),
            ),
            ("freesound.source_url".to_string(), source_url.clone()),
        ]);

        if let Some(download_url) = authenticated_download_url {
            metadata.insert(
                "freesound.authenticated_download_url".to_string(),
                download_url,
            );
            metadata.insert(
                "freesound.authenticated_download_requires_credentials".to_string(),
                "true".to_string(),
            );
        }
        if let Some(duration) = sound.duration {
            metadata.insert(
                "freesound.duration_seconds".to_string(),
                duration.to_string(),
            );
        }
        if let Some(filesize) = sound.filesize {
            metadata.insert(
                "freesound.original_file_size_bytes".to_string(),
                filesize.to_string(),
            );
        }

        MediaAsset::builder()
            .id(sound.id.to_string())
            .provider("freesound")
            .media_type(MediaType::Audio)
            .title(sound.name)
            .download_url(download_url)
            .download_url_kind(download_url_kind)
            .preview_url(preview_url.unwrap_or_default())
            .source_url(source_url)
            .author(sound.username)
            .author_url(author_url)
            .license(license)
            .tags(sound.tags)
            .maybe_file_size(sound.filesize)
            .provider_metadata(metadata)
            .build_or_log()
    }
}

#[async_trait]
impl Provider for FreesoundProvider {
    fn name(&self) -> &'static str {
        "freesound"
    }

    fn display_name(&self) -> &'static str {
        "Freesound"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Audio]
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
        "https://freesound.org/apiv2"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let Some(ref api_key) = self.api_key else {
            return Err(crate::error::DxError::MissingApiKey {
                provider: "freesound".to_string(),
                env_var: "FREESOUND_API_KEY".to_string(),
            });
        };

        let url = format!("{}/search/text/", self.base_url());

        let page_str = query.page.to_string();
        let page_size_str = query.count.min(150).to_string();

        let params = [
            ("query", query.query.as_str()),
            ("page", &page_str),
            ("page_size", &page_size_str),
            (
                "fields",
                "id,name,description,tags,license,username,previews,download,duration,filesize",
            ),
            ("token", api_key.as_str()),
        ];

        let response = self.client.get_with_query(&url, &params, &[]).await?;

        let api_response: FreesoundSearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = api_response
            .results
            .into_iter()
            .filter_map(Self::asset_from_sound)
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: api_response.count,
            assets,
            providers_searched: vec!["freesound".to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

impl ProviderInfo for FreesoundProvider {
    fn description(&self) -> &'static str {
        "Collaborative database of Creative Commons licensed sounds"
    }

    fn api_key_url(&self) -> &'static str {
        "https://freesound.org/apiv2/apply/"
    }

    fn default_license(&self) -> &'static str {
        "Creative Commons (CC0, CC-BY, CC-BY-NC)"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FreesoundSearchResponse {
    count: usize,
    next: Option<String>,
    previous: Option<String>,
    results: Vec<FreesoundSound>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FreesoundSound {
    id: u64,
    name: String,
    description: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    license: String,
    username: String,
    previews: Option<FreesoundPreviews>,
    download: Option<String>,
    duration: Option<f64>,
    filesize: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FreesoundPreviews {
    #[serde(rename = "preview-hq-mp3")]
    preview_hq_mp3: Option<String>,
    #[serde(rename = "preview-lq-mp3")]
    preview_lq_mp3: Option<String>,
    #[serde(rename = "preview-hq-ogg")]
    preview_hq_ogg: Option<String>,
    #[serde(rename = "preview-lq-ogg")]
    preview_lq_ogg: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let config = Config::default_for_testing();
        let provider = FreesoundProvider::new(&config);

        assert_eq!(provider.name(), "freesound");
        assert_eq!(provider.display_name(), "Freesound");
        assert!(provider.requires_api_key());
    }

    #[test]
    fn blank_api_key_does_not_mark_provider_available() {
        let mut config = Config::default_for_testing();
        config.freesound_api_key = Some("   ".to_string());
        let provider = FreesoundProvider::new(&config);

        assert!(
            !provider.is_available(),
            "blank Freesound API key must not make credential-backed provider available"
        );
    }

    #[test]
    fn test_license_parsing() {
        assert!(matches!(
            FreesoundProvider::parse_license("Creative Commons 0"),
            License::Cc0
        ));
        assert!(matches!(
            FreesoundProvider::parse_license("Attribution"),
            License::CcBy
        ));
    }

    #[test]
    fn test_supported_media_types() {
        let config = Config::default_for_testing();
        let provider = FreesoundProvider::new(&config);

        let types = provider.supported_media_types();
        assert!(types.contains(&MediaType::Audio));
        assert!(!types.contains(&MediaType::Image));
    }

    fn test_sound(download: Option<&str>, previews: Option<FreesoundPreviews>) -> FreesoundSound {
        FreesoundSound {
            id: 123,
            name: "Test sound".to_string(),
            description: None,
            tags: vec!["field".to_string()],
            license: "Attribution".to_string(),
            username: "recordist".to_string(),
            previews,
            download: download.map(str::to_string),
            duration: Some(1.5),
            filesize: Some(42),
        }
    }

    #[test]
    fn asset_from_sound_prefers_direct_preview_over_authenticated_download() {
        let sound = test_sound(
            Some("https://freesound.org/apiv2/sounds/123/download/"),
            Some(FreesoundPreviews {
                preview_hq_mp3: Some("https://cdn.freesound.org/preview.mp3".to_string()),
                preview_lq_mp3: None,
                preview_hq_ogg: None,
                preview_lq_ogg: None,
            }),
        );

        let asset = FreesoundProvider::asset_from_sound(sound).unwrap();

        assert_eq!(asset.download_url, "https://cdn.freesound.org/preview.mp3");
        assert_eq!(asset.download_url_kind, DownloadUrlKind::DirectFile);
        assert_eq!(
            asset
                .provider_metadata
                .get("freesound.download_url_source")
                .map(String::as_str),
            Some("preview")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("freesound.authenticated_download_requires_credentials")
                .map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn asset_from_sound_marks_download_only_url_as_credential_bound() {
        let sound = test_sound(
            Some("https://freesound.org/apiv2/sounds/123/download/"),
            None,
        );

        let asset = FreesoundProvider::asset_from_sound(sound).unwrap();

        assert_eq!(
            asset.download_url,
            "https://freesound.org/apiv2/sounds/123/download/"
        );
        assert_eq!(asset.download_url_kind, DownloadUrlKind::Unknown);
        assert_eq!(
            asset
                .provider_metadata
                .get("freesound.selected_download_requires_credentials")
                .map(String::as_str),
            Some("true")
        );
    }
}
