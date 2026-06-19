//! Archive splitting utilities.
//!
//! Split large archives into smaller parts.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Split archive into parts.
///
/// # Arguments
/// * `input` - Archive to split
/// * `output_dir` - Directory for split parts
/// * `part_size_mb` - Size of each part in megabytes
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::split;
///
/// split::split_archive("large.zip", "parts/", 100).unwrap();
/// ```
pub fn split_archive<P: AsRef<Path>>(
    input: P,
    output_dir: P,
    part_size_mb: u64,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create directory: {}", e),
        source: None,
    })?;

    // Try 7z split
    if let Ok(result) = split_with_7z(input_path, output_dir, part_size_mb) {
        return Ok(result);
    }

    // Try split command
    if let Ok(result) = split_with_split(input_path, output_dir, part_size_mb) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "Archive splitting failed. Install 7z or split.".to_string(),
        source: None,
    })
}

/// Split using 7z.
fn split_with_7z(input: &Path, output_dir: &Path, part_size_mb: u64) -> Result<ToolOutput> {
    let file_name = input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("archive");
    let output_path = output_dir.join(file_name);

    let mut cmd = Command::new("7z");
    cmd.arg("a")
        .arg(format!("-v{}m", part_size_mb))
        .arg(&output_path)
        .arg(input);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "7z split failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    // Count created parts
    let parts: Vec<_> = std::fs::read_dir(output_dir)
        .map_err(|e| DxError::FileIo {
            path: output_dir.to_path_buf(),
            message: format!("Failed to read directory: {}", e),
            source: None,
        })?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&file_name.to_string())
        })
        .map(|e| e.path())
        .collect();

    let part_count = parts.len();
    Ok(ToolOutput::success(format!(
        "Split archive into {} parts ({} MB each)",
        part_count, part_size_mb
    ))
    .with_paths(parts)
    .with_metadata("part_count", part_count.to_string())
    .with_metadata("part_size_mb", part_size_mb.to_string()))
}

/// Split using split command.
fn split_with_split(input: &Path, output_dir: &Path, part_size_mb: u64) -> Result<ToolOutput> {
    let file_name = input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("archive");
    let prefix = output_dir.join(format!("{}.part", file_name));

    let mut cmd = Command::new("split");
    cmd.arg("-b")
        .arg(format!("{}M", part_size_mb))
        .arg("-d") // Numeric suffixes
        .arg(input)
        .arg(&prefix);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run split: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "split command failed".to_string(),
            source: None,
        });
    }

    // Count created parts
    let parts: Vec<_> = std::fs::read_dir(output_dir)
        .map_err(|e| DxError::FileIo {
            path: output_dir.to_path_buf(),
            message: format!("Failed to read directory: {}", e),
            source: None,
        })?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".part"))
        .map(|e| e.path())
        .collect();

    Ok(ToolOutput::success(format!("Split into {} parts", parts.len())).with_paths(parts))
}

/// Split ZIP archive into volumes.
pub fn split_zip<P: AsRef<Path>>(inputs: &[P], output: P, part_size_mb: u64) -> Result<ToolOutput> {
    let output_path = output.as_ref();

    let mut cmd = Command::new("zip");
    cmd.arg("-r")
        .arg("-s")
        .arg(format!("{}m", part_size_mb))
        .arg(output_path);

    for input in inputs {
        cmd.arg(input.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run zip: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "zip split failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Created split ZIP ({} MB volumes)", part_size_mb),
        output_path,
    ))
}

/// Calculate optimal split size.
pub fn calculate_split_size(total_size: u64, max_parts: u32) -> u64 {
    let size_mb = total_size / (1024 * 1024);

    (size_mb / max_parts as u64).max(1)
}

/// Get file size in MB.
pub fn get_size_mb<P: AsRef<Path>>(path: P) -> Result<u64> {
    let metadata = std::fs::metadata(path.as_ref()).map_err(|e| DxError::FileIo {
        path: path.as_ref().to_path_buf(),
        message: format!("Failed to get file size: {}", e),
        source: None,
    })?;

    Ok(metadata.len() / (1024 * 1024))
}

/// Split for specific destination (e.g., email attachment).
pub fn split_for_email<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    // Standard email attachment limit is around 25MB
    split_archive(input, output_dir, 20)
}

/// Split for CD (700MB).
pub fn split_for_cd<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    split_archive(input, output_dir, 650)
}

/// Split for DVD (4.7GB).
pub fn split_for_dvd<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    split_archive(input, output_dir, 4500)
}

/// Split for FAT32 (4GB limit).
pub fn split_for_fat32<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    split_archive(input, output_dir, 4000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_split_size() {
        // 1GB file, max 10 parts = 100MB per part
        let size = calculate_split_size(1024 * 1024 * 1024, 10);
        assert_eq!(size, 102);
    }
}
