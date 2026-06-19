//! Core types for DX Media.
//!
//! This module defines the fundamental data structures used throughout the library.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumString};

use crate::error::DxError;

// ═══════════════════════════════════════════════════════════════════════════════
// SEARCH MODE
// ═══════════════════════════════════════════════════════════════════════════════

/// Search mode for controlling how providers are queried.
///
/// - **Quantity**: Fast mode with early-exit optimization. Returns results as soon as
///   enough are gathered (3x requested count). Ideal for quick searches.
/// - **Quality**: Waits for ALL providers to respond (or timeout). Gathers the most
///   comprehensive results from all sources. Better for thorough searches.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Display, EnumString,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    /// Fast mode: Early exit after gathering enough results (3x count).
    /// Skips slow providers for speed. DEFAULT mode.
    #[default]
    Quantity,
    /// Thorough mode: Wait for ALL providers to respond (or timeout).
    /// Gets comprehensive results from every source.
    Quality,
}

impl SearchMode {
    /// Returns true if this is quantity (fast) mode.
    #[must_use]
    pub fn is_quantity(&self) -> bool {
        matches!(self, Self::Quantity)
    }

    /// Returns true if this is quality (thorough) mode.
    #[must_use]
    pub fn is_quality(&self) -> bool {
        matches!(self, Self::Quality)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MEDIA TYPE
// ═══════════════════════════════════════════════════════════════════════════════

/// Supported media types for search and download.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    /// Photographs and images (JPEG, PNG, WebP, etc.)
    Image,
    /// Video files (MP4, WebM, etc.)
    Video,
    /// Audio files (MP3, WAV, FLAC, etc.)
    Audio,
    /// GIF animations
    Gif,
    /// Vector graphics (SVG)
    Vector,
    /// Documents (PDF, Word, etc.)
    Document,
    /// Data files (JSON, CSV, datasets)
    Data,
    /// 3D models (OBJ, FBX, GLTF)
    Model3D,
    /// Code snippets and templates
    Code,
    /// Text content (articles, quotes)
    Text,
}

impl MediaType {
    /// Returns all available media types.
    #[must_use]
    pub fn all() -> &'static [MediaType] {
        &[
            Self::Image,
            Self::Video,
            Self::Audio,
            Self::Gif,
            Self::Vector,
            Self::Document,
            Self::Data,
            Self::Model3D,
            Self::Code,
            Self::Text,
        ]
    }

    /// Returns file extensions typically associated with this media type.
    #[must_use]
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Image => &["jpg", "jpeg", "png", "webp", "avif", "bmp", "tiff", "ico"],
            Self::Video => &["mp4", "webm", "mov", "avi", "mkv"],
            Self::Audio => &["mp3", "wav", "flac", "ogg", "aac", "m4a", "wma", "opus"],
            Self::Gif => &["gif"],
            Self::Vector => &["svg", "eps", "ai"],
            Self::Document => &["pdf", "doc", "docx", "ppt", "pptx", "xls", "xlsx", "html"],
            Self::Data => &["json", "csv", "xml", "yaml", "parquet"],
            Self::Model3D => &["obj", "fbx", "gltf", "glb", "blend"],
            Self::Code => &["rs", "py", "js", "ts", "go", "java", "cpp"],
            Self::Text => &["txt", "md", "rst"],
        }
    }

    /// Returns positive MIME prefixes or exact MIME values associated with this media type.
    ///
    /// Use [`Self::matches_mime`] for compatibility checks. Some broad prefixes have
    /// explicit exclusions, exposed by [`Self::mime_exclusions`].
    #[must_use]
    pub fn mime_patterns(&self) -> &'static [&'static str] {
        match self {
            Self::Image => &["image/"],
            Self::Video => &["video/"],
            Self::Audio => &["audio/"],
            Self::Gif => &["image/gif"],
            Self::Vector => &["image/svg+xml", "application/postscript"],
            Self::Document => &[
                "application/pdf",
                "application/msword",
                "application/vnd.openxmlformats-officedocument",
                "text/html",
            ],
            Self::Data => &[
                "application/json",
                "text/csv",
                "application/xml",
                "text/yaml",
            ],
            Self::Model3D => &["model/", "application/octet-stream"],
            Self::Code => &["text/", "application/javascript"],
            Self::Text => &["text/"],
        }
    }

    /// Returns exact MIME values reserved for more specific media types.
    #[must_use]
    pub fn mime_exclusions(&self) -> &'static [&'static str] {
        match self {
            Self::Image => &["image/gif", "image/svg+xml"],
            _ => &[],
        }
    }

    /// Returns true when a MIME type is compatible with this media type.
    #[must_use]
    pub fn matches_mime(&self, mime: &str) -> bool {
        let mime = mime
            .split(';')
            .next()
            .unwrap_or(mime)
            .trim()
            .to_ascii_lowercase();

        match self {
            Self::Image => {
                mime.starts_with("image/")
                    && !self
                        .mime_exclusions()
                        .iter()
                        .any(|excluded| mime == *excluded)
            }
            Self::Vector => mime == "image/svg+xml" || mime == "application/postscript",
            Self::Document => {
                mime == "application/pdf"
                    || mime == "application/msword"
                    || mime.starts_with("application/vnd.openxmlformats-officedocument.")
                    || mime == "text/html"
            }
            _ => self.mime_patterns().iter().any(|pattern| {
                if pattern.ends_with('/') {
                    mime.starts_with(pattern)
                } else {
                    mime == *pattern
                }
            }),
        }
    }

    /// Returns true when an extension is compatible with this media type.
    #[must_use]
    pub fn matches_extension(&self, extension: &str) -> bool {
        let extension = extension.trim_start_matches('.').to_ascii_lowercase();
        self.extensions().contains(&extension.as_str())
    }

    /// Returns the media type as a lowercase string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Gif => "gif",
            Self::Vector => "vector",
            Self::Document => "document",
            Self::Data => "data",
            Self::Model3D => "model3d",
            Self::Code => "code",
            Self::Text => "text",
        }
    }

    /// Returns the plural form of the media type (for directory names).
    #[must_use]
    pub fn as_plural_str(&self) -> &'static str {
        match self {
            Self::Image => "images",
            Self::Video => "videos",
            Self::Audio => "audio",
            Self::Gif => "gifs",
            Self::Vector => "vectors",
            Self::Document => "documents",
            Self::Data => "data",
            Self::Model3D => "models",
            Self::Code => "code",
            Self::Text => "text",
        }
    }
}

/// Source of an asset MIME type hint.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MimeEvidenceSource {
    /// MIME was supplied directly by the provider response.
    ProviderSupplied,
    /// MIME was inferred from a provider URL or filename extension.
    UrlInferred,
    /// MIME was filled from a local default table.
    Defaulted,
}

impl MimeEvidenceSource {
    /// Stable receipt string for this MIME evidence source.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProviderSupplied => "provider-supplied",
            Self::UrlInferred => "url-inferred",
            Self::Defaulted => "defaulted",
        }
    }
}

/// Kind of URL stored in [`MediaAsset::download_url`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DownloadUrlKind {
    /// URL is expected to resolve directly to downloadable asset bytes.
    DirectFile,
    /// URL resolves to a provider preview derivative, not the full asset.
    PreviewDerivative,
    /// URL is a provider asset manifest or directory that needs another selection step.
    AssetManifest,
    /// URL is a human/provider landing page, not a direct asset file.
    LandingPage,
    /// URL role is not known from the provider response.
    #[default]
    Unknown,
}

impl DownloadUrlKind {
    /// Stable receipt string for this download URL kind.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DirectFile => "direct-file",
            Self::PreviewDerivative => "preview-derivative",
            Self::AssetManifest => "asset-manifest",
            Self::LandingPage => "landing-page",
            Self::Unknown => "unknown",
        }
    }
}

/// Validation result for an asset's declared type metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaTypeValidation {
    /// Declared media type.
    pub media_type: MediaType,
    /// Extension inferred from the download URL when available.
    pub extension: Option<String>,
    /// MIME type hint when available.
    pub mime_type: Option<String>,
    /// Where the MIME type evidence came from.
    pub mime_evidence_source: Option<MimeEvidenceSource>,
    /// Whether the extension matches the declared media type.
    pub extension_matches: Option<bool>,
    /// Whether the MIME type matches the declared media type.
    pub mime_matches: Option<bool>,
}

impl MediaTypeValidation {
    /// Returns true when at least one type hint was available to check.
    #[must_use]
    pub fn has_evidence(&self) -> bool {
        self.extension_matches.is_some() || self.mime_matches.is_some()
    }

    /// Returns true only when all available type hints match.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.has_evidence()
            && self.extension_matches.unwrap_or(true)
            && self.mime_matches.unwrap_or(true)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LICENSE
// ═══════════════════════════════════════════════════════════════════════════════

/// License types for media assets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum License {
    /// Creative Commons Zero - Public Domain
    #[strum(serialize = "CC0")]
    Cc0,
    /// Creative Commons Attribution
    #[strum(serialize = "CC-BY")]
    CcBy,
    /// Creative Commons Attribution ShareAlike
    #[strum(serialize = "CC-BY-SA")]
    CcBySa,
    /// Creative Commons Attribution NonCommercial
    #[strum(serialize = "CC-BY-NC")]
    CcByNc,
    /// Public Domain
    PublicDomain,
    /// Unsplash License (free for commercial use with attribution)
    Unsplash,
    /// Pexels License (free for commercial use)
    Pexels,
    /// Pixabay License (free for commercial use)
    Pixabay,
    /// Custom license with description
    Custom(String),
    /// Other or unspecified license
    Other(String),
}

impl License {
    /// Returns the license as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Cc0 => "CC0",
            Self::CcBy => "CC-BY",
            Self::CcBySa => "CC-BY-SA",
            Self::CcByNc => "CC-BY-NC",
            Self::PublicDomain => "Public Domain",
            Self::Unsplash => "Unsplash License",
            Self::Pexels => "Pexels License",
            Self::Pixabay => "Pixabay License",
            Self::Custom(s) => s.as_str(),
            Self::Other(s) => s.as_str(),
        }
    }

    /// Returns true when the value names a concrete license or rights statement.
    #[must_use]
    pub fn is_known(&self) -> bool {
        match self {
            Self::Other(value) => {
                let normalized = value.trim().to_ascii_lowercase();
                !normalized.is_empty()
                    && normalized != "unknown"
                    && normalized != "various"
                    && !normalized.contains("unknown")
                    && !normalized.contains("varies by")
                    && normalized != "giphy"
                    && normalized != "open library"
                    && normalized != "nekos.best"
                    && normalized != "waifu.pics"
                    && normalized != "unsplash"
                    && normalized != "wizards of the coast"
                    && !normalized.contains("not verified")
                    && !normalized.contains("not provided")
                    && !normalized.contains("unverified")
                    && !normalized.contains("provider default")
                    && !normalized.contains("free")
                    && !normalized.contains("check source")
                    && !normalized.contains("check repository")
            }
            _ => true,
        }
    }
}

impl Default for License {
    fn default() -> Self {
        Self::Other("Unknown".to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MEDIA ASSET
// ═══════════════════════════════════════════════════════════════════════════════

/// A downloadable media asset from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAsset {
    /// Unique identifier from the provider.
    pub id: String,
    /// Source provider name.
    pub provider: String,
    /// Media type.
    pub media_type: MediaType,
    /// Asset title or description.
    pub title: String,
    /// Download URL or provider locator URL.
    pub download_url: String,
    /// Whether `download_url` is direct bytes, a manifest, a landing page, or unknown.
    #[serde(default)]
    pub download_url_kind: DownloadUrlKind,
    /// Preview/thumbnail URL.
    pub preview_url: Option<String>,
    /// Web page URL on provider site.
    pub source_url: String,
    /// Author/creator name.
    pub author: Option<String>,
    /// Author profile URL.
    pub author_url: Option<String>,
    /// License information.
    pub license: License,
    /// Width in pixels (for images/videos).
    pub width: Option<u32>,
    /// Height in pixels (for images/videos).
    pub height: Option<u32>,
    /// File size in bytes.
    pub file_size: Option<u64>,
    /// MIME type.
    pub mime_type: Option<String>,
    /// Where the MIME type evidence came from.
    #[serde(default)]
    pub mime_evidence_source: Option<MimeEvidenceSource>,
    /// Tags/keywords.
    pub tags: Vec<String>,
    /// Provider-specific provenance fields preserved from the source API.
    #[serde(default)]
    pub provider_metadata: HashMap<String, String>,
    /// When the asset was indexed.
    pub indexed_at: DateTime<Utc>,
}

impl MediaAsset {
    /// Create a new media asset builder.
    #[must_use]
    pub fn builder() -> MediaAssetBuilder {
        MediaAssetBuilder::default()
    }

    /// Get a safe filename for this asset.
    #[must_use]
    pub fn safe_filename(&self) -> String {
        let title = sanitize_filename::sanitize(&self.title);
        let title = if title.len() > 50 {
            &title[..50]
        } else {
            &title
        };

        let ext = self
            .download_url
            .split('.')
            .last()
            .and_then(|e| e.split('?').next())
            .unwrap_or("bin");

        format!(
            "{}_{}_{}.{}",
            title,
            self.provider,
            &self.id[..8.min(self.id.len())],
            ext
        )
        .replace(' ', "_")
        .to_lowercase()
    }

    /// Returns provenance data suitable for receipts and audit output.
    #[must_use]
    pub fn provenance(&self) -> MediaAssetProvenance {
        MediaAssetProvenance {
            provider: self.provider.clone(),
            source_url: self.source_url.clone(),
            download_url: self.download_url.clone(),
            download_url_kind: self.download_url_kind,
            author: self.author.clone(),
            author_url: self.author_url.clone(),
            license: self.license.as_str().to_string(),
            license_known: self.license.is_known(),
            mime_type: self.mime_type.clone(),
            mime_evidence_source: self.mime_evidence_source,
            media_type: self.media_type,
            provider_metadata: self.provider_metadata.clone(),
            type_validation: self.validate_type_metadata(),
        }
    }

    /// Validate provider MIME and URL extension hints against the declared media type.
    #[must_use]
    pub fn validate_type_metadata(&self) -> MediaTypeValidation {
        let extension = self
            .download_url
            .split('?')
            .next()
            .and_then(|url| url.rsplit('.').next())
            .filter(|ext| !ext.contains('/') && !ext.is_empty())
            .map(str::to_ascii_lowercase);

        let extension_matches = extension
            .as_deref()
            .map(|ext| self.media_type.matches_extension(ext));
        let mime_matches = self
            .mime_type
            .as_deref()
            .map(|mime| self.media_type.matches_mime(mime));

        MediaTypeValidation {
            media_type: self.media_type,
            extension,
            mime_type: self.mime_type.clone(),
            mime_evidence_source: self.mime_evidence_source,
            extension_matches,
            mime_matches,
        }
    }
}

/// Receipt-ready provider provenance for a media asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAssetProvenance {
    /// Source provider name.
    pub provider: String,
    /// Provider landing page or canonical source URL.
    pub source_url: String,
    /// Download URL or provider locator URL used by the tool.
    pub download_url: String,
    /// Whether `download_url` is direct bytes, a manifest, a landing page, or unknown.
    pub download_url_kind: DownloadUrlKind,
    /// Author or creator when provided.
    pub author: Option<String>,
    /// Author profile URL when provided.
    pub author_url: Option<String>,
    /// License label.
    pub license: String,
    /// Whether the license is more specific than Unknown.
    pub license_known: bool,
    /// MIME type hint supplied by provider or inferred with evidence.
    pub mime_type: Option<String>,
    /// Where the MIME type evidence came from.
    pub mime_evidence_source: Option<MimeEvidenceSource>,
    /// Declared media type.
    pub media_type: MediaType,
    /// Provider-specific provenance fields preserved from the source API.
    pub provider_metadata: HashMap<String, String>,
    /// MIME/extension validation report.
    pub type_validation: MediaTypeValidation,
}

/// Builder for [`MediaAsset`].
#[derive(Debug, Default)]
pub struct MediaAssetBuilder {
    id: Option<String>,
    provider: Option<String>,
    media_type: Option<MediaType>,
    title: Option<String>,
    download_url: Option<String>,
    download_url_kind: Option<DownloadUrlKind>,
    preview_url: Option<String>,
    source_url: Option<String>,
    author: Option<String>,
    author_url: Option<String>,
    license: Option<License>,
    width: Option<u32>,
    height: Option<u32>,
    file_size: Option<u64>,
    mime_type: Option<String>,
    mime_evidence_source: Option<MimeEvidenceSource>,
    tags: Vec<String>,
    provider_metadata: HashMap<String, String>,
}

impl MediaAssetBuilder {
    /// Set the asset ID.
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the provider name.
    #[must_use]
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Set the media type.
    #[must_use]
    pub fn media_type(mut self, media_type: MediaType) -> Self {
        self.media_type = Some(media_type);
        self
    }

    /// Set the title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the download URL.
    #[must_use]
    pub fn download_url(mut self, url: impl Into<String>) -> Self {
        self.download_url = Some(url.into());
        self
    }

    /// Set a download URL that is known to return asset bytes directly.
    #[must_use]
    pub fn direct_download_url(mut self, url: impl Into<String>) -> Self {
        self.download_url = Some(url.into());
        self.download_url_kind = Some(DownloadUrlKind::DirectFile);
        self
    }

    /// Set the kind of URL stored in `download_url`.
    #[must_use]
    pub fn download_url_kind(mut self, kind: DownloadUrlKind) -> Self {
        self.download_url_kind = Some(kind);
        self
    }

    /// Set the preview URL.
    #[must_use]
    pub fn preview_url(mut self, url: impl Into<String>) -> Self {
        self.preview_url = Some(url.into());
        self
    }

    /// Set preview URL when available.
    #[must_use]
    pub fn maybe_preview_url(mut self, url: Option<impl Into<String>>) -> Self {
        self.preview_url = url.map(Into::into);
        self
    }

    /// Set the source URL.
    #[must_use]
    pub fn source_url(mut self, url: impl Into<String>) -> Self {
        self.source_url = Some(url.into());
        self
    }

    /// Set the author.
    #[must_use]
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the author URL.
    #[must_use]
    pub fn author_url(mut self, url: impl Into<String>) -> Self {
        self.author_url = Some(url.into());
        self
    }

    /// Set the license.
    #[must_use]
    pub fn license(mut self, license: License) -> Self {
        self.license = Some(license);
        self
    }

    /// Set the dimensions.
    #[must_use]
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set the file size.
    #[must_use]
    pub fn file_size(mut self, size: u64) -> Self {
        self.file_size = Some(size);
        self
    }

    /// Set the file size when a provider supplied it.
    #[must_use]
    pub fn maybe_file_size(mut self, size: Option<u64>) -> Self {
        self.file_size = size;
        self
    }

    /// Set the MIME type supplied by the provider.
    #[must_use]
    pub fn mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self.mime_evidence_source = Some(MimeEvidenceSource::ProviderSupplied);
        self
    }

    /// Set the MIME type with explicit evidence source.
    #[must_use]
    pub fn mime_type_with_evidence(
        mut self,
        mime: impl Into<String>,
        evidence_source: MimeEvidenceSource,
    ) -> Self {
        self.mime_type = Some(mime.into());
        self.mime_evidence_source = Some(evidence_source);
        self
    }

    /// Set a URL-inferred MIME type.
    #[must_use]
    pub fn url_inferred_mime_type(self, mime: impl Into<String>) -> Self {
        self.mime_type_with_evidence(mime, MimeEvidenceSource::UrlInferred)
    }

    /// Set a defaulted MIME type.
    #[must_use]
    pub fn defaulted_mime_type(self, mime: impl Into<String>) -> Self {
        self.mime_type_with_evidence(mime, MimeEvidenceSource::Defaulted)
    }

    /// Set the MIME type when a provider supplied one.
    #[must_use]
    pub fn maybe_mime_type(mut self, mime: Option<impl Into<String>>) -> Self {
        if let Some(mime) = mime {
            self.mime_type = Some(mime.into());
            self.mime_evidence_source = Some(MimeEvidenceSource::ProviderSupplied);
        }
        self
    }

    /// Set the MIME type when evidence is available.
    #[must_use]
    pub fn maybe_mime_type_with_evidence(
        mut self,
        mime: Option<impl Into<String>>,
        evidence_source: MimeEvidenceSource,
    ) -> Self {
        if let Some(mime) = mime {
            self.mime_type = Some(mime.into());
            self.mime_evidence_source = Some(evidence_source);
        }
        self
    }

    /// Set a URL-inferred MIME type when available.
    #[must_use]
    pub fn maybe_url_inferred_mime_type(self, mime: Option<impl Into<String>>) -> Self {
        self.maybe_mime_type_with_evidence(mime, MimeEvidenceSource::UrlInferred)
    }

    /// Set a defaulted MIME type when available.
    #[must_use]
    pub fn maybe_defaulted_mime_type(self, mime: Option<impl Into<String>>) -> Self {
        self.maybe_mime_type_with_evidence(mime, MimeEvidenceSource::Defaulted)
    }

    /// Set a provider-supplied MIME type when available.
    #[must_use]
    pub fn maybe_provider_mime_type(self, mime: Option<impl Into<String>>) -> Self {
        self.maybe_mime_type(mime)
    }

    /// Set the tags.
    #[must_use]
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set provider-specific provenance metadata.
    #[must_use]
    pub fn provider_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.provider_metadata = metadata;
        self
    }

    /// Add one provider-specific provenance metadata field.
    #[must_use]
    pub fn provider_metadata_entry(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.provider_metadata.insert(key.into(), value.into());
        self
    }

    /// Build the media asset.
    ///
    /// # Errors
    ///
    /// Returns `DxError::BuilderValidation` if required fields (id, provider, media_type,
    /// title, download_url, source_url) are not set.
    pub fn build(self) -> Result<MediaAsset, DxError> {
        let id = require_non_empty(self.id, "id")?;
        let provider = require_non_empty(self.provider, "provider")?;
        let media_type = self
            .media_type
            .ok_or(DxError::builder_validation("media_type"))?;
        let title = require_non_empty(self.title, "title")?;
        let download_url = require_non_empty(self.download_url, "download_url")?;
        let source_url = require_non_empty(self.source_url, "source_url")?;
        let mime_evidence_source = if self.mime_type.is_some() {
            Some(
                self.mime_evidence_source
                    .unwrap_or(MimeEvidenceSource::ProviderSupplied),
            )
        } else {
            None
        };

        Ok(MediaAsset {
            id,
            provider,
            media_type,
            title,
            download_url,
            download_url_kind: self.download_url_kind.unwrap_or_default(),
            preview_url: self.preview_url,
            source_url,
            author: self.author,
            author_url: self.author_url,
            license: self.license.unwrap_or_default(),
            width: self.width,
            height: self.height,
            file_size: self.file_size,
            mime_type: self.mime_type,
            mime_evidence_source,
            tags: self.tags,
            provider_metadata: self.provider_metadata,
            indexed_at: Utc::now(),
        })
    }

    /// Build the media asset, returning None if required fields are missing.
    ///
    /// This is useful in iterator chains where you want to silently skip invalid assets.
    ///
    /// # Deprecated
    ///
    /// This method is deprecated. Use [`build()`](Self::build) for explicit error handling
    /// or [`build_or_log()`](Self::build_or_log) for logging failures at debug level.
    #[deprecated(
        since = "1.0.0",
        note = "Use `build()` for explicit error handling or `build_or_log()` for logging failures"
    )]
    #[must_use]
    pub fn try_build(self) -> Option<MediaAsset> {
        match self.build() {
            Ok(asset) => Some(asset),
            Err(DxError::BuilderValidation { field }) => {
                tracing::warn!(
                    missing_field = %field,
                    "MediaAssetBuilder.build_or_log() failed: missing required field"
                );
                None
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "MediaAssetBuilder.build_or_log() failed"
                );
                None
            }
        }
    }

    /// Build the media asset, logging at debug level if required fields are missing.
    ///
    /// This is useful in iterator chains where you want to skip invalid assets
    /// while still having visibility into failures via debug logs.
    ///
    /// # Returns
    ///
    /// - `Some(MediaAsset)` if all required fields are set
    /// - `None` if any required field is missing (logs at debug level)
    ///
    /// # Example
    ///
    /// ```
    /// use dx_media::types::{MediaAsset, MediaType};
    ///
    /// let asset = MediaAsset::builder()
    ///     .id("123")
    ///     .provider("test")
    ///     .media_type(MediaType::Image)
    ///     .title("Test")
    ///     .download_url("https://example.com/image.jpg")
    ///     .source_url("https://example.com")
    ///     .build_or_log();
    ///
    /// assert!(asset.is_some());
    /// ```
    #[must_use]
    pub fn build_or_log(self) -> Option<MediaAsset> {
        match self.build() {
            Ok(asset) => Some(asset),
            Err(DxError::BuilderValidation { field }) => {
                tracing::debug!(
                    missing_field = %field,
                    "MediaAssetBuilder.build_or_log() skipping asset: missing required field"
                );
                None
            }
            Err(e) => {
                tracing::debug!(
                    error = %e,
                    "MediaAssetBuilder.build_or_log() skipping asset"
                );
                None
            }
        }
    }

    /// Build the media asset, panicking if required fields are missing.
    ///
    /// Use only in tests or when you've validated inputs.
    ///
    /// # Panics
    ///
    /// Panics if required fields (id, provider, media_type, title, download_url, source_url)
    /// are not set.
    #[cfg(test)]
    #[must_use]
    pub fn build_unchecked(self) -> MediaAsset {
        self.build()
            .expect("MediaAssetBuilder missing required fields")
    }
}

fn require_non_empty(value: Option<String>, field: &'static str) -> Result<String, DxError> {
    let value = value.ok_or(DxError::builder_validation(field))?;
    if value.trim().is_empty() {
        return Err(DxError::builder_validation(field));
    }
    Ok(value)
}

// ═══════════════════════════════════════════════════════════════════════════════
// SEARCH QUERY
// ═══════════════════════════════════════════════════════════════════════════════

/// Search query parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search query string.
    pub query: String,
    /// Media type to search for (None = all types).
    pub media_type: Option<MediaType>,
    /// Maximum number of results.
    pub count: usize,
    /// Page number (1-indexed).
    pub page: usize,
    /// Specific providers to search (empty = all).
    pub providers: Vec<String>,
    /// Minimum width filter.
    pub min_width: Option<u32>,
    /// Minimum height filter.
    pub min_height: Option<u32>,
    /// Orientation filter.
    pub orientation: Option<Orientation>,
    /// Color filter (hex or name).
    pub color: Option<String>,
    /// Search mode (Quantity=fast early-exit, Quality=wait for all).
    #[serde(default)]
    pub mode: SearchMode,
}

impl SearchQuery {
    /// Create a new search query.
    #[must_use]
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            media_type: None,
            count: 10,
            page: 1,
            providers: Vec::new(),
            min_width: None,
            min_height: None,
            orientation: None,
            color: None,
            mode: SearchMode::default(),
        }
    }

    /// Create a search query for a specific media type.
    #[must_use]
    pub fn for_type(query: impl Into<String>, media_type: MediaType) -> Self {
        Self {
            query: query.into(),
            media_type: Some(media_type),
            count: 10,
            page: 1,
            providers: Vec::new(),
            min_width: None,
            min_height: None,
            orientation: None,
            color: None,
            mode: SearchMode::default(),
        }
    }

    /// Set the media type filter.
    #[must_use]
    pub fn media_type(mut self, media_type: MediaType) -> Self {
        self.media_type = Some(media_type);
        self
    }

    /// Set the result count.
    #[must_use]
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Set the page number.
    #[must_use]
    pub fn page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }

    /// Set the search mode.
    #[must_use]
    pub fn mode(mut self, mode: SearchMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set specific providers to search.
    #[must_use]
    pub fn providers(mut self, providers: Vec<String>) -> Self {
        self.providers = providers;
        self
    }

    /// Set minimum dimensions.
    #[must_use]
    pub fn min_dimensions(mut self, width: u32, height: u32) -> Self {
        self.min_width = Some(width);
        self.min_height = Some(height);
        self
    }

    /// Set orientation filter.
    #[must_use]
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }
}

/// Image orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    /// Landscape (wider than tall).
    Landscape,
    /// Portrait (taller than wide).
    Portrait,
    /// Square (equal dimensions).
    Square,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SEARCH RESULT
// ═══════════════════════════════════════════════════════════════════════════════

/// Results from a search operation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchResult {
    /// The original query.
    #[serde(default)]
    pub query: String,
    /// Media type searched (None if all types).
    #[serde(default)]
    pub media_type: Option<MediaType>,
    /// Total results available (across all pages).
    #[serde(default)]
    pub total_count: usize,
    /// Assets returned in this page.
    #[serde(default)]
    pub assets: Vec<MediaAsset>,
    /// Providers that were searched.
    #[serde(default)]
    pub providers_searched: Vec<String>,
    /// Providers that failed (with error messages).
    #[serde(default)]
    pub provider_errors: Vec<(String, String)>,
    /// Search duration in milliseconds.
    #[serde(default)]
    pub duration_ms: u64,
    /// Per-provider timing in milliseconds for debugging.
    #[serde(default)]
    pub provider_timings: std::collections::HashMap<String, u64>,
}

/// Search result view that serializes receipt-ready provenance beside every asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultWithProvenance {
    /// The original query.
    pub query: String,
    /// Media type searched (None if all types).
    pub media_type: Option<MediaType>,
    /// Total results available (across all pages).
    pub total_count: usize,
    /// Assets returned in this page, each with computed provenance evidence.
    pub assets: Vec<MediaAssetWithProvenance>,
    /// Providers that were searched.
    pub providers_searched: Vec<String>,
    /// Providers that failed (with error messages).
    pub provider_errors: Vec<(String, String)>,
    /// Search duration in milliseconds.
    pub duration_ms: u64,
    /// Per-provider timing in milliseconds for debugging.
    pub provider_timings: HashMap<String, u64>,
}

/// Media asset view with nested receipt-ready provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAssetWithProvenance {
    /// The original asset fields.
    #[serde(flatten)]
    pub asset: MediaAsset,
    /// Source, license, provider metadata, and type-validation evidence.
    pub provenance: MediaAssetProvenance,
}

impl MediaAssetWithProvenance {
    /// Create an output view from an asset.
    #[must_use]
    pub fn from_asset(asset: &MediaAsset) -> Self {
        Self {
            asset: asset.clone(),
            provenance: asset.provenance(),
        }
    }
}

impl SearchResult {
    /// Create a new empty search result.
    #[must_use]
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            media_type: None,
            total_count: 0,
            assets: Vec::new(),
            providers_searched: Vec::new(),
            provider_errors: Vec::new(),
            duration_ms: 0,
            provider_timings: std::collections::HashMap::new(),
        }
    }

    /// Create a new empty search result for a specific media type.
    #[must_use]
    pub fn for_type(query: impl Into<String>, media_type: MediaType) -> Self {
        Self {
            query: query.into(),
            media_type: Some(media_type),
            total_count: 0,
            assets: Vec::new(),
            providers_searched: Vec::new(),
            provider_errors: Vec::new(),
            duration_ms: 0,
            provider_timings: std::collections::HashMap::new(),
        }
    }

    /// Check if the search returned any results.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Get the number of assets in this result.
    #[must_use]
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Return a serializable search view with receipt-ready provenance for every asset.
    #[must_use]
    pub fn with_asset_provenance(&self) -> SearchResultWithProvenance {
        SearchResultWithProvenance {
            query: self.query.clone(),
            media_type: self.media_type,
            total_count: self.total_count,
            assets: self
                .assets
                .iter()
                .map(MediaAssetWithProvenance::from_asset)
                .collect(),
            providers_searched: self.providers_searched.clone(),
            provider_errors: self.provider_errors.clone(),
            duration_ms: self.duration_ms,
            provider_timings: self.provider_timings.clone(),
        }
    }

    /// Merge another search result into this one.
    pub fn merge(&mut self, other: SearchResult) {
        self.total_count += other.total_count;
        self.assets.extend(other.assets);
        self.providers_searched.extend(other.providers_searched);
        self.provider_errors.extend(other.provider_errors);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMIT CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

/// Rate limit configuration for a provider.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    /// Maximum requests allowed.
    pub requests: u32,
    /// Time period in seconds.
    pub period_secs: u64,
}

impl RateLimitConfig {
    /// Create a new rate limit configuration.
    #[must_use]
    pub const fn new(requests: u32, period_secs: u64) -> Self {
        Self {
            requests,
            period_secs,
        }
    }

    /// No rate limiting.
    #[must_use]
    pub const fn unlimited() -> Self {
        Self {
            requests: u32::MAX,
            period_secs: 1,
        }
    }

    /// Calculate delay between requests in milliseconds.
    #[must_use]
    pub const fn delay_ms(&self) -> u64 {
        if self.requests == 0 {
            return 0;
        }
        (self.period_secs * 1000) / self.requests as u64
    }

    /// Check if rate limiting is enabled.
    #[must_use]
    pub const fn is_limited(&self) -> bool {
        self.requests != u32::MAX
    }

    /// Get the number of requests per window (alias for requests).
    #[must_use]
    pub const fn requests_per_window(&self) -> u32 {
        self.requests
    }

    /// Get the window duration in seconds (alias for period_secs).
    #[must_use]
    pub const fn window_secs(&self) -> u64 {
        self.period_secs
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::new(100, 60) // 100 requests per minute
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HEALTH CHECK
// ═══════════════════════════════════════════════════════════════════════════════

/// Result of a health check for a single provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Provider name.
    pub provider: String,
    /// Whether the provider is available and responding.
    pub available: bool,
    /// Response latency in milliseconds (if available).
    pub latency_ms: Option<u64>,
    /// Error message if the check failed.
    pub error: Option<String>,
    /// Circuit breaker state.
    pub circuit_state: String,
}

impl HealthCheckResult {
    /// Create a successful health check result.
    #[must_use]
    pub fn success(provider: impl Into<String>, latency_ms: u64, circuit_state: &str) -> Self {
        Self {
            provider: provider.into(),
            available: true,
            latency_ms: Some(latency_ms),
            error: None,
            circuit_state: circuit_state.to_string(),
        }
    }

    /// Create a failed health check result.
    #[must_use]
    pub fn failure(
        provider: impl Into<String>,
        error: impl Into<String>,
        circuit_state: &str,
    ) -> Self {
        Self {
            provider: provider.into(),
            available: false,
            latency_ms: None,
            error: Some(error.into()),
            circuit_state: circuit_state.to_string(),
        }
    }
}

/// Comprehensive health report for all providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Health check results for each provider.
    pub providers: Vec<HealthCheckResult>,
    /// Timestamp when the health check was performed.
    pub timestamp: DateTime<Utc>,
    /// Total number of providers checked.
    pub total_providers: usize,
    /// Number of healthy providers.
    pub healthy_count: usize,
    /// Number of unhealthy providers.
    pub unhealthy_count: usize,
    /// Overall health check duration in milliseconds.
    pub duration_ms: u64,
}

impl HealthReport {
    /// Create a new health report.
    #[must_use]
    pub fn new(providers: Vec<HealthCheckResult>, duration_ms: u64) -> Self {
        let total_providers = providers.len();
        let healthy_count = providers.iter().filter(|p| p.available).count();
        let unhealthy_count = total_providers - healthy_count;

        Self {
            providers,
            timestamp: Utc::now(),
            total_providers,
            healthy_count,
            unhealthy_count,
            duration_ms,
        }
    }

    /// Check if all providers are healthy.
    #[must_use]
    pub fn all_healthy(&self) -> bool {
        self.unhealthy_count == 0
    }

    /// Get the list of unhealthy providers.
    #[must_use]
    pub fn unhealthy_providers(&self) -> Vec<&HealthCheckResult> {
        self.providers.iter().filter(|p| !p.available).collect()
    }

    /// Get the list of healthy providers.
    #[must_use]
    pub fn healthy_providers(&self) -> Vec<&HealthCheckResult> {
        self.providers.iter().filter(|p| p.available).collect()
    }

    /// Get the average latency of healthy providers.
    #[must_use]
    pub fn average_latency_ms(&self) -> Option<u64> {
        let latencies: Vec<u64> = self.providers.iter().filter_map(|p| p.latency_ms).collect();

        if latencies.is_empty() {
            None
        } else {
            Some(latencies.iter().sum::<u64>() / latencies.len() as u64)
        }
    }
}
