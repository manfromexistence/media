//! Utility tools.
//!
//! This module provides 13 general utility tools:
//! 1. Hash Calculator - Calculate file checksums
//! 2. Base64 Encoder/Decoder - Encode/decode base64
//! 3. URL Encoder/Decoder - Encode/decode URLs
//! 4. JSON Formatter - Format and validate JSON
//! 5. YAML Converter - Convert between JSON/YAML
//! 6. CSV Converter - Convert CSV to other formats
//! 7. Diff Tool - Compare files
//! 8. UUID Generator - Generate unique IDs
//! 9. Timestamp Tool - Convert timestamps
//! 10. Random Generator - Generate random data
//! 11. Duplicate Finder - Find duplicate files by content hash (NEW)
//! 12. File Watcher - Watch directories for changes (NEW)
//! 13. Checksum Tool - Multi-algorithm checksum verification (NEW)

pub mod base64;
pub mod checksum;
pub mod csv_convert;
pub mod diff;
pub mod duplicate;
pub mod hash;
pub mod json_format;
pub mod random;
pub mod timestamp;
pub mod url_encode;
pub mod uuid;
pub mod watcher;
pub mod yaml_convert;

pub use base64::*;
pub use checksum::*;
pub use csv_convert::*;
pub use diff::*;
pub use duplicate::*;
pub use hash::*;
pub use json_format::*;
pub use random::*;
pub use timestamp::*;
pub use url_encode::*;
pub use uuid::*;
pub use watcher::*;
pub use yaml_convert::*;

use crate::error::Result;
use std::path::Path;

/// Utility tools collection.
pub struct UtilityTools;

impl UtilityTools {
    /// Create a new UtilityTools instance.
    pub fn new() -> Self {
        Self
    }

    /// Calculate file hash.
    pub fn hash_file<P: AsRef<Path>>(
        &self,
        input: P,
        algorithm: hash::HashAlgorithm,
    ) -> Result<super::ToolOutput> {
        hash::hash_file(input, algorithm)
    }

    /// Encode to base64.
    pub fn base64_encode<P: AsRef<Path>>(&self, input: P) -> Result<super::ToolOutput> {
        base64::encode_file(input)
    }

    /// Decode from base64.
    pub fn base64_decode<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        base64::decode_file(input, output)
    }

    /// URL encode string.
    pub fn url_encode(&self, input: &str) -> Result<super::ToolOutput> {
        url_encode::encode(input)
    }

    /// URL decode string.
    pub fn url_decode(&self, input: &str) -> Result<super::ToolOutput> {
        url_encode::decode(input)
    }

    /// Format JSON.
    pub fn format_json<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        json_format::format_json_file(input, output)
    }

    /// Convert JSON to YAML.
    pub fn json_to_yaml<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        yaml_convert::json_to_yaml(input, output)
    }

    /// Convert YAML to JSON.
    pub fn yaml_to_json<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        yaml_convert::yaml_to_json(input, output)
    }

    /// Compare two files.
    pub fn diff_files<P: AsRef<Path>>(&self, file1: P, file2: P) -> Result<super::ToolOutput> {
        diff::diff_files(file1, file2)
    }

    /// Generate UUID.
    pub fn generate_uuid(&self) -> Result<super::ToolOutput> {
        let uuid_str = uuid::generate_v4();
        Ok(super::ToolOutput::success(uuid_str))
    }

    /// Get current timestamp.
    pub fn timestamp(&self) -> Result<super::ToolOutput> {
        timestamp::now(timestamp::TimestampFormat::default())
    }

    /// Generate random string.
    pub fn random_string(&self, length: usize) -> Result<super::ToolOutput> {
        random::string(length, random::CharSet::default())
    }

    /// CSV to JSON conversion.
    pub fn csv_to_json<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        csv_convert::csv_to_json(input, output)
    }
}

impl Default for UtilityTools {
    fn default() -> Self {
        Self::new()
    }
}
