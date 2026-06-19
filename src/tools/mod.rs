//! Media processing tools module.
//!
//! This module provides media acquisition and processing tools organized by category:
//! - **Image Tools**: Format conversion, resizing, compression, watermarking, etc.
//! - **Video Tools**: Transcoding, trimming, GIF creation, thumbnail extraction, etc.
//! - **Audio Tools**: Conversion, tag editing, normalization, waveform visualization, etc.
//! - **Document Tools**: PDF manipulation, Markdown conversion, CSV/JSON conversion, etc.
//! - **Archive Tools**: Compression, extraction, integrity checking, etc.
//! - **Utility Tools**: File management, hashing, encoding, clipboard operations, etc.

pub mod archive;
pub mod audio;
pub mod document;
pub(crate) mod ffmpeg;
pub(crate) mod ffprobe;
pub mod image;
pub(crate) mod receipts;
pub mod registry;
pub mod utility;
pub mod video;

// Re-export commonly used items
pub use archive::ArchiveTools;
pub use audio::AudioTools;
pub use document::DocumentTools;
pub use image::ImageTools;
pub use registry::{
    ToolDescriptor, ToolDescriptorRecord, ToolReadiness, ToolReceiptReadiness,
    ToolTypeValidationReadiness, all_tool_descriptors, tool_descriptor_records,
    tool_descriptor_records_for_category,
};
pub use utility::UtilityTools;
pub use video::VideoTools;

use std::collections::HashMap;
use std::panic::Location;
use std::path::{Path, PathBuf};

use crate::types::MediaType;
use serde::Serialize;

/// Trait for all tool operations.
pub trait Tool {
    /// Returns the name of the tool.
    fn name(&self) -> &'static str;

    /// Returns a description of the tool.
    fn description(&self) -> &'static str;

    /// Returns the category of the tool.
    fn category(&self) -> ToolCategory;
}

/// Tool categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    /// Provider-backed media acquisition tools.
    Media,
    /// Image processing tools.
    Image,
    /// Video processing tools.
    Video,
    /// Audio processing tools.
    Audio,
    /// Document processing tools.
    Document,
    /// Archive/compression tools.
    Archive,
    /// System/utility tools.
    Utility,
}

impl ToolCategory {
    /// Returns all tool categories.
    pub fn all() -> &'static [ToolCategory] {
        &[
            Self::Media,
            Self::Image,
            Self::Video,
            Self::Audio,
            Self::Document,
            Self::Archive,
            Self::Utility,
        ]
    }

    /// Returns a category from a user-facing filter value.
    pub fn from_filter(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        Self::all()
            .iter()
            .copied()
            .find(|category| category.as_str() == normalized)
    }

    /// Returns all stable category names.
    pub fn valid_names() -> Vec<&'static str> {
        Self::all().iter().map(ToolCategory::as_str).collect()
    }

    /// Returns the category name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Media => "media",
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Document => "document",
            Self::Archive => "archive",
            Self::Utility => "utility",
        }
    }
}

/// Declares where a tool's output comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSourceKind {
    /// The tool only transforms local inputs or generated local data.
    LocalOnly,
    /// The tool output is backed by an external media provider.
    ProviderBacked,
    /// The tool output came from a caller-supplied direct URL.
    DirectUrl,
    /// The tool output comes from bundled fixtures or recorded test data.
    FixtureBacked,
    /// The tool needs credentials before it can produce real provider output.
    RequiresCredentials,
}

impl ToolSourceKind {
    /// Returns the source kind as a stable metadata string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalOnly => "local-only",
            Self::ProviderBacked => "provider-backed",
            Self::DirectUrl => "direct-url",
            Self::FixtureBacked => "fixture-backed",
            Self::RequiresCredentials => "requires-credentials",
        }
    }
}

/// Receipt metadata for a tool result.
#[derive(Debug, Clone)]
pub struct ToolReceipt {
    /// Tool name or command path.
    pub tool_name: String,
    /// Output source kind.
    pub source_kind: ToolSourceKind,
    /// Provider name when the tool is provider-backed.
    pub provider: Option<String>,
    /// License or provenance statement when known.
    pub license: Option<String>,
    /// Human-readable source path, URL, or provider page when known.
    pub source: Option<String>,
    /// Required local dependency, such as ffmpeg.
    pub dependency: Option<String>,
}

impl ToolReceipt {
    /// Creates a local-only receipt for a tool.
    #[must_use]
    pub fn local(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            source_kind: ToolSourceKind::LocalOnly,
            provider: None,
            license: None,
            source: None,
            dependency: None,
        }
    }

    /// Creates a provider-backed receipt for a tool.
    #[must_use]
    pub fn provider_backed(tool_name: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            source_kind: ToolSourceKind::ProviderBacked,
            provider: Some(provider.into()),
            license: None,
            source: None,
            dependency: None,
        }
    }

    /// Creates a receipt for a caller-supplied direct URL.
    #[must_use]
    pub fn direct_url(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            source_kind: ToolSourceKind::DirectUrl,
            provider: None,
            license: None,
            source: None,
            dependency: None,
        }
    }

    /// Creates a fixture-backed receipt for a test or fixture-only tool output.
    #[must_use]
    pub fn fixture_backed(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            source_kind: ToolSourceKind::FixtureBacked,
            provider: None,
            license: None,
            source: None,
            dependency: None,
        }
    }

    /// Creates a receipt for a tool that needs credentials before it can run.
    #[must_use]
    pub fn requires_credentials(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            source_kind: ToolSourceKind::RequiresCredentials,
            provider: None,
            license: None,
            source: None,
            dependency: None,
        }
    }

    /// Records a required local dependency.
    #[must_use]
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependency = Some(dependency.into());
        self
    }

    /// Records the provider for a provider-backed receipt.
    #[must_use]
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Records the license or provenance statement when known.
    #[must_use]
    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    /// Records the source path or URL.
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// Tool operation result with detailed output.
#[derive(Debug, Clone, Serialize)]
pub struct ToolOutput {
    /// Whether the operation succeeded.
    pub success: bool,
    /// Output message.
    pub message: String,
    /// Output file path(s) if any.
    pub output_paths: Vec<PathBuf>,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

impl ToolOutput {
    /// Create a successful output.
    #[track_caller]
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            output_paths: Vec::new(),
            metadata: Self::base_receipt_metadata(true, Location::caller()),
        }
    }

    /// Create a successful output with output path.
    #[track_caller]
    pub fn success_with_path(message: impl Into<String>, path: impl AsRef<Path>) -> Self {
        Self {
            success: true,
            message: message.into(),
            output_paths: vec![path.as_ref().to_path_buf()],
            metadata: Self::base_receipt_metadata(true, Location::caller()),
        }
    }

    /// Create a failed output.
    #[track_caller]
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            output_paths: Vec::new(),
            metadata: Self::base_receipt_metadata(false, Location::caller()),
        }
    }

    /// Add a metadata entry.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add multiple metadata entries.
    #[must_use]
    pub fn with_metadata_entries(mut self, entries: HashMap<String, String>) -> Self {
        self.metadata.extend(entries);
        self
    }

    /// Add output paths.
    #[must_use]
    pub fn with_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.output_paths = paths;
        self
    }

    /// Attach receipt metadata to this output.
    #[must_use]
    pub fn with_receipt(mut self, receipt: ToolReceipt) -> Self {
        self.metadata
            .insert("tool.name".to_string(), receipt.tool_name);
        self.metadata.insert(
            "tool.source_kind".to_string(),
            receipt.source_kind.as_str().to_string(),
        );
        self.metadata.insert(
            "tool.receipt_completeness".to_string(),
            "explicit".to_string(),
        );

        if let Some(provider) = receipt.provider {
            self.metadata.insert("tool.provider".to_string(), provider);
        }
        if let Some(license) = receipt.license {
            self.metadata.insert("tool.license".to_string(), license);
        }
        if let Some(source) = receipt.source {
            self.metadata.insert("tool.source".to_string(), source);
        }
        if let Some(dependency) = receipt.dependency {
            self.metadata
                .insert("tool.dependency".to_string(), dependency);
        }

        self
    }

    /// Record a stable tool name without replacing other receipt metadata.
    #[must_use]
    pub fn with_tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.metadata
            .insert("tool.name".to_string(), tool_name.into());
        self
    }

    /// Record the output source kind.
    #[must_use]
    pub fn with_source_kind(mut self, source_kind: ToolSourceKind) -> Self {
        self.metadata.insert(
            "tool.source_kind".to_string(),
            source_kind.as_str().to_string(),
        );
        self
    }

    /// Validate an output path's extension against an expected media type.
    #[must_use]
    pub fn with_output_type_validation(
        mut self,
        path: impl AsRef<Path>,
        expected: MediaType,
    ) -> Self {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase);
        let valid = extension
            .as_deref()
            .is_some_and(|ext| expected.extensions().contains(&ext));

        self.metadata.insert(
            "tool.expected_media_type".to_string(),
            expected.as_str().to_string(),
        );
        self.metadata.insert(
            "tool.output_extension".to_string(),
            extension.unwrap_or_else(|| "unknown".to_string()),
        );
        self.metadata.insert(
            "tool.type_validation".to_string(),
            if valid { "pass" } else { "fail" }.to_string(),
        );
        if valid {
            self.metadata.remove("tool.type_validation_reason");
        } else {
            self.metadata.insert(
                "tool.type_validation_reason".to_string(),
                "extension-mismatch".to_string(),
            );
        }
        self
    }

    fn base_receipt_metadata(
        success: bool,
        caller: &'static Location<'static>,
    ) -> HashMap<String, String> {
        HashMap::from([
            (
                "tool.name".to_string(),
                Self::infer_tool_name(caller.file()),
            ),
            (
                "tool.receipt_version".to_string(),
                "dx-media-tool-receipt-v1".to_string(),
            ),
            (
                "tool.receipt_completeness".to_string(),
                "default".to_string(),
            ),
            (
                "tool.receipt_status".to_string(),
                if success { "success" } else { "failure" }.to_string(),
            ),
            (
                "tool.source_kind".to_string(),
                ToolSourceKind::LocalOnly.as_str().to_string(),
            ),
            (
                "tool.callsite".to_string(),
                format!("{}:{}", caller.file(), caller.line()),
            ),
        ])
    }

    fn infer_tool_name(file: &str) -> String {
        let normalized = file.replace('\\', "/");
        let module_path = normalized
            .rsplit_once("src/")
            .map(|(_, path)| path)
            .unwrap_or(normalized.as_str())
            .trim_end_matches(".rs")
            .replace('/', "::");

        if module_path.is_empty() {
            "unknown".to_string()
        } else {
            module_path
        }
    }
}
