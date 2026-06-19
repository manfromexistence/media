//! Archive listing utilities.
//!
//! List contents of various archive formats.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Archive entry information.
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    /// Entry name/path.
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// Compressed size.
    pub compressed_size: Option<u64>,
    /// Is directory.
    pub is_dir: bool,
    /// Modification time.
    pub modified: Option<String>,
}

/// List archive contents.
///
/// # Arguments
/// * `input` - Archive file path
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::list;
///
/// let result = list::list_archive("archive.zip").unwrap();
/// println!("{}", result.message);
/// ```
pub fn list_archive<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    // Detect archive type
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let name = input_path.to_string_lossy().to_lowercase();

    if ext == "zip" || ext == "zipx" {
        return list_zip(input_path);
    }

    if name.ends_with(".tar.gz")
        || name.ends_with(".tgz")
        || name.ends_with(".tar.bz2")
        || name.ends_with(".tbz2")
        || name.ends_with(".tar.xz")
        || name.ends_with(".txz")
        || name.ends_with(".tar.zst")
        || ext == "tar"
    {
        return list_tar(input_path);
    }

    if ext == "7z" {
        return list_7z(input_path);
    }

    if ext == "rar" {
        return list_rar(input_path);
    }

    // Try 7z as fallback (supports many formats)
    list_7z(input_path)
}

/// List ZIP archive contents.
fn list_zip(input: &Path) -> Result<ToolOutput> {
    // Try unzip -l
    let mut cmd = Command::new("unzip");
    cmd.arg("-l").arg(input);

    if let Ok(result) = cmd.output() {
        if result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout);
            let file_count = count_files_in_listing(&output);

            return Ok(ToolOutput::success(output.to_string())
                .with_metadata("format", "zip".to_string())
                .with_metadata("file_count", file_count.to_string()));
        }
    }

    // Try 7z as fallback
    list_7z(input)
}

/// List TAR archive contents.
fn list_tar(input: &Path) -> Result<ToolOutput> {
    let name = input.to_string_lossy().to_lowercase();

    let mut cmd = Command::new("tar");
    cmd.arg("-t").arg("-v");

    // Auto-detect compression
    if name.ends_with(".gz") || name.ends_with(".tgz") {
        cmd.arg("-z");
    } else if name.ends_with(".bz2") || name.ends_with(".tbz2") {
        cmd.arg("-j");
    } else if name.ends_with(".xz") || name.ends_with(".txz") {
        cmd.arg("-J");
    } else if name.ends_with(".zst") {
        cmd.arg("--zstd");
    }

    cmd.arg("-f").arg(input);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run tar: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Failed to list archive".to_string(),
            source: None,
        });
    }

    let output = String::from_utf8_lossy(&result.stdout);
    let file_count = output.lines().count();

    Ok(ToolOutput::success(output.to_string())
        .with_metadata("format", "tar".to_string())
        .with_metadata("file_count", file_count.to_string()))
}

/// List 7z archive contents.
fn list_7z(input: &Path) -> Result<ToolOutput> {
    let mut cmd = Command::new("7z");
    cmd.arg("l").arg(input);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Failed to list archive with 7z".to_string(),
            source: None,
        });
    }

    let output = String::from_utf8_lossy(&result.stdout);
    let file_count = count_files_in_7z_listing(&output);

    Ok(ToolOutput::success(output.to_string())
        .with_metadata("format", "7z".to_string())
        .with_metadata("file_count", file_count.to_string()))
}

/// List RAR archive contents.
fn list_rar(input: &Path) -> Result<ToolOutput> {
    // Try unrar
    let mut cmd = Command::new("unrar");
    cmd.arg("l").arg(input);

    if let Ok(result) = cmd.output() {
        if result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout);
            let file_count = count_files_in_listing(&output);

            return Ok(ToolOutput::success(output.to_string())
                .with_metadata("format", "rar".to_string())
                .with_metadata("file_count", file_count.to_string()));
        }
    }

    // Try 7z as fallback
    list_7z(input)
}

/// Count files in generic listing.
fn count_files_in_listing(listing: &str) -> usize {
    // Rough estimate based on non-empty lines
    listing
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter(|l| !l.starts_with('-'))
        .filter(|l| !l.contains("Archive:"))
        .filter(|l| !l.contains("files"))
        .count()
        .saturating_sub(2) // Header lines
}

/// Count files in 7z listing.
fn count_files_in_7z_listing(listing: &str) -> usize {
    // Look for "X files" in output
    for line in listing.lines() {
        if line.contains(" files") {
            if let Some(num) = line.split_whitespace().next() {
                if let Ok(count) = num.parse::<usize>() {
                    return count;
                }
            }
        }
    }

    // Fallback to line counting
    count_files_in_listing(listing)
}

/// Get archive info summary.
pub fn get_archive_info<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    let file_size = std::fs::metadata(input_path).map_or(0, |m| m.len());

    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown");

    let mut output = ToolOutput::success(format!(
        "Archive: {}\nSize: {} bytes\nFormat: {}",
        input_path.display(),
        file_size,
        ext
    ))
    .with_metadata("size", file_size.to_string())
    .with_metadata("format", ext.to_string());

    // Try to get detailed info
    if let Ok(list_result) = list_archive(input_path) {
        if let Some(count) = list_result.metadata.get("file_count") {
            output = output.with_metadata("file_count", count.clone());
        }
    }

    Ok(output)
}

/// List only specific file types.
pub fn list_filtered<P: AsRef<Path>>(input: P, extensions: &[&str]) -> Result<ToolOutput> {
    let result = list_archive(input)?;

    let filtered: Vec<&str> = result
        .message
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            extensions
                .iter()
                .any(|ext| lower.ends_with(&format!(".{}", ext.to_lowercase())))
        })
        .collect();

    Ok(ToolOutput::success(filtered.join("\n"))
        .with_metadata("filtered_count", filtered.len().to_string()))
}

/// List directories only.
pub fn list_directories<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let result = list_archive(input)?;

    let dirs: Vec<&str> = result
        .message
        .lines()
        .filter(|line| line.ends_with('/') || line.contains(" d") || line.contains("<DIR>"))
        .collect();

    Ok(ToolOutput::success(dirs.join("\n")).with_metadata("dir_count", dirs.len().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_files() {
        let listing = "Archive: test.zip\n  Length      Date    Time    Name\n    123  01-01-2024 12:00   file.txt\n    456  01-01-2024 12:00   dir/other.txt\n  2 files";
        let count = count_files_in_listing(listing);
        assert!(count > 0);
    }
}
