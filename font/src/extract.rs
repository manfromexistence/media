//! ZIP extraction utilities for font archives
//!
//! Provides safe and efficient extraction of ZIP archives containing fonts.

use crate::error::{FontError, FontResult};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, instrument};
use zip::ZipArchive;

/// Extract a ZIP archive to a target directory
///
/// # Arguments
/// * `zip_path` - Path to the ZIP file
/// * `target_dir` - Directory to extract files into
///
/// # Returns
/// * `Ok(())` if extraction succeeds
/// * `Err(FontError)` if extraction fails
#[instrument(skip_all, fields(zip_path = %zip_path.display(), target_dir = %target_dir.display()))]
pub fn extract_zip(zip_path: &Path, target_dir: &Path) -> FontResult<()> {
    debug!("Opening ZIP archive");

    let file = File::open(zip_path)
        .map_err(|e| FontError::verification(format!("Failed to open ZIP file: {}", e)))?;

    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| FontError::verification(format!("Failed to read ZIP archive: {}", e)))?;

    debug!(file_count = archive.len(), "Extracting ZIP archive");

    // Use the built-in extract method which handles all edge cases
    archive
        .extract(target_dir)
        .map_err(|e| FontError::verification(format!("Failed to extract ZIP archive: {}", e)))?;

    debug!("ZIP extraction completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    use zip::CompressionMethod;
    use zip::write::{FileOptions, ZipWriter};

    fn create_test_zip(path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut zip = ZipWriter::new(file);

        let options: FileOptions<()> =
            FileOptions::default().compression_method(CompressionMethod::Deflated);

        // Add a test font file
        zip.start_file("test-font.ttf", options)?;
        zip.write_all(b"fake font data")?;

        // Add a nested file
        zip.start_file("fonts/nested-font.woff2", options)?;
        zip.write_all(b"fake woff2 data")?;

        zip.finish()?;
        Ok(())
    }

    #[test]
    fn test_extract_zip() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("test.zip");
        let extract_dir = temp_dir.path().join("extracted");

        // Create test ZIP
        create_test_zip(&zip_path).unwrap();

        // Extract it
        extract_zip(&zip_path, &extract_dir).unwrap();

        // Verify extraction
        assert!(extract_dir.join("test-font.ttf").exists());
        assert!(extract_dir.join("fonts/nested-font.woff2").exists());
    }

    #[test]
    fn test_extract_nonexistent_zip() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("nonexistent.zip");
        let extract_dir = temp_dir.path().join("extracted");

        let result = extract_zip(&zip_path, &extract_dir);
        assert!(result.is_err());
    }
}
