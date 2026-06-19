//! Registry metadata for media tools.
//!
//! This is source-owned discoverability data. It should describe only tools that
//! are declared by the CLI/API surface, and it should stay honest about feature
//! gates, external dependencies, and credential requirements.

use serde::Serialize;

use crate::tools::{ToolCategory, ToolSourceKind};

/// Readiness state for a declared tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolReadiness {
    /// Implemented with local Rust code or standard library/runtime behavior.
    Local,
    /// Implemented when a Cargo feature is enabled.
    FeatureGated,
    /// Requires a local external executable.
    ExternalDependency,
    /// Requires configured provider credentials.
    RequiresCredentials,
    /// Declared by the CLI but not wired to a working implementation yet.
    DeclaredOnly,
}

impl ToolReadiness {
    /// Stable string for CLI/API output.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::FeatureGated => "feature-gated",
            Self::ExternalDependency => "external-dependency",
            Self::RequiresCredentials => "requires-credentials",
            Self::DeclaredOnly => "declared-only",
        }
    }
}

/// Listing-level receipt availability for a declared tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolReceiptReadiness {
    /// The runtime path is expected to emit a tool receipt when it runs.
    RuntimeReceipt,
    /// The command returns per-asset provenance, not a ToolOutput receipt.
    AssetProvenance,
    /// The command is listed but has no working runtime receipt yet.
    DeclaredOnly,
    /// The command cannot emit a real provider result until credentials are configured.
    RequiresCredentials,
}

impl ToolReceiptReadiness {
    /// Stable string for CLI/API output.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RuntimeReceipt => "runtime-receipt",
            Self::AssetProvenance => "asset-provenance",
            Self::DeclaredOnly => "declared-only",
            Self::RequiresCredentials => "requires-credentials",
        }
    }
}

/// Listing-level type-validation evidence for a declared tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolTypeValidationReadiness {
    /// Provider MIME/extension metadata is surfaced with the asset.
    ProviderMetadata,
    /// Provider metadata and output path validation are expected at runtime.
    ProviderAndOutput,
    /// Output type validation is extension based.
    Extension,
    /// Runtime can record extension presence, but content bytes are arbitrary.
    ExtensionPresence,
    /// Type validation is not meaningful for this metadata/text-only tool.
    NotApplicable,
    /// The command is listed but has no working runtime validation yet.
    DeclaredOnly,
    /// The command cannot validate a real provider result until credentials are configured.
    RequiresCredentials,
}

impl ToolTypeValidationReadiness {
    /// Stable string for CLI/API output.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProviderMetadata => "provider-metadata",
            Self::ProviderAndOutput => "provider-and-output",
            Self::Extension => "extension",
            Self::ExtensionPresence => "extension-presence",
            Self::NotApplicable => "not-applicable",
            Self::DeclaredOnly => "declared-only",
            Self::RequiresCredentials => "requires-credentials",
        }
    }
}

/// Describes a media tool without claiming runtime proof.
#[derive(Debug, Clone, Copy)]
pub struct ToolDescriptor {
    /// Stable command-style name.
    pub name: &'static str,
    /// Tool category.
    pub category: ToolCategory,
    /// One-line purpose.
    pub description: &'static str,
    /// Output source kind.
    pub source_kind: ToolSourceKind,
    /// Readiness state.
    pub readiness: ToolReadiness,
    /// Required feature flag, if any.
    pub feature: Option<&'static str>,
    /// Required local dependency, if any.
    pub dependency: Option<&'static str>,
    /// Accepted input media kinds or values.
    pub input_types: &'static [&'static str],
    /// Produced output media kinds or values.
    pub output_types: &'static [&'static str],
}

impl ToolDescriptor {
    /// True when the tool is available through the Rust API/registry but no CLI route is wired.
    #[must_use]
    pub fn api_only(&self) -> bool {
        api_only_without_cli_command(self.name)
    }

    /// True when the tool needs provider or service credentials before producing real output.
    #[must_use]
    pub fn requires_credentials(&self) -> bool {
        self.readiness == ToolReadiness::RequiresCredentials
            || self.source_kind == ToolSourceKind::RequiresCredentials
    }

    /// Stable credential status for listing output.
    #[must_use]
    pub fn credential_status(&self) -> &'static str {
        if self.requires_credentials() {
            "required"
        } else {
            "not-required"
        }
    }

    /// Stable external dependency status for listing output.
    #[must_use]
    pub fn external_dependency_status(&self) -> &'static str {
        if self.requires_credentials() {
            return "requires-credentials";
        }
        if self.dependency.is_some() {
            return "not-checked";
        }

        "not-required"
    }

    /// Return discoverable CLI command paths for this tool.
    ///
    /// Empty means the tool is discoverable through the Rust API/registry, but
    /// no CLI parser route is wired yet.
    #[must_use]
    pub fn command_paths(&self) -> Vec<String> {
        if let Some(media_path) = media_command_path(self.name) {
            return vec![media_path.to_string()];
        }
        if api_only_without_cli_command(self.name) {
            return Vec::new();
        }

        let command = self
            .name
            .split_once('.')
            .map_or(self.name, |(_, command)| command);
        let mut paths = vec![format!("media {} {}", self.category.as_str(), command)];

        if let Some(legacy_path) = legacy_tool_command_path(self.name) {
            paths.push(legacy_path.to_string());
        }

        paths
    }

    /// Return discoverable CLI routes with route-local readiness evidence.
    #[must_use]
    pub fn command_routes(&self) -> Vec<ToolRouteRecord> {
        self.command_paths()
            .into_iter()
            .map(|path| self.command_route_record(path))
            .collect()
    }

    fn command_route_record(&self, path: String) -> ToolRouteRecord {
        let surface = route_surface(&path);

        ToolRouteRecord {
            path,
            surface,
            readiness: self.route_readiness().as_str(),
            receipt_readiness: self.route_receipt_readiness(surface).as_str(),
            type_validation_readiness: self.route_type_validation_readiness(surface).as_str(),
        }
    }

    fn route_readiness(&self) -> ToolReadiness {
        if listed_without_runtime_receipt(self.name) {
            ToolReadiness::DeclaredOnly
        } else {
            self.readiness
        }
    }

    fn route_receipt_readiness(&self, surface: &str) -> ToolReceiptReadiness {
        if legacy_route_without_receipt(self.name, surface) {
            ToolReceiptReadiness::DeclaredOnly
        } else {
            self.receipt_readiness()
        }
    }

    fn route_type_validation_readiness(&self, surface: &str) -> ToolTypeValidationReadiness {
        if legacy_route_without_receipt(self.name, surface) {
            ToolTypeValidationReadiness::DeclaredOnly
        } else {
            self.type_validation_readiness()
        }
    }

    /// Return the listing-level receipt availability for this tool.
    #[must_use]
    pub fn receipt_readiness(&self) -> ToolReceiptReadiness {
        if self.name == "media.search" {
            return ToolReceiptReadiness::AssetProvenance;
        }
        if stdout_only_without_tool_receipt(self.name) {
            return ToolReceiptReadiness::DeclaredOnly;
        }
        if listed_without_runtime_receipt(self.name) {
            return ToolReceiptReadiness::DeclaredOnly;
        }

        match self.readiness {
            ToolReadiness::DeclaredOnly => ToolReceiptReadiness::DeclaredOnly,
            ToolReadiness::RequiresCredentials => ToolReceiptReadiness::RequiresCredentials,
            ToolReadiness::Local
            | ToolReadiness::FeatureGated
            | ToolReadiness::ExternalDependency => ToolReceiptReadiness::RuntimeReceipt,
        }
    }

    /// Return the listing-level type-validation evidence available for this tool.
    #[must_use]
    pub fn type_validation_readiness(&self) -> ToolTypeValidationReadiness {
        if stdout_only_without_tool_receipt(self.name) {
            return ToolTypeValidationReadiness::NotApplicable;
        }
        if listed_without_runtime_receipt(self.name) {
            return ToolTypeValidationReadiness::DeclaredOnly;
        }

        match self.readiness {
            ToolReadiness::DeclaredOnly => ToolTypeValidationReadiness::DeclaredOnly,
            ToolReadiness::RequiresCredentials => ToolTypeValidationReadiness::RequiresCredentials,
            ToolReadiness::Local
            | ToolReadiness::FeatureGated
            | ToolReadiness::ExternalDependency => {
                if self.name == "media.download.direct-url" {
                    ToolTypeValidationReadiness::Extension
                } else if self.name == "utility.base64-encode" {
                    ToolTypeValidationReadiness::NotApplicable
                } else if self.name == "utility.base64-decode" {
                    ToolTypeValidationReadiness::ExtensionPresence
                } else if input_needs_extension_validation(self.name, self.input_types) {
                    ToolTypeValidationReadiness::Extension
                } else if self.source_kind == ToolSourceKind::ProviderBacked {
                    ToolTypeValidationReadiness::ProviderMetadata
                } else if outputs_need_extension_validation(self.output_types) {
                    ToolTypeValidationReadiness::Extension
                } else {
                    ToolTypeValidationReadiness::NotApplicable
                }
            }
        }
    }

    /// Return runtime receipt names used by implementation paths for this descriptor.
    #[must_use]
    pub fn implementation_receipt_names(&self) -> &'static [&'static str] {
        implementation_receipt_names(self.name)
    }
}

/// Route-local honesty metadata for one CLI entry point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolRouteRecord {
    /// CLI path shown to users.
    pub path: String,
    /// CLI surface that owns this route.
    pub surface: &'static str,
    /// Route-local implementation readiness as a stable string.
    pub readiness: &'static str,
    /// Route-local receipt availability as a stable string.
    pub receipt_readiness: &'static str,
    /// Route-local type-validation evidence as a stable string.
    pub type_validation_readiness: &'static str,
}

/// Machine-readable descriptor for CLI/API listing output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolDescriptorRecord {
    /// Stable command-style name.
    pub name: &'static str,
    /// Tool category as a stable lowercase string.
    pub category: &'static str,
    /// One-line purpose.
    pub description: &'static str,
    /// Output source kind as a stable string.
    pub source_kind: &'static str,
    /// Readiness state as a stable string.
    pub readiness: &'static str,
    /// True when no CLI parser route is wired for this tool.
    pub api_only: bool,
    /// True when credentials are required before producing real output.
    pub requires_credentials: bool,
    /// Stable credential status string.
    pub credential_status: &'static str,
    /// Static external dependency status; this listing does not probe the host.
    pub external_dependency_status: &'static str,
    /// CLI command paths that expose this tool; empty when no CLI route exists yet.
    pub command_paths: Vec<String>,
    /// CLI routes with route-local readiness and receipt evidence.
    pub routes: Vec<ToolRouteRecord>,
    /// Receipt availability as a stable string.
    pub receipt_readiness: &'static str,
    /// Type-validation evidence as a stable string.
    pub type_validation_readiness: &'static str,
    /// Runtime receipt names emitted by implementation helpers for this descriptor.
    pub implementation_receipt_names: &'static [&'static str],
    /// Required feature flag, if any.
    pub feature: Option<&'static str>,
    /// Required local dependency, credential group, or provider dependency.
    pub dependency: Option<&'static str>,
    /// Accepted input media kinds or values.
    pub input_types: &'static [&'static str],
    /// Produced output media kinds or values.
    pub output_types: &'static [&'static str],
}

impl From<&ToolDescriptor> for ToolDescriptorRecord {
    fn from(tool: &ToolDescriptor) -> Self {
        Self {
            name: tool.name,
            category: tool.category.as_str(),
            description: tool.description,
            source_kind: tool.source_kind.as_str(),
            readiness: tool.readiness.as_str(),
            api_only: tool.api_only(),
            requires_credentials: tool.requires_credentials(),
            credential_status: tool.credential_status(),
            external_dependency_status: tool.external_dependency_status(),
            command_paths: tool.command_paths(),
            routes: tool.command_routes(),
            receipt_readiness: tool.receipt_readiness().as_str(),
            type_validation_readiness: tool.type_validation_readiness().as_str(),
            implementation_receipt_names: tool.implementation_receipt_names(),
            feature: tool.feature,
            dependency: tool.dependency,
            input_types: tool.input_types,
            output_types: tool.output_types,
        }
    }
}

macro_rules! tool {
    ($name:literal, $category:ident, $description:literal, $source:ident, $readiness:ident, $feature:expr, $dependency:expr, [$($input:literal),* $(,)?], [$($output:literal),* $(,)?]) => {
        ToolDescriptor {
            name: $name,
            category: ToolCategory::$category,
            description: $description,
            source_kind: ToolSourceKind::$source,
            readiness: ToolReadiness::$readiness,
            feature: $feature,
            dependency: $dependency,
            input_types: &[$($input),*],
            output_types: &[$($output),*],
        }
    };
}

fn media_command_path(name: &str) -> Option<&'static str> {
    match name {
        "media.search" => Some("media search"),
        "media.download" => Some("media download --provider <provider> <asset-id>"),
        "media.download.direct-url" => Some("media download <url>"),
        _ => None,
    }
}

fn api_only_without_cli_command(name: &str) -> bool {
    matches!(
        name,
        "audio.transcribe"
            | "audio.generate-subtitles"
            | "audio.detect-language"
            | "audio.prepare-for-transcription"
            | "audio.extract-speech-segments"
            | "audio.analyze-levels"
    )
}

fn legacy_tool_command_path(name: &str) -> Option<&'static str> {
    match name {
        "image.convert" => Some("media tools image convert"),
        "image.resize" => Some("media tools image resize"),
        "image.favicon" => Some("media tools image favicon"),
        "video.transcode" => Some("media tools video convert"),
        "video.extract-audio" => Some("media tools video extract-audio"),
        "video.to-gif" => Some("media tools video to-gif"),
        "audio.convert" => Some("media tools audio convert"),
        "audio.trim" => Some("media tools audio trim"),
        "archive.zip" => Some("media tools archive zip"),
        "archive.unzip" => Some("media tools archive extract"),
        _ => None,
    }
}

fn route_surface(path: &str) -> &'static str {
    if path.starts_with("media tools ") {
        "legacy-tools-cli"
    } else {
        "unified-cli"
    }
}

fn legacy_route_without_receipt(name: &str, surface: &str) -> bool {
    surface == "legacy-tools-cli" && legacy_tool_command_path(name).is_some()
}

fn outputs_need_extension_validation(outputs: &[&str]) -> bool {
    outputs.iter().any(|output| {
        matches!(
            *output,
            "archive"
                | "audio"
                | "document"
                | "file"
                | "gif"
                | "html"
                | "ico"
                | "image"
                | "json"
                | "pdf"
                | "png"
                | "svg"
                | "tar"
                | "text"
                | "video"
                | "yaml"
                | "zip"
        )
    })
}

fn input_needs_extension_validation(name: &str, inputs: &[&str]) -> bool {
    name == "archive.list" && inputs.contains(&"zip")
}

fn implementation_receipt_names(name: &str) -> &'static [&'static str] {
    match name {
        "media.download.direct-url" => &["media.download.direct-url"],
        "archive.zip" => &["archive.zip.native"],
        "archive.unzip" => &["archive.unzip.native"],
        "archive.list" => &["archive.list-zip.native"],
        "document.markdown-to-html" => &["document.markdown-to-html.native"],
        "document.extract-text" => &["document.extract-text", "document.extract-text.native"],
        "image.convert" => &["image.convert"],
        "image.resize" => &["image.resize"],
        "image.compress" => &["image.compress"],
        "image.favicon" => &["image.generate-icons-from-svg"],
        "image.palette" => &["image.palette"],
        "audio.convert" => &["audio.convert"],
        "audio.analyze-levels" => &["audio.analyze-levels"],
        "audio.prepare-for-transcription" => &["audio.prepare-for-transcription"],
        "audio.extract-speech-segments" => &["audio.extract-speech-segments"],
        "utility.hash" => &["utility.hash"],
        "utility.base64-encode" => &["utility.base64-encode"],
        "utility.base64-decode" => &["utility.base64-decode"],
        "utility.find-duplicates" => &["utility.find-duplicates"],
        "utility.verify-checksum" => &["utility.verify-checksum"],
        "utility.format-json" => &["utility.format-json"],
        "utility.json-to-yaml" => &["utility.json-to-yaml"],
        "utility.yaml-to-json" => &["utility.yaml-to-json"],
        _ => &[],
    }
}

fn stdout_only_without_tool_receipt(name: &str) -> bool {
    matches!(
        name,
        "utility.url-encode"
            | "utility.url-decode"
            | "utility.uuid"
            | "utility.validate-uuid"
            | "utility.timestamp"
    )
}

fn listed_without_runtime_receipt(name: &str) -> bool {
    matches!(
        name,
        "image.ocr"
            | "video.transcode"
            | "video.extract-audio"
            | "video.trim"
            | "video.scale"
            | "video.to-gif"
            | "video.thumbnail"
            | "video.mute"
            | "video.watermark"
            | "video.speed"
            | "video.concat"
            | "video.subtitles"
            | "audio.trim"
            | "audio.merge"
            | "audio.normalize"
            | "audio.remove-silence"
            | "audio.split"
            | "audio.effects"
            | "audio.spectrum"
            | "audio.metadata"
            | "archive.tar"
            | "archive.untar"
            | "archive.gzip"
            | "archive.gunzip"
            | "document.pdf-merge"
            | "document.pdf-split"
            | "document.pdf-compress"
            | "document.pdf-encrypt"
            | "document.pdf-watermark"
            | "document.pdf-to-image"
            | "document.html-to-pdf"
            | "utility.convert-csv"
    )
}

/// Returns all declared media-processing tools.
#[must_use]
pub fn all_tool_descriptors() -> &'static [ToolDescriptor] {
    TOOL_DESCRIPTORS
}

/// Returns all declared tools as stable machine-readable records.
#[must_use]
pub fn tool_descriptor_records() -> Vec<ToolDescriptorRecord> {
    TOOL_DESCRIPTORS
        .iter()
        .map(ToolDescriptorRecord::from)
        .collect()
}

/// Returns declared tools in one category as stable machine-readable records.
#[must_use]
pub fn tool_descriptor_records_for_category(category: &str) -> Vec<ToolDescriptorRecord> {
    let category = category.to_ascii_lowercase();
    TOOL_DESCRIPTORS
        .iter()
        .filter(|tool| tool.category.as_str() == category)
        .map(ToolDescriptorRecord::from)
        .collect()
}

const TOOL_DESCRIPTORS: &[ToolDescriptor] = &[
    tool!(
        "media.search",
        Media,
        "Search provider-backed media assets",
        ProviderBacked,
        Local,
        None,
        None,
        ["query", "provider-filter", "media-type"],
        ["media-assets", "provenance"]
    ),
    tool!(
        "media.download",
        Media,
        "Download a provider-backed media asset by provider asset ID",
        ProviderBacked,
        DeclaredOnly,
        None,
        None,
        ["provider-name", "provider-asset-id"],
        ["file", "tool-receipt"]
    ),
    tool!(
        "media.download.direct-url",
        Media,
        "Download a caller-supplied media URL with direct-url receipt metadata",
        DirectUrl,
        Local,
        None,
        None,
        ["direct-url", "output-directory"],
        ["file", "tool-receipt"]
    ),
    tool!(
        "image.convert",
        Image,
        "Convert image formats; unified CLI uses native image-core, compatibility API uses ImageMagick",
        LocalOnly,
        FeatureGated,
        Some("image-core"),
        Some("imagemagick"),
        ["image"],
        ["image"]
    ),
    tool!(
        "image.resize",
        Image,
        "Resize images",
        LocalOnly,
        FeatureGated,
        Some("image-core"),
        None,
        ["image"],
        ["image"]
    ),
    tool!(
        "image.compress",
        Image,
        "Compress images",
        LocalOnly,
        FeatureGated,
        Some("image-core"),
        None,
        ["image"],
        ["image"]
    ),
    tool!(
        "image.favicon",
        Image,
        "Generate favicons from SVG input",
        LocalOnly,
        FeatureGated,
        Some("image-svg"),
        None,
        ["svg"],
        ["ico", "png"]
    ),
    tool!(
        "image.watermark",
        Image,
        "Add text watermark to images",
        LocalOnly,
        DeclaredOnly,
        Some("image-core"),
        None,
        ["image"],
        ["image"]
    ),
    tool!(
        "image.filter",
        Image,
        "Apply image filters",
        LocalOnly,
        DeclaredOnly,
        Some("image-core"),
        None,
        ["image"],
        ["image"]
    ),
    tool!(
        "image.exif",
        Image,
        "Read image metadata",
        LocalOnly,
        DeclaredOnly,
        Some("image-core"),
        None,
        ["image"],
        ["metadata"]
    ),
    tool!(
        "image.qr",
        Image,
        "Generate QR codes",
        LocalOnly,
        DeclaredOnly,
        Some("image-qr"),
        None,
        ["text"],
        ["png", "svg"]
    ),
    tool!(
        "image.palette",
        Image,
        "Extract image color palette",
        LocalOnly,
        FeatureGated,
        Some("image-core"),
        None,
        ["image"],
        ["metadata"]
    ),
    tool!(
        "image.ocr",
        Image,
        "Extract text from images",
        LocalOnly,
        ExternalDependency,
        None,
        Some("tesseract"),
        ["image"],
        ["text"]
    ),
    tool!(
        "video.transcode",
        Video,
        "Transcode video format",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video"],
        ["video"]
    ),
    tool!(
        "video.extract-audio",
        Video,
        "Extract audio from video",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video"],
        ["audio"]
    ),
    tool!(
        "video.trim",
        Video,
        "Trim video by time range",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video"],
        ["video"]
    ),
    tool!(
        "video.scale",
        Video,
        "Scale video dimensions",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video"],
        ["video"]
    ),
    tool!(
        "video.to-gif",
        Video,
        "Create GIF from video",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video"],
        ["gif"]
    ),
    tool!(
        "video.thumbnail",
        Video,
        "Extract video thumbnail",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video"],
        ["image"]
    ),
    tool!(
        "video.mute",
        Video,
        "Remove video audio track",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video"],
        ["video"]
    ),
    tool!(
        "video.watermark",
        Video,
        "Add video watermark",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video"],
        ["video"]
    ),
    tool!(
        "video.speed",
        Video,
        "Change video playback speed",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video"],
        ["video"]
    ),
    tool!(
        "video.concat",
        Video,
        "Concatenate videos",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video"],
        ["video"]
    ),
    tool!(
        "video.subtitles",
        Video,
        "Burn subtitles into video",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["video", "subtitle"],
        ["video"]
    ),
    tool!(
        "audio.convert",
        Audio,
        "Convert audio format",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["audio"]
    ),
    tool!(
        "audio.trim",
        Audio,
        "Trim audio by time range",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["audio"]
    ),
    tool!(
        "audio.merge",
        Audio,
        "Merge audio files",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["audio"]
    ),
    tool!(
        "audio.normalize",
        Audio,
        "Normalize audio loudness",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["audio"]
    ),
    tool!(
        "audio.analyze-levels",
        Audio,
        "Analyze audio levels with FFmpeg volumedetect",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["metadata"]
    ),
    tool!(
        "audio.remove-silence",
        Audio,
        "Remove silence from audio",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["audio"]
    ),
    tool!(
        "audio.split",
        Audio,
        "Split audio by silence",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["audio"]
    ),
    tool!(
        "audio.effects",
        Audio,
        "Apply audio effects",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["audio"]
    ),
    tool!(
        "audio.spectrum",
        Audio,
        "Generate audio spectrum visualization",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["image", "video"]
    ),
    tool!(
        "audio.metadata",
        Audio,
        "Read audio metadata",
        LocalOnly,
        ExternalDependency,
        Some("audio-tags"),
        Some("ffprobe"),
        ["audio"],
        ["metadata"]
    ),
    tool!(
        "audio.transcribe",
        Audio,
        "Transcribe audio with a configured speech provider",
        ProviderBacked,
        RequiresCredentials,
        None,
        Some("speech-provider-credentials"),
        ["audio"],
        ["text"]
    ),
    tool!(
        "audio.generate-subtitles",
        Audio,
        "Generate subtitles with a configured speech provider",
        ProviderBacked,
        RequiresCredentials,
        None,
        Some("speech-provider-credentials"),
        ["audio"],
        ["srt", "vtt"]
    ),
    tool!(
        "audio.detect-language",
        Audio,
        "Detect spoken language with a configured speech provider",
        ProviderBacked,
        RequiresCredentials,
        None,
        Some("speech-provider-credentials"),
        ["audio"],
        ["metadata"]
    ),
    tool!(
        "audio.prepare-for-transcription",
        Audio,
        "Prepare audio as 16kHz mono WAV for speech providers",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["wav"]
    ),
    tool!(
        "audio.extract-speech-segments",
        Audio,
        "Extract likely speech segments using FFmpeg silence filtering",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ffmpeg"),
        ["audio"],
        ["audio"]
    ),
    tool!(
        "archive.zip",
        Archive,
        "Create ZIP archive",
        LocalOnly,
        FeatureGated,
        Some("archive-core"),
        None,
        ["file", "directory"],
        ["zip"]
    ),
    tool!(
        "archive.unzip",
        Archive,
        "Extract ZIP archive",
        LocalOnly,
        FeatureGated,
        Some("archive-core"),
        None,
        ["zip"],
        ["file", "directory"]
    ),
    tool!(
        "archive.tar",
        Archive,
        "Create TAR archive",
        LocalOnly,
        FeatureGated,
        Some("archive-core"),
        None,
        ["file", "directory"],
        ["tar"]
    ),
    tool!(
        "archive.untar",
        Archive,
        "Extract TAR archive",
        LocalOnly,
        FeatureGated,
        Some("archive-core"),
        None,
        ["tar"],
        ["file", "directory"]
    ),
    tool!(
        "archive.gzip",
        Archive,
        "Compress with gzip",
        LocalOnly,
        FeatureGated,
        Some("archive-core"),
        None,
        ["file"],
        ["gz"]
    ),
    tool!(
        "archive.gunzip",
        Archive,
        "Decompress gzip",
        LocalOnly,
        FeatureGated,
        Some("archive-core"),
        None,
        ["gz"],
        ["file"]
    ),
    tool!(
        "archive.list",
        Archive,
        "List archive contents",
        LocalOnly,
        FeatureGated,
        Some("archive-core"),
        None,
        ["zip"],
        ["metadata"]
    ),
    tool!(
        "document.markdown-to-html",
        Document,
        "Convert Markdown to HTML",
        LocalOnly,
        FeatureGated,
        Some("document-core"),
        None,
        ["markdown"],
        ["html"]
    ),
    tool!(
        "document.extract-text",
        Document,
        "Extract text from text/html or documents",
        LocalOnly,
        ExternalDependency,
        Some("document-core"),
        Some("pdftotext/xpdf/tika/antiword/docx2txt/libreoffice"),
        ["text", "html", "pdf", "doc"],
        ["text"]
    ),
    tool!(
        "document.pdf-merge",
        Document,
        "Merge PDF files",
        LocalOnly,
        ExternalDependency,
        None,
        Some("pdfunite/qpdf/gs"),
        ["pdf"],
        ["pdf"]
    ),
    tool!(
        "document.pdf-split",
        Document,
        "Split PDF file",
        LocalOnly,
        ExternalDependency,
        None,
        Some("qpdf/pdfseparate"),
        ["pdf"],
        ["pdf"]
    ),
    tool!(
        "document.pdf-compress",
        Document,
        "Compress PDF file",
        LocalOnly,
        ExternalDependency,
        None,
        Some("ghostscript/qpdf"),
        ["pdf"],
        ["pdf"]
    ),
    tool!(
        "document.pdf-encrypt",
        Document,
        "Encrypt PDF file",
        LocalOnly,
        ExternalDependency,
        None,
        Some("qpdf/pdftk"),
        ["pdf"],
        ["pdf"]
    ),
    tool!(
        "document.pdf-watermark",
        Document,
        "Add PDF watermark",
        LocalOnly,
        ExternalDependency,
        None,
        Some("pdftk/qpdf/gs"),
        ["pdf"],
        ["pdf"]
    ),
    tool!(
        "document.pdf-to-image",
        Document,
        "Render PDF pages to images",
        LocalOnly,
        ExternalDependency,
        None,
        Some("pdftoppm/mutool/gs"),
        ["pdf"],
        ["image"]
    ),
    tool!(
        "document.html-to-pdf",
        Document,
        "Convert HTML to PDF",
        LocalOnly,
        ExternalDependency,
        None,
        Some("wkhtmltopdf/chrome"),
        ["html"],
        ["pdf"]
    ),
    tool!(
        "utility.hash",
        Utility,
        "Calculate file hash",
        LocalOnly,
        Local,
        None,
        None,
        ["file"],
        ["metadata"]
    ),
    tool!(
        "utility.base64-encode",
        Utility,
        "Base64 encode file",
        LocalOnly,
        Local,
        None,
        None,
        ["file"],
        ["text"]
    ),
    tool!(
        "utility.base64-decode",
        Utility,
        "Base64 decode text",
        LocalOnly,
        Local,
        None,
        None,
        ["text"],
        ["file"]
    ),
    tool!(
        "utility.url-encode",
        Utility,
        "URL encode text",
        LocalOnly,
        Local,
        None,
        None,
        ["text"],
        ["text"]
    ),
    tool!(
        "utility.url-decode",
        Utility,
        "URL decode text",
        LocalOnly,
        Local,
        None,
        None,
        ["text"],
        ["text"]
    ),
    tool!(
        "utility.uuid",
        Utility,
        "Generate UUID",
        LocalOnly,
        Local,
        None,
        None,
        ["none"],
        ["text"]
    ),
    tool!(
        "utility.validate-uuid",
        Utility,
        "Validate UUID",
        LocalOnly,
        Local,
        None,
        None,
        ["text"],
        ["metadata"]
    ),
    tool!(
        "utility.timestamp",
        Utility,
        "Convert timestamp",
        LocalOnly,
        Local,
        None,
        None,
        ["timestamp"],
        ["text"]
    ),
    tool!(
        "utility.find-duplicates",
        Utility,
        "Find duplicate files",
        LocalOnly,
        Local,
        None,
        None,
        ["directory"],
        ["metadata"]
    ),
    tool!(
        "utility.verify-checksum",
        Utility,
        "Verify file checksum",
        LocalOnly,
        Local,
        None,
        None,
        ["file", "checksum"],
        ["metadata"]
    ),
    tool!(
        "utility.json-to-yaml",
        Utility,
        "Convert JSON to YAML",
        LocalOnly,
        Local,
        None,
        None,
        ["json"],
        ["yaml"]
    ),
    tool!(
        "utility.yaml-to-json",
        Utility,
        "Convert YAML to JSON",
        LocalOnly,
        Local,
        None,
        None,
        ["yaml"],
        ["json"]
    ),
    tool!(
        "utility.format-json",
        Utility,
        "Format JSON file",
        LocalOnly,
        Local,
        None,
        None,
        ["json"],
        ["json"]
    ),
    tool!(
        "utility.convert-csv",
        Utility,
        "Convert CSV data",
        LocalOnly,
        DeclaredOnly,
        Some("utility-core"),
        None,
        ["csv"],
        ["json", "csv"]
    ),
];
