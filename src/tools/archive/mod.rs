//! Archive and compression tools.
//!
//! This module provides 10 archive manipulation tools:
//! 1. Zip Creator - Create ZIP archives
//! 2. Zip Extractor - Extract ZIP archives
//! 3. Tar Creator - Create TAR archives
//! 4. Tar Extractor - Extract TAR archives
//! 5. Compressor - Compress files (gzip, bzip2, xz)
//! 6. Decompressor - Decompress files
//! 7. Archive List - List archive contents
//! 8. Archive Encrypt - Encrypt archives
//! 9. Archive Split - Split large archives
//! 10. Archive Merge - Merge split archives
//!
//! ## Native Processing
//!
//! Enable the `archive-core` feature for native Rust archive processing
//! without external tools. Uses `zip`, `tar`, and `flate2` crates.

pub mod compress;
pub mod decompress;
pub mod encrypt;
pub mod list;
pub mod merge;
pub mod native;
pub mod rar;
pub mod sevenz;
pub mod split;
pub mod tar;
pub mod zip;

pub use compress::*;
pub use decompress::*;
pub use encrypt::*;
pub use list::*;
pub use merge::*;
pub use native::*;
pub use rar::*;
pub use sevenz::*;
pub use split::*;
pub use tar::*;
pub use zip::*;

use crate::error::{DxError, Result};
use std::path::Path;

fn native_archive_error(path: &Path, action: &str, error: std::io::Error) -> DxError {
    DxError::FileIo {
        path: path.to_path_buf(),
        message: format!("{action}: {error}"),
        source: Some(error),
    }
}

/// Archive tools collection.
pub struct ArchiveTools;

impl ArchiveTools {
    /// Create a new ArchiveTools instance.
    pub fn new() -> Self {
        Self
    }

    /// Create ZIP archive.
    pub fn create_zip<P: AsRef<Path>>(&self, inputs: &[P], output: P) -> Result<super::ToolOutput> {
        let output = output.as_ref();
        native::create_zip_native(inputs, output, None).map_err(|error| {
            native_archive_error(output, "Failed to create native ZIP archive", error)
        })
    }

    /// Extract ZIP archive.
    pub fn extract_zip<P: AsRef<Path>>(
        &self,
        input: P,
        output_dir: P,
    ) -> Result<super::ToolOutput> {
        let input = input.as_ref();
        let output_dir = output_dir.as_ref();
        native::extract_zip_native(input, output_dir).map_err(|error| {
            native_archive_error(input, "Failed to extract native ZIP archive", error)
        })
    }

    /// Create TAR archive.
    pub fn create_tar<P: AsRef<Path>>(&self, inputs: &[P], output: P) -> Result<super::ToolOutput> {
        tar::create_tar(inputs, output)
    }

    /// Extract TAR archive.
    pub fn extract_tar<P: AsRef<Path>>(
        &self,
        input: P,
        output_dir: P,
    ) -> Result<super::ToolOutput> {
        tar::extract_tar(input, output_dir)
    }

    /// Compress file with gzip.
    pub fn gzip<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        compress::gzip(input, output)
    }

    /// Decompress gzip file.
    pub fn gunzip<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        decompress::gunzip(input, output)
    }

    /// List archive contents.
    pub fn list<P: AsRef<Path>>(&self, input: P) -> Result<super::ToolOutput> {
        let input = input.as_ref();
        let extension = input
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default();

        if extension.eq_ignore_ascii_case("zip") {
            return native::list_zip_native(input)
                .map(|output| {
                    output
                        .with_tool_name("archive.list")
                        .with_metadata("tool.implementation", "archive.list-zip.native")
                })
                .map_err(|error| {
                    native_archive_error(input, "Failed to list native ZIP archive", error)
                });
        }

        list::list_archive(input)
    }

    /// Create encrypted archive.
    pub fn encrypt_archive<P: AsRef<Path>>(
        &self,
        inputs: &[P],
        output: P,
        password: &str,
    ) -> Result<super::ToolOutput> {
        encrypt::create_encrypted_zip(inputs, output, password)
    }

    /// Split archive into parts.
    pub fn split_archive<P: AsRef<Path>>(
        &self,
        input: P,
        output_dir: P,
        part_size_mb: u64,
    ) -> Result<super::ToolOutput> {
        split::split_archive(input, output_dir, part_size_mb)
    }

    /// Merge split archives.
    pub fn merge_archives<P: AsRef<Path>>(
        &self,
        parts: &[P],
        output: P,
    ) -> Result<super::ToolOutput> {
        merge::merge_archives(parts, output)
    }
}

impl Default for ArchiveTools {
    fn default() -> Self {
        Self::new()
    }
}
