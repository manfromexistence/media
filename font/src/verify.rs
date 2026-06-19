//! File verification for dx-font
//!
//! This module provides verification functionality for downloaded font files,
//! including magic byte validation, zip archive verification, and file integrity checks.

use crate::error::{FontError, FontResult};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::{debug, warn};

// Magic bytes for font formats
/// ZIP archive magic bytes (PK\x03\x04)
pub const ZIP_MAGIC: &[u8] = &[0x50, 0x4B, 0x03, 0x04];
/// TrueType font magic bytes
pub const TTF_MAGIC: &[u8] = &[0x00, 0x01, 0x00, 0x00];
/// OpenType font magic bytes ("OTTO")
pub const OTF_MAGIC: &[u8] = &[0x4F, 0x54, 0x54, 0x4F];
/// WOFF font magic bytes ("wOFF")
pub const WOFF_MAGIC: &[u8] = &[0x77, 0x4F, 0x46, 0x46];
/// WOFF2 font magic bytes ("wOF2")
pub const WOFF2_MAGIC: &[u8] = &[0x77, 0x4F, 0x46, 0x32];
/// TrueType Collection magic bytes ("ttcf")
pub const TTC_MAGIC: &[u8] = &[0x74, 0x74, 0x63, 0x66];

/// File verifier for downloaded font files
#[derive(Debug, Clone, Default)]
pub struct FileVerifier;

impl FileVerifier {
    /// Create a new file verifier
    pub fn new() -> Self {
        Self
    }

    /// Verify a downloaded file is valid
    ///
    /// # Arguments
    /// * `path` - Path to the file to verify
    /// * `expected_format` - Expected file format (e.g., "ttf", "otf", "woff", "woff2", "zip")
    ///
    /// # Returns
    /// * `Ok(())` - File is valid
    /// * `Err(FontError::Verification)` - File is invalid
    pub fn verify(path: &Path, expected_format: &str) -> FontResult<()> {
        Self::verify_not_empty(path)?;
        Self::verify_magic_bytes(path, expected_format)?;

        // Additional verification for zip files
        if expected_format == "zip" {
            Self::verify_zip(path)?;
        }

        debug!("File verification passed for {}", path.display());
        Ok(())
    }

    /// Verify that a file is not empty
    ///
    /// # Arguments
    /// * `path` - Path to the file to verify
    pub fn verify_not_empty(path: &Path) -> FontResult<()> {
        let metadata = std::fs::metadata(path).map_err(|e| {
            FontError::verification(format!(
                "Failed to read file metadata for {}: {}",
                path.display(),
                e
            ))
        })?;

        if metadata.len() == 0 {
            return Err(FontError::verification(format!(
                "File is empty: {}",
                path.display()
            )));
        }

        debug!(
            "File {} is not empty ({} bytes)",
            path.display(),
            metadata.len()
        );
        Ok(())
    }

    /// Verify that a file has the expected magic bytes
    ///
    /// # Arguments
    /// * `path` - Path to the file to verify
    /// * `format` - Expected file format
    pub fn verify_magic_bytes(path: &Path, format: &str) -> FontResult<()> {
        let mut file = File::open(path).map_err(|e| {
            FontError::verification(format!("Failed to open file {}: {}", path.display(), e))
        })?;

        let mut magic = [0u8; 4];
        let bytes_read = file.read(&mut magic).map_err(|e| {
            FontError::verification(format!(
                "Failed to read magic bytes from {}: {}",
                path.display(),
                e
            ))
        })?;

        if bytes_read < 4 {
            return Err(FontError::verification(format!(
                "File too small to verify: {} (only {} bytes)",
                path.display(),
                bytes_read
            )));
        }

        let format_lower = format.to_lowercase();
        match format_lower.as_str() {
            "zip" => {
                if magic != ZIP_MAGIC {
                    return Err(FontError::verification(format!(
                        "Invalid ZIP magic bytes in {}: expected {:02X?}, got {:02X?}",
                        path.display(),
                        ZIP_MAGIC,
                        magic
                    )));
                }
            }
            "ttf" => {
                // TTF can have either TTF_MAGIC or TTC_MAGIC (for collections)
                if magic != TTF_MAGIC && magic != TTC_MAGIC {
                    return Err(FontError::verification(format!(
                        "Invalid TTF magic bytes in {}: expected {:02X?} or {:02X?}, got {:02X?}",
                        path.display(),
                        TTF_MAGIC,
                        TTC_MAGIC,
                        magic
                    )));
                }
            }
            "otf" => {
                // OTF can have OTF_MAGIC or TTF_MAGIC (some OTF files use TTF structure)
                if magic != OTF_MAGIC && magic != TTF_MAGIC {
                    return Err(FontError::verification(format!(
                        "Invalid OTF magic bytes in {}: expected {:02X?} or {:02X?}, got {:02X?}",
                        path.display(),
                        OTF_MAGIC,
                        TTF_MAGIC,
                        magic
                    )));
                }
            }
            "woff" => {
                if magic != WOFF_MAGIC {
                    return Err(FontError::verification(format!(
                        "Invalid WOFF magic bytes in {}: expected {:02X?}, got {:02X?}",
                        path.display(),
                        WOFF_MAGIC,
                        magic
                    )));
                }
            }
            "woff2" => {
                if magic != WOFF2_MAGIC {
                    return Err(FontError::verification(format!(
                        "Invalid WOFF2 magic bytes in {}: expected {:02X?}, got {:02X?}",
                        path.display(),
                        WOFF2_MAGIC,
                        magic
                    )));
                }
            }
            _ => {
                // Unknown format, skip magic byte verification
                debug!(
                    "Unknown format '{}', skipping magic byte verification for {}",
                    format,
                    path.display()
                );
            }
        }

        debug!(
            "Magic bytes verified for {} (format: {})",
            path.display(),
            format
        );
        Ok(())
    }

    /// Verify that a file is a valid ZIP archive
    ///
    /// # Arguments
    /// * `path` - Path to the ZIP file to verify
    pub fn verify_zip(path: &Path) -> FontResult<()> {
        let file = File::open(path).map_err(|e| {
            FontError::verification(format!("Failed to open ZIP file {}: {}", path.display(), e))
        })?;

        let archive = zip::ZipArchive::new(file).map_err(|e| {
            FontError::verification(format!("Invalid ZIP archive {}: {}", path.display(), e))
        })?;

        if archive.is_empty() {
            return Err(FontError::verification(format!(
                "ZIP archive is empty: {}",
                path.display()
            )));
        }

        debug!(
            "ZIP archive verified for {} ({} entries)",
            path.display(),
            archive.len()
        );
        Ok(())
    }

    /// Verify a file and clean up on failure
    ///
    /// If verification fails, the file is deleted and an error is returned.
    ///
    /// # Arguments
    /// * `path` - Path to the file to verify
    /// * `expected_format` - Expected file format
    pub fn verify_and_cleanup(path: &Path, expected_format: &str) -> FontResult<()> {
        match Self::verify(path, expected_format) {
            Ok(()) => Ok(()),
            Err(e) => {
                warn!(
                    "Verification failed for {}, cleaning up: {}",
                    path.display(),
                    e
                );
                if let Err(cleanup_err) = std::fs::remove_file(path) {
                    warn!(
                        "Failed to clean up invalid file {}: {}",
                        path.display(),
                        cleanup_err
                    );
                }
                Err(e)
            }
        }
    }

    /// Get the expected format from a file extension
    pub fn format_from_extension(path: &Path) -> Option<&str> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "ttf" => "ttf",
                "otf" => "otf",
                "woff" => "woff",
                "woff2" => "woff2",
                "zip" => "zip",
                _ => "unknown",
            })
    }

    /// Detect the format of a file based on its magic bytes
    pub fn detect_format(path: &Path) -> FontResult<String> {
        let mut file = File::open(path).map_err(|e| {
            FontError::verification(format!("Failed to open file {}: {}", path.display(), e))
        })?;

        let mut magic = [0u8; 4];
        let bytes_read = file.read(&mut magic).map_err(|e| {
            FontError::verification(format!(
                "Failed to read magic bytes from {}: {}",
                path.display(),
                e
            ))
        })?;

        if bytes_read < 4 {
            return Err(FontError::verification(format!(
                "File too small to detect format: {}",
                path.display()
            )));
        }

        let format = if magic == ZIP_MAGIC {
            "zip"
        } else if magic == TTF_MAGIC || magic == TTC_MAGIC {
            "ttf"
        } else if magic == OTF_MAGIC {
            "otf"
        } else if magic == WOFF_MAGIC {
            "woff"
        } else if magic == WOFF2_MAGIC {
            "woff2"
        } else {
            "unknown"
        };

        Ok(format.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_verify_not_empty_success() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.txt", b"hello");
        assert!(FileVerifier::verify_not_empty(&path).is_ok());
    }

    #[test]
    fn test_verify_not_empty_failure() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "empty.txt", b"");
        let result = FileVerifier::verify_not_empty(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_verify_magic_bytes_zip() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.zip", &[0x50, 0x4B, 0x03, 0x04, 0x00]);
        assert!(FileVerifier::verify_magic_bytes(&path, "zip").is_ok());
    }

    #[test]
    fn test_verify_magic_bytes_ttf() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.ttf", &[0x00, 0x01, 0x00, 0x00, 0x00]);
        assert!(FileVerifier::verify_magic_bytes(&path, "ttf").is_ok());
    }

    #[test]
    fn test_verify_magic_bytes_otf() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.otf", &[0x4F, 0x54, 0x54, 0x4F, 0x00]);
        assert!(FileVerifier::verify_magic_bytes(&path, "otf").is_ok());
    }

    #[test]
    fn test_verify_magic_bytes_woff() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.woff", &[0x77, 0x4F, 0x46, 0x46, 0x00]);
        assert!(FileVerifier::verify_magic_bytes(&path, "woff").is_ok());
    }

    #[test]
    fn test_verify_magic_bytes_woff2() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.woff2", &[0x77, 0x4F, 0x46, 0x32, 0x00]);
        assert!(FileVerifier::verify_magic_bytes(&path, "woff2").is_ok());
    }

    #[test]
    fn test_verify_magic_bytes_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.ttf", &[0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
        let result = FileVerifier::verify_magic_bytes(&path, "ttf");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid"));
    }

    #[test]
    fn test_verify_magic_bytes_unknown_format() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.xyz", &[0x00, 0x00, 0x00, 0x00, 0x00]);
        // Unknown formats should pass (no verification)
        assert!(FileVerifier::verify_magic_bytes(&path, "xyz").is_ok());
    }

    #[test]
    fn test_verify_magic_bytes_file_too_small() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "small.ttf", &[0x00, 0x01]);
        let result = FileVerifier::verify_magic_bytes(&path, "ttf");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }

    #[test]
    fn test_verify_and_cleanup_success() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.woff", &[0x77, 0x4F, 0x46, 0x46, 0x00]);
        assert!(FileVerifier::verify_and_cleanup(&path, "woff").is_ok());
        assert!(path.exists()); // File should still exist
    }

    #[test]
    fn test_verify_and_cleanup_failure() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.woff", &[0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
        let result = FileVerifier::verify_and_cleanup(&path, "woff");
        assert!(result.is_err());
        assert!(!path.exists()); // File should be deleted
    }

    #[test]
    fn test_format_from_extension() {
        assert_eq!(
            FileVerifier::format_from_extension(Path::new("font.ttf")),
            Some("ttf")
        );
        assert_eq!(
            FileVerifier::format_from_extension(Path::new("font.OTF")),
            Some("otf")
        );
        assert_eq!(
            FileVerifier::format_from_extension(Path::new("font.woff2")),
            Some("woff2")
        );
        assert_eq!(
            FileVerifier::format_from_extension(Path::new("font.xyz")),
            Some("unknown")
        );
        assert_eq!(
            FileVerifier::format_from_extension(Path::new("noextension")),
            None
        );
    }

    #[test]
    fn test_detect_format() {
        let temp_dir = TempDir::new().unwrap();

        let ttf_path = create_test_file(&temp_dir, "test.ttf", &[0x00, 0x01, 0x00, 0x00, 0x00]);
        assert_eq!(FileVerifier::detect_format(&ttf_path).unwrap(), "ttf");

        let otf_path = create_test_file(&temp_dir, "test.otf", &[0x4F, 0x54, 0x54, 0x4F, 0x00]);
        assert_eq!(FileVerifier::detect_format(&otf_path).unwrap(), "otf");

        let woff_path = create_test_file(&temp_dir, "test.woff", &[0x77, 0x4F, 0x46, 0x46, 0x00]);
        assert_eq!(FileVerifier::detect_format(&woff_path).unwrap(), "woff");

        let woff2_path = create_test_file(&temp_dir, "test.woff2", &[0x77, 0x4F, 0x46, 0x32, 0x00]);
        assert_eq!(FileVerifier::detect_format(&woff2_path).unwrap(), "woff2");

        let zip_path = create_test_file(&temp_dir, "test.zip", &[0x50, 0x4B, 0x03, 0x04, 0x00]);
        assert_eq!(FileVerifier::detect_format(&zip_path).unwrap(), "zip");

        let unknown_path = create_test_file(&temp_dir, "test.bin", &[0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
        assert_eq!(
            FileVerifier::detect_format(&unknown_path).unwrap(),
            "unknown"
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::io::Write;
    use tempfile::TempDir;

    // Feature: dx-font-production-ready, Property 8: File Verification Correctness
    // **Validates: Requirements 12.1, 12.2, 12.3**
    //
    // For any downloaded file:
    // - Empty files (0 bytes) are rejected
    // - Files with incorrect magic bytes for their declared format are rejected
    // - Invalid zip archives are rejected
    // - Valid files pass verification

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn empty_files_are_rejected(
            format in prop_oneof![
                Just("ttf"),
                Just("otf"),
                Just("woff"),
                Just("woff2"),
                Just("zip")
            ]
        ) {
            let temp_dir = TempDir::new().unwrap();
            let path = temp_dir.path().join(format!("test.{}", format));
            File::create(&path).unwrap(); // Create empty file

            let result = FileVerifier::verify(&path, format);
            prop_assert!(
                result.is_err(),
                "Empty file should be rejected for format {}",
                format
            );
            prop_assert!(
                result.unwrap_err().to_string().contains("empty"),
                "Error should mention 'empty'"
            );
        }

        #[test]
        fn incorrect_magic_bytes_are_rejected(
            format in prop_oneof![
                Just("ttf"),
                Just("otf"),
                Just("woff"),
                Just("woff2"),
                Just("zip")
            ],
            wrong_magic in prop::collection::vec(any::<u8>(), 4..10)
        ) {
            // Skip if the random bytes happen to match the correct magic
            let correct_magic = match format {
                "ttf" => TTF_MAGIC,
                "otf" => OTF_MAGIC,
                "woff" => WOFF_MAGIC,
                "woff2" => WOFF2_MAGIC,
                "zip" => ZIP_MAGIC,
                _ => &[],
            };

            // Also check for alternative valid magic bytes
            let alt_magic: &[u8] = match format {
                "ttf" => TTC_MAGIC,
                "otf" => TTF_MAGIC, // OTF can also use TTF magic
                _ => &[],
            };

            if wrong_magic.len() >= 4 {
                let magic_slice = &wrong_magic[0..4];
                if magic_slice == correct_magic || magic_slice == alt_magic {
                    return Ok(()); // Skip this case
                }
            }

            let temp_dir = TempDir::new().unwrap();
            let path = temp_dir.path().join(format!("test.{}", format));
            let mut file = File::create(&path).unwrap();
            file.write_all(&wrong_magic).unwrap();

            let result = FileVerifier::verify_magic_bytes(&path, format);
            prop_assert!(
                result.is_err(),
                "Incorrect magic bytes should be rejected for format {}: {:02X?}",
                format,
                &wrong_magic[0..4.min(wrong_magic.len())]
            );
        }

        #[test]
        fn valid_magic_bytes_are_accepted(
            format in prop_oneof![
                Just("ttf"),
                Just("otf"),
                Just("woff"),
                Just("woff2"),
                Just("zip")
            ],
            extra_bytes in prop::collection::vec(any::<u8>(), 0..100)
        ) {
            let magic = match format {
                "ttf" => TTF_MAGIC,
                "otf" => OTF_MAGIC,
                "woff" => WOFF_MAGIC,
                "woff2" => WOFF2_MAGIC,
                "zip" => ZIP_MAGIC,
                _ => &[],
            };

            let temp_dir = TempDir::new().unwrap();
            let path = temp_dir.path().join(format!("test.{}", format));
            let mut file = File::create(&path).unwrap();
            file.write_all(magic).unwrap();
            file.write_all(&extra_bytes).unwrap();

            let result = FileVerifier::verify_magic_bytes(&path, format);
            prop_assert!(
                result.is_ok(),
                "Valid magic bytes should be accepted for format {}: {:?}",
                format,
                result.err()
            );
        }

        #[test]
        fn unknown_formats_pass_verification(
            format in "[a-z]{3,5}",
            content in prop::collection::vec(any::<u8>(), 4..100)
        ) {
            // Skip known formats
            if ["ttf", "otf", "woff", "woff2", "zip"].contains(&format.as_str()) {
                return Ok(());
            }

            let temp_dir = TempDir::new().unwrap();
            let path = temp_dir.path().join(format!("test.{}", format));
            let mut file = File::create(&path).unwrap();
            file.write_all(&content).unwrap();

            let result = FileVerifier::verify_magic_bytes(&path, &format);
            prop_assert!(
                result.is_ok(),
                "Unknown format '{}' should pass magic byte verification",
                format
            );
        }
    }

    // Feature: dx-font-production-ready, Property 11: Verification Failure Cleanup
    // **Validates: Requirements 12.4**
    //
    // For any download where file verification fails, the partially downloaded file
    // SHALL be deleted from disk, and an error SHALL be returned.

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn verification_failure_cleans_up_file(
            format in prop_oneof![
                Just("ttf"),
                Just("otf"),
                Just("woff"),
                Just("woff2")
            ],
            invalid_content in prop::collection::vec(0xFFu8..=0xFFu8, 4..50)
        ) {
            let temp_dir = TempDir::new().unwrap();
            let path = temp_dir.path().join(format!("test.{}", format));

            // Create file with invalid content
            let mut file = File::create(&path).unwrap();
            file.write_all(&invalid_content).unwrap();
            drop(file);

            // Verify file exists before cleanup
            prop_assert!(path.exists(), "File should exist before verification");

            // Attempt verification with cleanup
            let result = FileVerifier::verify_and_cleanup(&path, format);

            // Verification should fail
            prop_assert!(
                result.is_err(),
                "Verification should fail for invalid content"
            );

            // File should be deleted
            prop_assert!(
                !path.exists(),
                "File should be deleted after verification failure"
            );
        }

        #[test]
        fn verification_success_keeps_file(
            format in prop_oneof![
                Just("ttf"),
                Just("otf"),
                Just("woff"),
                Just("woff2")
            ],
            extra_bytes in prop::collection::vec(any::<u8>(), 0..50)
        ) {
            let magic = match format {
                "ttf" => TTF_MAGIC,
                "otf" => OTF_MAGIC,
                "woff" => WOFF_MAGIC,
                "woff2" => WOFF2_MAGIC,
                _ => &[],
            };

            let temp_dir = TempDir::new().unwrap();
            let path = temp_dir.path().join(format!("test.{}", format));

            // Create file with valid magic bytes
            let mut file = File::create(&path).unwrap();
            file.write_all(magic).unwrap();
            file.write_all(&extra_bytes).unwrap();
            drop(file);

            // Verify file exists before verification
            prop_assert!(path.exists(), "File should exist before verification");

            // Attempt verification with cleanup
            let result = FileVerifier::verify_and_cleanup(&path, format);

            // Verification should succeed
            prop_assert!(
                result.is_ok(),
                "Verification should succeed for valid content: {:?}",
                result.err()
            );

            // File should still exist
            prop_assert!(
                path.exists(),
                "File should still exist after successful verification"
            );
        }
    }
}
