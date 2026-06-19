//! Download functionality for fetching media assets.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::error::{DxError, Result};
use crate::http::{HttpClient, validate_url, verify_content_type};
use crate::tools::{ToolOutput, ToolReceipt};
use crate::types::{DownloadUrlKind, MediaAsset, MediaType, RateLimitConfig};

/// Progress callback type for download progress updates.
pub type ProgressCallback = Arc<dyn Fn(u64, u64) + Send + Sync>;

/// Downloader for fetching media assets.
#[derive(Debug, Clone)]
pub struct Downloader {
    client: HttpClient,
    download_dir: PathBuf,
}

impl Downloader {
    /// Create a new downloader with default settings.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        // No rate limiting for downloads - providers handle this
        let rate_limit = RateLimitConfig::unlimited();
        let client = HttpClient::with_config(
            rate_limit,
            config.retry_attempts,
            Duration::from_secs(config.timeout_secs),
        )
        .unwrap_or_default();

        Self {
            client,
            download_dir: config.download_dir.clone(),
        }
    }

    /// Set the download directory.
    #[must_use]
    pub fn with_download_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.download_dir = dir.into();
        self
    }

    /// Download a media asset to the default download directory.
    pub async fn download(&self, asset: &MediaAsset) -> Result<PathBuf> {
        self.download_to(&self.download_dir, asset).await
    }

    /// Download a media asset to the default download directory with a provenance receipt.
    pub async fn download_with_receipt(&self, asset: &MediaAsset) -> Result<ToolOutput> {
        self.download_to_with_receipt(&self.download_dir, asset)
            .await
    }

    /// Download a media asset to a specific directory.
    pub async fn download_to(&self, dir: &Path, asset: &MediaAsset) -> Result<PathBuf> {
        let output = self.download_to_with_receipt(dir, asset).await?;
        output
            .output_paths
            .first()
            .cloned()
            .ok_or_else(|| DxError::Download {
                url: asset.download_url.clone(),
                message: "Download completed without an output path".to_string(),
            })
    }

    /// Download a media asset to a specific directory with a provenance receipt.
    pub async fn download_to_with_receipt(
        &self,
        dir: &Path,
        asset: &MediaAsset,
    ) -> Result<ToolOutput> {
        if asset.download_url_kind != DownloadUrlKind::DirectFile {
            return Err(DxError::Download {
                url: asset.download_url.clone(),
                message: format!(
                    "download_url_kind '{}' is not directly downloadable; provider resolver support is required before this asset can be fetched",
                    asset.download_url_kind.as_str()
                ),
            });
        }

        let filename = self.generate_filename(asset);
        let filepath = dir.join(&filename);

        // Ensure directory exists
        if let Some(parent) = filepath.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| DxError::FileIo {
                    path: parent.to_path_buf(),
                    message: format!("Failed to create directory: {}", e),
                    source: Some(e),
                })?;
        }

        // Download the file with URL validation and content-type verification
        let evidence = self
            .download_file(
                &asset.download_url,
                &filepath,
                asset.media_type,
                asset.mime_type.as_deref(),
            )
            .await?;

        let output = Self::download_receipt_for_asset(
            asset,
            evidence.path,
            evidence.type_evidence.accepted_mime_type.as_deref(),
            evidence.bytes_written,
        );
        let output =
            with_download_type_evidence_metadata(output, &evidence.type_evidence, "provider-mime");

        Ok(output)
    }

    /// Download a direct URL to an exact output path with a provenance receipt.
    pub async fn download_url_to_with_receipt(
        &self,
        url: &str,
        path: &Path,
        media_type: MediaType,
        expected_mime_type: Option<&str>,
    ) -> Result<ToolOutput> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| DxError::FileIo {
                        path: parent.to_path_buf(),
                        message: format!("Failed to create directory: {}", e),
                        source: Some(e),
                    })?;
            }
        }

        let evidence = self
            .download_file(url, path, media_type, expected_mime_type)
            .await?;

        let output = Self::direct_url_download_receipt(
            url,
            media_type,
            evidence.path,
            evidence.type_evidence.accepted_mime_type.as_deref(),
            evidence.bytes_written,
        );
        let output =
            with_download_type_evidence_metadata(output, &evidence.type_evidence, "requested-mime");

        Ok(output)
    }

    /// Download a media asset with progress callback.
    pub async fn download_with_progress(
        &self,
        asset: &MediaAsset,
        _on_progress: ProgressCallback,
    ) -> Result<PathBuf> {
        // For now, we don't have streaming progress - just download
        // Future enhancement: implement streaming download with progress
        self.download(asset).await
    }

    /// Download a file from URL to a path.
    async fn download_file(
        &self,
        url: &str,
        path: &Path,
        media_type: MediaType,
        expected_mime_type: Option<&str>,
    ) -> Result<DownloadEvidence> {
        // Validate URL before making request (SSRF prevention)
        validate_url(url)?;

        let response = self.client.get_raw_validating_redirects(url).await?;

        if !response.status().is_success() {
            return Err(DxError::Download {
                url: url.to_string(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let actual_mime_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .map(normalize_content_type);

        // Verify content-type matches expected media type
        if let Some(content_type) = response.headers().get("content-type") {
            if let Ok(ct_str) = content_type.to_str() {
                verify_content_type(ct_str, media_type)?;

                if let Some(expected) = expected_mime_type {
                    if response_mime_conflicts_with_expected_mime(ct_str, expected) {
                        return Err(DxError::content_type_mismatch(
                            expected,
                            normalize_content_type(ct_str),
                        ));
                    }
                }
            }
        }
        validate_actual_mime_matches_output_path(url, path, actual_mime_type.as_deref())?;

        let bytes = response.bytes().await.map_err(|e| DxError::Download {
            url: url.to_string(),
            message: format!("Failed to read response body: {}", e),
        })?;
        let type_evidence = validate_download_type_evidence(
            url,
            media_type,
            actual_mime_type.as_deref(),
            expected_mime_type,
            "expected-mime",
            &bytes,
        )?;
        let bytes_written = bytes.len() as u64;

        tokio::fs::write(path, &bytes)
            .await
            .map_err(|e| DxError::FileIo {
                path: path.to_path_buf(),
                message: format!("Failed to write file: {}", e),
                source: Some(e),
            })?;

        Ok(DownloadEvidence {
            path: path.to_path_buf(),
            bytes_written,
            type_evidence,
        })
    }

    /// Build a provider-backed receipt for an already completed download.
    #[must_use]
    pub fn download_receipt_for_asset(
        asset: &MediaAsset,
        path: impl AsRef<Path>,
        actual_mime_type: Option<&str>,
        bytes_written: u64,
    ) -> ToolOutput {
        let path = path.as_ref();
        let provenance = asset.provenance();
        let validation = &provenance.type_validation;
        let provider_type_validation = if validation.is_valid() {
            "pass"
        } else if validation.has_evidence() {
            "fail"
        } else {
            "missing"
        };

        let mut output = ToolOutput::success_with_path(
            format!("Downloaded {} from {}", asset.title, asset.provider),
            path,
        )
        .with_receipt(
            ToolReceipt::provider_backed("media.download", &asset.provider)
                .with_license(provenance.license)
                .with_source(provenance.source_url),
        )
        .with_output_type_validation(path, asset.media_type)
        .with_metadata("tool.download_url", asset.download_url.clone())
        .with_metadata(
            "tool.download_url_kind",
            provenance.download_url_kind.as_str(),
        )
        .with_metadata("tool.declared_media_type", asset.media_type.as_str())
        .with_metadata("tool.license_known", provenance.license_known.to_string())
        .with_metadata("tool.bytes_written", bytes_written.to_string())
        .with_metadata("tool.provider_type_validation", provider_type_validation);

        if let Some(preview_url) = &asset.preview_url {
            output = output.with_metadata("tool.preview_url", preview_url.clone());
        }
        if let Some(author) = provenance.author {
            output = output.with_metadata("tool.author", author);
        }
        if let Some(author_url) = provenance.author_url {
            output = output.with_metadata("tool.author_url", author_url);
        }
        if let Some(provider_mime_type) = provenance.mime_type {
            output = output.with_metadata("tool.provider_mime_type", provider_mime_type);
        }
        if let Some(mime_evidence_source) = provenance.mime_evidence_source {
            output = output.with_metadata(
                "tool.provider_mime_evidence_source",
                mime_evidence_source.as_str(),
            );
        }
        output = with_actual_mime_evidence(output, actual_mime_type, path);
        if let Some(extension) = &validation.extension {
            output = output.with_metadata("tool.provider_extension", extension.clone());
        }
        if let Some(extension_matches) = validation.extension_matches {
            output = output.with_metadata(
                "tool.provider_extension_matches",
                extension_matches.to_string(),
            );
        }
        if let Some(mime_matches) = validation.mime_matches {
            output = output.with_metadata("tool.provider_mime_matches", mime_matches.to_string());
        }

        for (key, value) in provenance.provider_metadata {
            output = output.with_metadata(format!("provider.{key}"), value);
        }

        output
    }

    /// Build a direct URL receipt for an already completed download.
    #[must_use]
    pub fn direct_url_download_receipt(
        url: &str,
        media_type: MediaType,
        path: impl AsRef<Path>,
        actual_mime_type: Option<&str>,
        bytes_written: u64,
    ) -> ToolOutput {
        let path = path.as_ref();
        let output = ToolOutput::success_with_path(
            format!("Downloaded direct URL to {}", path.display()),
            path,
        )
        .with_receipt(
            ToolReceipt::direct_url("media.download.direct-url")
                .with_license("unknown")
                .with_source(url),
        )
        .with_output_type_validation(path, media_type)
        .with_metadata("tool.download_url", url)
        .with_metadata("tool.download_url_kind", "direct-file")
        .with_metadata("tool.declared_media_type", media_type.as_str())
        .with_metadata("tool.license_known", "false")
        .with_metadata("tool.bytes_written", bytes_written.to_string())
        .with_metadata("source.direct_url.source_kind", "direct-url")
        .with_metadata("source.direct_url.provenance", "download-url")
        .with_metadata("source.direct_url.license", "not-provided");

        with_actual_mime_evidence(output, actual_mime_type, path)
    }

    /// Generate a filename for an asset.
    fn generate_filename(&self, asset: &MediaAsset) -> String {
        // Sanitize the ID to be a valid filename
        let sanitized_id = self.sanitize_filename(&asset.id);
        let extension = self.guess_extension(asset);
        format!("{}-{}.{}", asset.provider, sanitized_id, extension)
    }

    /// Sanitize a string to be a valid filename.
    fn sanitize_filename(&self, input: &str) -> String {
        // Replace invalid characters with underscores
        input
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                c if c.is_ascii_control() => '_',
                c => c,
            })
            .collect::<String>()
            // Limit length to avoid filesystem issues
            .chars()
            .take(100)
            .collect()
    }

    /// Guess the file extension from the asset.
    fn guess_extension(&self, asset: &MediaAsset) -> &'static str {
        // Try to extract from URL first
        if let Some(ext) = self.extension_from_url(&asset.download_url, asset.media_type) {
            return ext;
        }

        if let Some(mime_type) = &asset.mime_type {
            if let Some(ext) = extension_from_mime(mime_type, asset.media_type) {
                return ext;
            }
        }

        // Fall back to media type default
        match asset.media_type {
            crate::types::MediaType::Image => "jpg",
            crate::types::MediaType::Video => "mp4",
            crate::types::MediaType::Audio => "mp3",
            crate::types::MediaType::Gif => "gif",
            crate::types::MediaType::Vector => "svg",
            crate::types::MediaType::Document => "pdf",
            crate::types::MediaType::Data => "json",
            crate::types::MediaType::Model3D => "glb",
            crate::types::MediaType::Code => "txt",
            crate::types::MediaType::Text => "txt",
        }
    }

    /// Extract extension from URL.
    fn extension_from_url(&self, url: &str, media_type: MediaType) -> Option<&'static str> {
        let parsed = url::Url::parse(url).ok()?;
        let filename = parsed.path_segments()?.next_back()?;
        let extension = filename.rsplit_once('.')?.1.to_ascii_lowercase();
        let normalized = match extension.as_str() {
            "jpg" | "jpeg" => "jpg",
            "png" => "png",
            "gif" => "gif",
            "webp" => "webp",
            "avif" => "avif",
            "bmp" => "bmp",
            "tif" | "tiff" => "tiff",
            "svg" => "svg",
            "ico" => "ico",
            "mp4" => "mp4",
            "webm" => "webm",
            "mov" => "mov",
            "mp3" => "mp3",
            "wav" => "wav",
            "flac" => "flac",
            "ogg" => "ogg",
            "pdf" => "pdf",
            "json" => "json",
            "csv" => "csv",
            "gltf" => "gltf",
            "glb" => "glb",
            "txt" => "txt",
            _ => return None,
        };

        media_type
            .matches_extension(normalized)
            .then_some(normalized)
    }

    /// Get the default download directory.
    #[must_use]
    pub fn download_dir(&self) -> &Path {
        &self.download_dir
    }
}

#[derive(Debug, Clone)]
struct DownloadEvidence {
    path: PathBuf,
    bytes_written: u64,
    type_evidence: DownloadTypeEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DownloadTypeEvidence {
    accepted_mime_type: Option<String>,
    mime_evidence_source: Option<&'static str>,
    body_signature_validation: Option<&'static str>,
}

fn normalize_content_type(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase()
}

fn response_mime_conflicts_with_expected_mime(actual: &str, expected: &str) -> bool {
    let actual = normalize_content_type(actual);
    let expected = normalize_content_type(expected);

    actual != expected
        && actual != "application/octet-stream"
        && expected != "application/octet-stream"
}

fn extension_from_mime(mime_type: &str, media_type: MediaType) -> Option<&'static str> {
    let normalized_mime = normalize_content_type(mime_type);
    let extension = match normalized_mime.as_str() {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/avif" => "avif",
        "image/bmp" | "image/x-ms-bmp" => "bmp",
        "image/tiff" => "tiff",
        "image/svg+xml" => "svg",
        "image/x-icon" | "image/vnd.microsoft.icon" => "ico",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/wav" | "audio/wave" | "audio/x-wav" => "wav",
        "audio/flac" => "flac",
        "audio/ogg" | "application/ogg" => "ogg",
        "application/pdf" => "pdf",
        "application/json" => "json",
        "text/csv" => "csv",
        "model/gltf+json" => "gltf",
        "model/gltf-binary" | "application/octet-stream" => "glb",
        "text/plain" => "txt",
        _ => return None,
    };

    media_type.matches_extension(extension).then_some(extension)
}

fn mime_matches_extension(mime_type: &str, extension: &str) -> Option<bool> {
    let normalized_mime = normalize_content_type(mime_type);
    let normalized_extension = extension.trim_start_matches('.').to_ascii_lowercase();
    let expected_extensions: &[&str] = match normalized_mime.as_str() {
        "image/jpeg" | "image/jpg" => &["jpg", "jpeg"],
        "image/png" => &["png"],
        "image/gif" => &["gif"],
        "image/webp" => &["webp"],
        "image/avif" => &["avif"],
        "image/bmp" | "image/x-ms-bmp" => &["bmp"],
        "image/tiff" => &["tif", "tiff"],
        "image/svg+xml" => &["svg"],
        "image/x-icon" | "image/vnd.microsoft.icon" => &["ico"],
        "video/mp4" => &["mp4"],
        "video/webm" => &["webm"],
        "video/quicktime" => &["mov"],
        "audio/mpeg" | "audio/mp3" => &["mp3"],
        "audio/wav" | "audio/wave" | "audio/x-wav" => &["wav"],
        "audio/flac" => &["flac"],
        "audio/ogg" | "application/ogg" => &["ogg"],
        "application/pdf" => &["pdf"],
        "application/json" => &["json"],
        "text/csv" => &["csv"],
        "model/gltf+json" => &["gltf"],
        "model/gltf-binary" => &["glb"],
        "text/plain" => &["txt"],
        _ => return None,
    };

    Some(expected_extensions.contains(&normalized_extension.as_str()))
}

fn actual_mime_matches_path_extension(mime_type: &str, path: &Path) -> Option<bool> {
    let extension = path.extension()?.to_str()?;
    mime_matches_extension(mime_type, extension)
}

fn validate_actual_mime_matches_output_path(
    url: &str,
    path: &Path,
    actual_mime_type: Option<&str>,
) -> Result<()> {
    let Some(actual_mime_type) = actual_mime_type else {
        return Ok(());
    };

    if actual_mime_matches_path_extension(actual_mime_type, path) != Some(false) {
        return Ok(());
    }

    Err(DxError::Download {
        url: url.to_string(),
        message: format!(
            "Response Content-Type '{actual_mime_type}' does not match output path {}: actual-mime-extension-mismatch",
            path.display()
        ),
    })
}

fn with_actual_mime_evidence(
    mut output: ToolOutput,
    actual_mime_type: Option<&str>,
    path: &Path,
) -> ToolOutput {
    if let Some(actual_mime_type) = actual_mime_type {
        if let Some(extension_matches) = actual_mime_matches_path_extension(actual_mime_type, path)
        {
            output = output.with_metadata(
                "tool.actual_file_validation",
                if extension_matches { "pass" } else { "fail" },
            );
            if !extension_matches {
                output = output
                    .with_metadata("tool.type_validation", "fail")
                    .with_metadata(
                        "tool.type_validation_reason",
                        "actual-mime-extension-mismatch",
                    );
            }
            output = output.with_metadata(
                "tool.actual_mime_extension_matches",
                extension_matches.to_string(),
            );
        } else {
            output = output.with_metadata("tool.actual_file_validation", "unknown");
        }
        output = output.with_metadata("tool.actual_mime_type", actual_mime_type);
    }

    output
}

fn with_download_type_evidence_metadata(
    mut output: ToolOutput,
    type_evidence: &DownloadTypeEvidence,
    expected_mime_source_label: &'static str,
) -> ToolOutput {
    if let Some(validation) = type_evidence.body_signature_validation {
        output = output.with_metadata("tool.actual_body_signature_validation", validation);
    }

    if let Some(mime_evidence_source) = type_evidence.mime_evidence_source {
        let source = if mime_evidence_source == "expected-mime" {
            expected_mime_source_label
        } else {
            mime_evidence_source
        };
        output = output.with_metadata("tool.actual_mime_evidence_source", source);
    }

    output
}

fn validate_download_type_evidence(
    url: &str,
    media_type: MediaType,
    actual_mime_type: Option<&str>,
    expected_mime_type: Option<&str>,
    expected_mime_source_label: &'static str,
    bytes: &[u8],
) -> Result<DownloadTypeEvidence> {
    if let Some(evidence) = validate_expected_mime_body_signature(
        url,
        media_type,
        actual_mime_type,
        expected_mime_type,
        expected_mime_source_label,
        bytes,
    )? {
        return Ok(evidence);
    }

    if let Some(evidence) = validate_body_signature_evidence(url, actual_mime_type, bytes)? {
        validate_download_evidence_matches_media_type(media_type, &evidence)?;
        return Ok(evidence);
    }

    if has_specific_mime_evidence(actual_mime_type, media_type) {
        return Ok(DownloadTypeEvidence {
            accepted_mime_type: actual_mime_type.map(normalize_content_type),
            mime_evidence_source: Some("response-content-type"),
            body_signature_validation: None,
        });
    }

    if has_specific_mime_evidence(expected_mime_type, media_type) {
        return Ok(DownloadTypeEvidence {
            accepted_mime_type: expected_mime_type.map(normalize_content_type),
            mime_evidence_source: Some(expected_mime_source_label),
            body_signature_validation: None,
        });
    }

    if let Some(mime_type) = url_media_extension_mime(url, media_type) {
        return Ok(DownloadTypeEvidence {
            accepted_mime_type: Some(mime_type.to_string()),
            mime_evidence_source: Some("url-extension"),
            body_signature_validation: None,
        });
    }

    if !media_type_requires_type_evidence(media_type) {
        return Ok(DownloadTypeEvidence {
            accepted_mime_type: None,
            mime_evidence_source: None,
            body_signature_validation: None,
        });
    }

    Err(DxError::Download {
        url: url.to_string(),
        message: "Downloaded binary media lacks trusted type evidence: missing-type-evidence; require response Content-Type, provider MIME, or URL extension evidence before accepting output extension validation".to_string(),
    })
}

fn validate_download_evidence_matches_media_type(
    media_type: MediaType,
    evidence: &DownloadTypeEvidence,
) -> Result<()> {
    let Some(actual_mime_type) = evidence.accepted_mime_type.as_deref() else {
        return Ok(());
    };

    if media_type.matches_mime(actual_mime_type) {
        return Ok(());
    }

    Err(DxError::content_type_mismatch(
        media_type.as_str(),
        actual_mime_type,
    ))
}

fn validate_expected_mime_body_signature(
    url: &str,
    media_type: MediaType,
    actual_mime_type: Option<&str>,
    expected_mime_type: Option<&str>,
    expected_mime_source_label: &'static str,
    bytes: &[u8],
) -> Result<Option<DownloadTypeEvidence>> {
    if has_specific_mime_evidence(actual_mime_type, media_type) {
        return Ok(None);
    }

    let Some(expected_mime_type) = expected_mime_type else {
        return Ok(None);
    };

    if !has_specific_mime_evidence(Some(expected_mime_type), media_type) {
        return Ok(None);
    }

    let Some((signature_name, matches)) = body_signature_match(expected_mime_type, bytes) else {
        return Ok(None);
    };

    if matches {
        Ok(Some(DownloadTypeEvidence {
            accepted_mime_type: Some(normalize_content_type(expected_mime_type)),
            mime_evidence_source: Some(expected_mime_source_label),
            body_signature_validation: Some("pass"),
        }))
    } else {
        Err(DxError::Download {
            url: url.to_string(),
            message: format!(
                "Response body does not match provider MIME '{expected_mime_type}': expected {signature_name} signature"
            ),
        })
    }
}

fn has_specific_mime_evidence(mime_type: Option<&str>, media_type: MediaType) -> bool {
    let Some(mime_type) = mime_type else {
        return false;
    };

    let normalized_mime = normalize_content_type(mime_type);
    normalized_mime != "application/octet-stream" && media_type.matches_mime(&normalized_mime)
}

fn url_media_extension_mime(url: &str, media_type: MediaType) -> Option<&'static str> {
    let mime_type = mime_from_url_extension(url)?;
    media_type.matches_mime(mime_type).then_some(mime_type)
}

fn mime_from_url_extension(url: &str) -> Option<&'static str> {
    let extension = url_media_extension(url)?;
    let mime_type = match extension.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        "pdf" => "application/pdf",
        "glb" => "model/gltf-binary",
        "gltf" => "model/gltf+json",
        _ => return None,
    };

    Some(mime_type)
}

fn url_media_extension(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    let filename = parsed.path_segments()?.next_back()?;
    Some(filename.rsplit_once('.')?.1.to_ascii_lowercase())
}

fn media_type_requires_type_evidence(media_type: MediaType) -> bool {
    matches!(
        media_type,
        MediaType::Image
            | MediaType::Video
            | MediaType::Audio
            | MediaType::Gif
            | MediaType::Vector
            | MediaType::Document
            | MediaType::Model3D
    )
}

#[cfg(test)]
fn validate_body_signature(
    url: &str,
    actual_mime_type: Option<&str>,
    bytes: &[u8],
) -> Result<Option<&'static str>> {
    Ok(
        validate_body_signature_evidence(url, actual_mime_type, bytes)?
            .and_then(|evidence| evidence.body_signature_validation),
    )
}

fn validate_body_signature_evidence(
    url: &str,
    actual_mime_type: Option<&str>,
    bytes: &[u8],
) -> Result<Option<DownloadTypeEvidence>> {
    if let Some(actual_mime_type) = actual_mime_type {
        let normalized_mime = normalize_content_type(actual_mime_type);
        if normalized_mime != "application/octet-stream" {
            if let Some((signature_name, matches)) = body_signature_match(actual_mime_type, bytes) {
                return if matches {
                    Ok(Some(DownloadTypeEvidence {
                        accepted_mime_type: Some(normalized_mime),
                        mime_evidence_source: Some("response-content-type"),
                        body_signature_validation: Some("pass"),
                    }))
                } else {
                    Err(DxError::Download {
                        url: url.to_string(),
                        message: format!(
                            "Response body does not match Content-Type '{actual_mime_type}': expected {signature_name} signature"
                        ),
                    })
                };
            }
        }
    }

    let actual_mime_is_missing_or_generic = actual_mime_type.is_none()
        || actual_mime_type
            .map(normalize_content_type)
            .is_some_and(|mime| mime == "application/octet-stream");
    if !actual_mime_is_missing_or_generic {
        return Ok(None);
    };

    let Some((mime_type, signature_name, matches)) = body_signature_match_from_url(url, bytes)
    else {
        return Ok(None);
    };

    if matches {
        Ok(Some(DownloadTypeEvidence {
            accepted_mime_type: Some(mime_type.to_string()),
            mime_evidence_source: Some("url-extension"),
            body_signature_validation: Some("pass"),
        }))
    } else {
        Err(DxError::Download {
            url: url.to_string(),
            message: format!(
                "Response body does not match URL extension evidence: expected {signature_name} signature"
            ),
        })
    }
}

fn body_signature_match_from_url(
    url: &str,
    bytes: &[u8],
) -> Option<(&'static str, &'static str, bool)> {
    let mime_type = mime_from_url_extension(url)?;
    body_signature_match(mime_type, bytes)
        .map(|(signature_name, matches)| (mime_type, signature_name, matches))
}

fn body_signature_match(mime_type: &str, bytes: &[u8]) -> Option<(&'static str, bool)> {
    let normalized_mime = normalize_content_type(mime_type);
    let result = match normalized_mime.as_str() {
        "image/png" => ("PNG", bytes.starts_with(b"\x89PNG\r\n\x1a\n")),
        "image/jpeg" | "image/jpg" => ("JPEG", bytes.starts_with(&[0xff, 0xd8, 0xff])),
        "image/gif" => (
            "GIF",
            bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        ),
        "image/webp" => (
            "WEBP",
            bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP",
        ),
        "image/avif" => ("AVIF", iso_bmff_has_brand(bytes, &[b"avif", b"avis"])),
        "image/bmp" | "image/x-ms-bmp" => ("BMP", bytes.starts_with(b"BM")),
        "image/tiff" => (
            "TIFF",
            bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*"),
        ),
        "image/svg+xml" => ("SVG", bytes_are_svg(bytes)),
        "image/x-icon" | "image/vnd.microsoft.icon" => (
            "ICO",
            bytes.starts_with(&[0, 0, 1, 0]) || bytes.starts_with(&[0, 0, 2, 0]),
        ),
        "video/mp4" => (
            "MP4",
            iso_bmff_has_brand(
                bytes,
                &[
                    b"isom", b"iso2", b"avc1", b"mp41", b"mp42", b"dash", b"M4V ",
                ],
            ),
        ),
        "video/webm" => ("WEBM", bytes.starts_with(&[0x1a, 0x45, 0xdf, 0xa3])),
        "video/quicktime" => ("QuickTime", iso_bmff_has_brand(bytes, &[b"qt  "])),
        "audio/mpeg" | "audio/mp3" => ("MP3", bytes_are_mp3(bytes)),
        "audio/wav" | "audio/wave" | "audio/x-wav" => (
            "WAV",
            bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WAVE",
        ),
        "audio/flac" => ("FLAC", bytes.starts_with(b"fLaC")),
        "audio/ogg" | "application/ogg" => ("OGG", bytes.starts_with(b"OggS")),
        "audio/aac" => ("AAC", bytes_are_adts_aac(bytes)),
        "audio/mp4" | "audio/x-m4a" => (
            "M4A",
            iso_bmff_has_brand(bytes, &[b"M4A ", b"M4B ", b"isom", b"mp42"]),
        ),
        "application/pdf" => ("PDF", bytes.starts_with(b"%PDF-")),
        "model/gltf-binary" => ("GLB", bytes.starts_with(b"glTF")),
        "model/gltf+json" => ("glTF JSON", bytes_are_gltf_json(bytes)),
        _ => return None,
    };

    Some(result)
}

fn iso_bmff_has_brand(bytes: &[u8], expected_brands: &[&[u8; 4]]) -> bool {
    if bytes.len() < 12 || &bytes[4..8] != b"ftyp" {
        return false;
    }

    let major_brand = &bytes[8..12];
    if expected_brands
        .iter()
        .any(|brand| major_brand == brand.as_slice())
    {
        return true;
    }

    if bytes.len() <= 16 {
        return false;
    }

    bytes[16..].chunks_exact(4).any(|brand| {
        expected_brands
            .iter()
            .any(|expected| brand == expected.as_slice())
    })
}

fn bytes_are_mp3(bytes: &[u8]) -> bool {
    bytes.starts_with(b"ID3")
        || bytes
            .windows(2)
            .next()
            .is_some_and(|header| header[0] == 0xff && (header[1] & 0xe0) == 0xe0)
}

fn bytes_are_adts_aac(bytes: &[u8]) -> bool {
    bytes
        .windows(2)
        .next()
        .is_some_and(|header| header[0] == 0xff && (header[1] & 0xf0) == 0xf0)
}

fn bytes_are_svg(bytes: &[u8]) -> bool {
    let sample_len = bytes.len().min(512);
    let Ok(sample) = std::str::from_utf8(&bytes[..sample_len]) else {
        return false;
    };

    let sample = sample.trim_start_matches('\u{feff}').trim_start();
    sample.starts_with("<svg") || (sample.starts_with("<?xml") && sample.contains("<svg"))
}

fn bytes_are_gltf_json(bytes: &[u8]) -> bool {
    let sample_len = bytes.len().min(512);
    let Ok(sample) = std::str::from_utf8(&bytes[..sample_len]) else {
        return false;
    };

    let sample = sample.trim_start_matches('\u{feff}').trim_start();
    sample.starts_with('{') && sample.contains("\"asset\"") && sample.contains("\"version\"")
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DownloadUrlKind, MediaType};

    #[test]
    fn test_generate_filename() {
        let downloader = Downloader::default();
        let asset = MediaAsset::builder()
            .id("12345")
            .provider("unsplash")
            .media_type(MediaType::Image)
            .title("Test Image")
            .download_url("https://example.com/image.jpg")
            .source_url("https://unsplash.com/photos/12345")
            .build()
            .expect("test asset should build");

        let filename = downloader.generate_filename(&asset);
        assert_eq!(filename, "unsplash-12345.jpg");
    }

    #[tokio::test]
    async fn download_to_with_receipt_rejects_non_direct_download_url_kind() {
        let downloader = Downloader::default();
        let asset = MediaAsset::builder()
            .id("asset-1")
            .provider("polyhaven")
            .media_type(MediaType::Model3D)
            .title("Model Landing Page")
            .download_url("https://polyhaven.com/a/model")
            .download_url_kind(DownloadUrlKind::LandingPage)
            .source_url("https://polyhaven.com/a/model")
            .build()
            .expect("test asset should build");

        let err = downloader
            .download_to_with_receipt(Path::new("downloads"), &asset)
            .await
            .expect_err("landing page URL must not be downloaded as asset bytes");

        match err {
            DxError::Download { url, message } => {
                assert_eq!(url, "https://polyhaven.com/a/model");
                assert!(message.contains("landing-page"));
                assert!(message.contains("provider resolver"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn download_to_with_receipt_rejects_missing_download_url_kind() {
        let downloader = Downloader::default();
        let asset = MediaAsset::builder()
            .id("asset-2")
            .provider("ambiguous-provider")
            .media_type(MediaType::Image)
            .title("Ambiguous URL")
            .download_url("not-a-direct-url")
            .source_url("https://example.com/source")
            .build()
            .expect("test asset should build");

        let err = downloader
            .download_to_with_receipt(Path::new("downloads"), &asset)
            .await
            .expect_err("missing URL kind must not be downloaded as asset bytes");

        match err {
            DxError::Download { url, message } => {
                assert_eq!(url, "not-a-direct-url");
                assert!(message.contains("unknown"));
                assert!(message.contains("not directly downloadable"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[test]
    fn test_generate_filename_uses_provider_mime_for_extensionless_url() {
        let downloader = Downloader::default();
        let asset = MediaAsset::builder()
            .id("seed-ada")
            .provider("dicebear")
            .media_type(MediaType::Image)
            .title("Avatar")
            .download_url("https://api.dicebear.com/9.x/adventurer/png?seed=ada")
            .source_url("https://www.dicebear.com/")
            .mime_type("image/png")
            .build()
            .expect("test asset should build");

        let filename = downloader.generate_filename(&asset);
        assert_eq!(filename, "dicebear-seed-ada.png");
    }

    #[test]
    fn test_generate_filename_ignores_mime_extension_when_media_type_disagrees() {
        let downloader = Downloader::default();
        let asset = MediaAsset::builder()
            .id("asset-1")
            .provider("fixture")
            .media_type(MediaType::Document)
            .title("Mismatched")
            .download_url("https://example.com/download")
            .source_url("https://example.com/source")
            .mime_type("image/png")
            .build()
            .expect("test asset should build");

        let filename = downloader.generate_filename(&asset);
        assert_eq!(filename, "fixture-asset-1.pdf");
    }

    #[test]
    fn test_extension_from_url() {
        let downloader = Downloader::default();

        assert_eq!(
            downloader.extension_from_url("https://example.com/image.jpg", MediaType::Image),
            Some("jpg")
        );
        assert_eq!(
            downloader.extension_from_url("https://example.com/image.PNG", MediaType::Image),
            Some("png")
        );
        assert_eq!(
            downloader.extension_from_url("https://example.com/image.avif", MediaType::Image),
            Some("avif")
        );
        assert_eq!(
            downloader.extension_from_url("https://example.com/image.bmp", MediaType::Image),
            Some("bmp")
        );
        assert_eq!(
            downloader.extension_from_url("https://example.com/image.tif", MediaType::Image),
            Some("tiff")
        );
        assert_eq!(
            downloader.extension_from_url("https://example.com/video.mp4", MediaType::Video),
            Some("mp4")
        );
        assert_eq!(
            downloader.extension_from_url("https://example.com/unknown", MediaType::Image),
            None
        );
        assert_eq!(
            downloader
                .extension_from_url("https://example.com/archive.jpg/download", MediaType::Image,),
            None
        );
        assert_eq!(
            downloader
                .extension_from_url("https://example.com/image.txt?format=jpg", MediaType::Image,),
            None
        );
        assert_eq!(
            downloader.extension_from_url("https://example.com/animation.gif", MediaType::Gif),
            Some("gif")
        );
    }

    #[test]
    fn direct_url_download_receipt_records_source_and_type_evidence() {
        let output = Downloader::direct_url_download_receipt(
            "https://example.com/photo.png",
            MediaType::Image,
            Path::new("photo.png"),
            Some("image/png"),
            42,
        );

        assert_eq!(
            output.metadata.get("tool.name").map(String::as_str),
            Some("media.download.direct-url")
        );
        assert_eq!(
            output.metadata.get("tool.source_kind").map(String::as_str),
            Some("direct-url")
        );
        assert!(!output.metadata.contains_key("tool.provider"));
        assert_eq!(
            output.metadata.get("tool.source").map(String::as_str),
            Some("https://example.com/photo.png")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.download_url_kind")
                .map(String::as_str),
            Some("direct-file")
        );
        assert_eq!(
            output.metadata.get("tool.license").map(String::as_str),
            Some("unknown")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.license_known")
                .map(String::as_str),
            Some("false")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.type_validation")
                .map(String::as_str),
            Some("pass")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.actual_file_validation")
                .map(String::as_str),
            Some("pass")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.actual_mime_type")
                .map(String::as_str),
            Some("image/png")
        );
    }

    #[test]
    fn direct_url_download_receipt_marks_actual_mime_extension_mismatch() {
        let output = Downloader::direct_url_download_receipt(
            "https://example.com/photo.png",
            MediaType::Image,
            Path::new("photo.png"),
            Some("image/jpeg"),
            42,
        );

        assert_eq!(
            output
                .metadata
                .get("tool.type_validation")
                .map(String::as_str),
            Some("fail")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.type_validation_reason")
                .map(String::as_str),
            Some("actual-mime-extension-mismatch")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.actual_file_validation")
                .map(String::as_str),
            Some("fail")
        );
    }

    #[test]
    fn actual_mime_output_extension_mismatch_is_rejected_before_write() {
        let err = validate_actual_mime_matches_output_path(
            "https://example.com/photo",
            Path::new("photo.jpg"),
            Some("image/png"),
        )
        .expect_err("known MIME/output extension mismatch should reject the download");

        match err {
            DxError::Download { url, message } => {
                assert_eq!(url, "https://example.com/photo");
                assert!(message.contains("image/png"));
                assert!(message.contains("photo.jpg"));
                assert!(message.contains("actual-mime-extension-mismatch"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[test]
    fn expected_mime_comparison_allows_generic_response_mime_for_body_validation() {
        assert!(!response_mime_conflicts_with_expected_mime(
            "application/octet-stream",
            "image/png",
        ));
        assert!(!response_mime_conflicts_with_expected_mime(
            "image/png; charset=binary",
            "image/png",
        ));
        assert!(response_mime_conflicts_with_expected_mime(
            "image/jpeg",
            "image/png",
        ));
    }

    #[test]
    fn image_mime_validation_covers_advertised_image_extensions() {
        assert_eq!(
            extension_from_mime("image/avif", MediaType::Image),
            Some("avif")
        );
        assert_eq!(
            extension_from_mime("image/bmp", MediaType::Image),
            Some("bmp")
        );
        assert_eq!(
            extension_from_mime("image/tiff", MediaType::Image),
            Some("tiff")
        );
        assert_eq!(mime_matches_extension("image/avif", "avif"), Some(true));
        assert_eq!(mime_matches_extension("image/bmp", "bmp"), Some(true));
        assert_eq!(mime_matches_extension("image/tiff", "tif"), Some(true));
        assert_eq!(mime_matches_extension("image/tiff", "tiff"), Some(true));
        assert_eq!(
            extension_from_mime("image/x-icon", MediaType::Image),
            Some("ico")
        );
        assert_eq!(
            extension_from_mime("image/vnd.microsoft.icon", MediaType::Image),
            Some("ico")
        );
        assert_eq!(mime_matches_extension("image/x-icon", "ico"), Some(true));
        assert_eq!(
            mime_matches_extension("image/vnd.microsoft.icon", "ico"),
            Some(true)
        );
    }

    #[test]
    fn extension_from_url_supports_ico_image_outputs() {
        let downloader = Downloader::default();

        assert_eq!(
            downloader.extension_from_url("https://example.com/favicon.ico", MediaType::Image),
            Some("ico")
        );
    }

    #[test]
    fn body_signature_validation_rejects_html_claiming_png() {
        let err = validate_body_signature(
            "https://example.com/fake.png",
            Some("image/png"),
            b"<!doctype html><title>not an image</title>",
        )
        .expect_err("HTML bytes must not validate as PNG");

        match err {
            DxError::Download { url, message } => {
                assert_eq!(url, "https://example.com/fake.png");
                assert!(message.contains("image/png"));
                assert!(message.contains("signature"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[test]
    fn body_signature_validation_rejects_fake_image_when_mime_missing() {
        let err = validate_body_signature(
            "https://example.com/fake.jpg",
            None,
            b"<!doctype html><title>not an image</title>",
        )
        .expect_err("URL extension should drive signature checks when MIME is missing");

        match err {
            DxError::Download { url, message } => {
                assert_eq!(url, "https://example.com/fake.jpg");
                assert!(message.contains("URL extension"));
                assert!(message.contains("JPEG"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[test]
    fn body_signature_validation_rejects_fake_image_when_mime_is_octet_stream() {
        let err = validate_body_signature(
            "https://example.com/fake.png",
            Some("application/octet-stream"),
            b"not a png",
        )
        .expect_err("generic MIME should not bypass URL-extension signature checks");

        match err {
            DxError::Download { url, message } => {
                assert_eq!(url, "https://example.com/fake.png");
                assert!(message.contains("URL extension"));
                assert!(message.contains("PNG"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[test]
    fn body_signature_validation_accepts_real_png_signature_without_mime() {
        let png_bytes = b"\x89PNG\r\n\x1a\nrest";

        assert_eq!(
            validate_body_signature("https://example.com/image.png", None, png_bytes)
                .expect("URL extension and PNG signature should validate"),
            Some("pass")
        );
    }

    #[test]
    fn download_type_evidence_rejects_url_inferred_gif_for_generic_image() {
        let err = validate_download_type_evidence(
            "https://example.com/animated.gif",
            MediaType::Image,
            Some("application/octet-stream"),
            None,
            "provider-mime",
            b"GIF89a\x01\x00\x01\x00",
        )
        .expect_err("generic image downloads must not accept GIF URL/body evidence");

        match err {
            DxError::ContentTypeMismatch { expected, actual } => {
                assert_eq!(expected, "image");
                assert_eq!(actual, "image/gif");
            }
            other => panic!("expected content-type mismatch, got {other:?}"),
        }
    }

    #[test]
    fn download_type_evidence_rejects_url_inferred_svg_for_generic_image() {
        let err = validate_download_type_evidence(
            "https://example.com/icon.svg",
            MediaType::Image,
            None,
            None,
            "provider-mime",
            br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#,
        )
        .expect_err("generic image downloads must not accept SVG URL/body evidence");

        match err {
            DxError::ContentTypeMismatch { expected, actual } => {
                assert_eq!(expected, "image");
                assert_eq!(actual, "image/svg+xml");
            }
            other => panic!("expected content-type mismatch, got {other:?}"),
        }
    }

    #[test]
    fn download_type_evidence_accepts_url_inferred_gif_for_gif_type() {
        assert_eq!(
            validate_download_type_evidence(
                "https://example.com/animated.gif",
                MediaType::Gif,
                Some("application/octet-stream"),
                None,
                "provider-mime",
                b"GIF89a\x01\x00\x01\x00",
            )
            .expect("GIF media type should accept matching GIF URL/body evidence"),
            DownloadTypeEvidence {
                accepted_mime_type: Some("image/gif".to_string()),
                mime_evidence_source: Some("url-extension"),
                body_signature_validation: Some("pass"),
            }
        );
    }

    #[test]
    fn download_type_evidence_accepts_url_inferred_svg_for_vector_type() {
        assert_eq!(
            validate_download_type_evidence(
                "https://example.com/icon.svg",
                MediaType::Vector,
                None,
                None,
                "provider-mime",
                br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#,
            )
            .expect("vector media type should accept matching SVG URL/body evidence"),
            DownloadTypeEvidence {
                accepted_mime_type: Some("image/svg+xml".to_string()),
                mime_evidence_source: Some("url-extension"),
                body_signature_validation: Some("pass"),
            }
        );
    }

    #[test]
    fn download_type_evidence_uses_provider_mime_for_extensionless_response() {
        let png_bytes = b"\x89PNG\r\n\x1a\nrest";

        assert_eq!(
            validate_download_type_evidence(
                "https://cdn.example.com/download?id=avatar",
                MediaType::Image,
                None,
                Some("image/png"),
                "provider-mime",
                png_bytes,
            )
            .expect("provider MIME and PNG signature should validate"),
            DownloadTypeEvidence {
                accepted_mime_type: Some("image/png".to_string()),
                mime_evidence_source: Some("provider-mime"),
                body_signature_validation: Some("pass"),
            }
        );
    }

    #[test]
    fn download_type_evidence_prefers_provider_mime_over_generic_response_url_extension() {
        let png_bytes = b"\x89PNG\r\n\x1a\nrest";

        assert_eq!(
            validate_download_type_evidence(
                "https://cdn.example.com/asset.gif",
                MediaType::Image,
                Some("application/octet-stream"),
                Some("image/png"),
                "provider-mime",
                png_bytes,
            )
            .expect("provider MIME should validate generic response bytes before URL extension"),
            DownloadTypeEvidence {
                accepted_mime_type: Some("image/png".to_string()),
                mime_evidence_source: Some("provider-mime"),
                body_signature_validation: Some("pass"),
            }
        );
    }

    #[test]
    fn provider_mime_download_evidence_is_visible_in_receipt() {
        let asset = MediaAsset::builder()
            .id("avatar-ada")
            .provider("fixture-provider")
            .media_type(MediaType::Image)
            .title("Ada Avatar")
            .direct_download_url("https://cdn.example.com/download?id=avatar")
            .source_url("https://example.com/avatar")
            .mime_type("image/png")
            .build()
            .expect("test asset should build");

        let evidence = validate_download_type_evidence(
            &asset.download_url,
            asset.media_type,
            None,
            asset.mime_type.as_deref(),
            "provider-mime",
            b"\x89PNG\r\n\x1a\nrest",
        )
        .expect("provider MIME and body signature should validate");
        let output = Downloader::download_receipt_for_asset(
            &asset,
            Path::new("avatar.png"),
            evidence.accepted_mime_type.as_deref(),
            12,
        );
        let output = with_download_type_evidence_metadata(output, &evidence, "provider-mime");

        assert_eq!(
            output
                .metadata
                .get("tool.actual_mime_type")
                .map(String::as_str),
            Some("image/png")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.actual_mime_evidence_source")
                .map(String::as_str),
            Some("provider-mime")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.actual_file_validation")
                .map(String::as_str),
            Some("pass")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.actual_body_signature_validation")
                .map(String::as_str),
            Some("pass")
        );
    }

    #[test]
    fn download_type_evidence_rejects_binary_media_without_type_evidence() {
        let err = validate_download_type_evidence(
            "https://cdn.example.com/download?id=avatar",
            MediaType::Image,
            None,
            None,
            "provider-mime",
            b"\x89PNG\r\n\x1a\nrest",
        )
        .expect_err("binary media requires MIME, provider MIME, or URL extension evidence");

        match err {
            DxError::Download { url, message } => {
                assert_eq!(url, "https://cdn.example.com/download?id=avatar");
                assert!(message.contains("missing-type-evidence"));
                assert!(message.contains("Content-Type"));
                assert!(message.contains("provider MIME"));
                assert!(message.contains("URL extension"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[test]
    fn body_signature_validation_accepts_real_png_signature() {
        let png_bytes = b"\x89PNG\r\n\x1a\nrest";

        assert_eq!(
            validate_body_signature(
                "https://example.com/image.png",
                Some("image/png"),
                png_bytes
            )
            .expect("PNG signature should validate"),
            Some("pass")
        );
    }

    #[test]
    fn body_signature_validation_rejects_fake_video_when_mime_missing() {
        let err = validate_body_signature(
            "https://example.com/fake.mp4",
            None,
            b"<!doctype html><title>not a video</title>",
        )
        .expect_err("URL extension should drive video signature checks when MIME is missing");

        match err {
            DxError::Download { url, message } => {
                assert_eq!(url, "https://example.com/fake.mp4");
                assert!(message.contains("MP4"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[test]
    fn body_signature_validation_rejects_fake_audio_when_mime_is_octet_stream() {
        let err = validate_body_signature(
            "https://example.com/fake.mp3",
            Some("application/octet-stream"),
            b"not an mp3",
        )
        .expect_err("generic MIME should not bypass audio signature checks");

        match err {
            DxError::Download { url, message } => {
                assert_eq!(url, "https://example.com/fake.mp3");
                assert!(message.contains("MP3"));
            }
            other => panic!("expected download error, got {other:?}"),
        }
    }

    #[test]
    fn body_signature_validation_accepts_common_media_signatures() {
        let mp4_bytes = b"\0\0\0\x18ftypisom\0\0\0\0isomiso2mp41";
        let webm_bytes = b"\x1A\x45\xDF\xA3webm";
        let mp3_bytes = b"ID3\x04\0\0\0\0\0\x21audio";
        let glb_bytes = b"glTF\x02\0\0\0\x14\0\0\0";
        let svg_bytes = br#"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg"></svg>"#;
        let avif_bytes = b"\0\0\0\x18ftypavif\0\0\0\0avifmif1";

        for (url, mime, bytes) in [
            (
                "https://example.com/video.mp4",
                Some("video/mp4"),
                mp4_bytes.as_slice(),
            ),
            (
                "https://example.com/video.webm",
                Some("video/webm"),
                webm_bytes.as_slice(),
            ),
            (
                "https://example.com/audio.mp3",
                Some("audio/mpeg"),
                mp3_bytes.as_slice(),
            ),
            (
                "https://example.com/model.glb",
                Some("model/gltf-binary"),
                glb_bytes.as_slice(),
            ),
            (
                "https://example.com/vector.svg",
                Some("image/svg+xml"),
                svg_bytes.as_slice(),
            ),
            (
                "https://example.com/image.avif",
                Some("image/avif"),
                avif_bytes.as_slice(),
            ),
        ] {
            assert_eq!(
                validate_body_signature(url, mime, bytes)
                    .unwrap_or_else(|err| panic!("{url} should validate: {err}")),
                Some("pass")
            );
        }
    }
}
