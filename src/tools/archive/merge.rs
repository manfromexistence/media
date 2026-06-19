//! Archive merging utilities.
//!
//! Merge split archives back together.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Merge split archive parts.
///
/// # Arguments
/// * `parts` - Archive parts in order
/// * `output` - Output merged file
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::merge;
///
/// merge::merge_archives(&["archive.001", "archive.002"], "merged.zip").unwrap();
/// ```
pub fn merge_archives<P: AsRef<Path>>(parts: &[P], output: P) -> Result<ToolOutput> {
    if parts.is_empty() {
        return Err(DxError::Config {
            message: "No parts provided".to_string(),
            source: None,
        });
    }

    let output_path = output.as_ref();

    // Check all parts exist
    for part in parts {
        if !part.as_ref().exists() {
            return Err(DxError::FileIo {
                path: part.as_ref().to_path_buf(),
                message: "Part not found".to_string(),
                source: None,
            });
        }
    }

    // Detect split type and merge accordingly
    let first_part = parts[0].as_ref();
    let name = first_part.to_string_lossy().to_lowercase();

    // 7z split archives (.001, .002, etc.)
    if name.ends_with(".001") || name.contains(".7z.") {
        return merge_7z_parts(first_part, output_path);
    }

    // ZIP split archives (.z01, .z02, etc. or .zip.001)
    if name.ends_with(".z01") || name.ends_with(".zip.001") {
        return merge_zip_parts(first_part, output_path);
    }

    // Generic binary concatenation
    merge_binary(parts, output_path)
}

/// Merge 7z split archives.
fn merge_7z_parts(first_part: &Path, output: &Path) -> Result<ToolOutput> {
    // 7z can extract directly from first part
    let mut cmd = Command::new("7z");
    cmd.arg("x")
        .arg("-y")
        .arg(format!(
            "-o{}",
            output.parent().unwrap_or(Path::new(".")).to_string_lossy()
        ))
        .arg(first_part);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "7z extraction failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Merged and extracted 7z split archive",
        output,
    ))
}

/// Merge ZIP split archives.
fn merge_zip_parts(first_part: &Path, output: &Path) -> Result<ToolOutput> {
    // Find main .zip file
    let dir = first_part.parent().unwrap_or(Path::new("."));
    let base_name = first_part
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Look for .zip file with same base name
    let main_zip = dir.join(format!("{}.zip", base_name.trim_end_matches(".z")));

    if main_zip.exists() {
        // Combine using zip -FF
        let mut cmd = Command::new("zip");
        cmd.arg("-FF").arg(&main_zip).arg("--out").arg(output);

        if let Ok(result) = cmd.output() {
            if result.status.success() {
                return Ok(ToolOutput::success_with_path(
                    "Fixed and merged split ZIP",
                    output,
                ));
            }
        }
    }

    // Try 7z extraction
    let mut cmd = Command::new("7z");
    cmd.arg("x")
        .arg("-y")
        .arg(format!(
            "-o{}",
            output.parent().unwrap_or(Path::new(".")).to_string_lossy()
        ))
        .arg(first_part);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "ZIP merge failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Merged split ZIP archive",
        output,
    ))
}

/// Merge parts by binary concatenation.
fn merge_binary<P: AsRef<Path>>(parts: &[P], output: &Path) -> Result<ToolOutput> {
    use std::io::Write;

    let mut out_file = std::fs::File::create(output).map_err(|e| DxError::FileIo {
        path: output.to_path_buf(),
        message: format!("Failed to create output file: {}", e),
        source: None,
    })?;

    let mut total_size = 0u64;

    for part in parts {
        let data = std::fs::read(part.as_ref()).map_err(|e| DxError::FileIo {
            path: part.as_ref().to_path_buf(),
            message: format!("Failed to read part: {}", e),
            source: None,
        })?;

        total_size += data.len() as u64;

        out_file.write_all(&data).map_err(|e| DxError::FileIo {
            path: output.to_path_buf(),
            message: format!("Failed to write: {}", e),
            source: None,
        })?;
    }

    Ok(ToolOutput::success_with_path(
        format!("Merged {} parts ({} bytes)", parts.len(), total_size),
        output,
    )
    .with_metadata("total_size", total_size.to_string())
    .with_metadata("part_count", parts.len().to_string()))
}

/// Auto-detect and merge split archive.
pub fn auto_merge<P: AsRef<Path>>(first_part: P, output: P) -> Result<ToolOutput> {
    let first = first_part.as_ref();
    let output = output.as_ref();

    if !first.exists() {
        return Err(DxError::FileIo {
            path: first.to_path_buf(),
            message: "First part not found".to_string(),
            source: None,
        });
    }

    // Find all parts
    let parts = find_related_parts(first)?;

    if parts.is_empty() {
        return Err(DxError::Config {
            message: "No parts found".to_string(),
            source: None,
        });
    }

    merge_archives(&parts, output.to_path_buf())
}

/// Find related parts of split archive.
pub fn find_related_parts<P: AsRef<Path>>(first_part: P) -> Result<Vec<std::path::PathBuf>> {
    let first = first_part.as_ref();
    let dir = first.parent().unwrap_or(Path::new("."));
    let file_name = first.file_name().and_then(|s| s.to_str()).unwrap_or("");

    // Extract base name (without number suffix)
    let base_name = extract_base_name(file_name);

    let mut parts: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| DxError::FileIo {
            path: dir.to_path_buf(),
            message: format!("Failed to read directory: {}", e),
            source: None,
        })?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with(&base_name) && is_part_file(&name)
        })
        .map(|e| e.path())
        .collect();

    // Sort parts numerically
    parts.sort_by(|a, b| {
        let a_num = extract_part_number(a);
        let b_num = extract_part_number(b);
        a_num.cmp(&b_num)
    });

    Ok(parts)
}

/// Extract base name from split archive part.
fn extract_base_name(name: &str) -> String {
    // Handle various split patterns
    // file.zip.001 -> file.zip
    // file.7z.001 -> file.7z
    // file.z01 -> file
    // file.part1.rar -> file

    let lower = name.to_lowercase();

    if lower.ends_with(".001") || lower.ends_with(".002") {
        // Remove .001, .002, etc.
        name.rsplit_once('.')
            .map(|(b, _)| b)
            .unwrap_or(name)
            .to_string()
    } else if lower.contains(".z0") || lower.contains(".z1") {
        // ZIP split: file.z01, file.z02
        name.rsplit_once('.')
            .map(|(b, _)| b)
            .unwrap_or(name)
            .to_string()
    } else if lower.contains(".part") {
        // RAR split: file.part1.rar
        name.split(".part").next().unwrap_or(name).to_string()
    } else {
        name.to_string()
    }
}

/// Check if file looks like a split part.
fn is_part_file(name: &str) -> bool {
    let lower = name.to_lowercase();

    // Common split patterns
    lower.ends_with(".001")
        || lower.ends_with(".002")
        || lower.contains(".z0")
        || lower.contains(".z1")
        || lower.contains(".part")
        || lower.ends_with(".aa")
        || lower.ends_with(".ab")
}

/// Extract part number from filename.
fn extract_part_number(path: &Path) -> u32 {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    // Try to find numeric suffix
    for part in name.rsplit('.') {
        if let Ok(num) = part.parse::<u32>() {
            return num;
        }
        // Handle z01, z02, etc.
        if part.starts_with('z') {
            if let Ok(num) = part[1..].parse::<u32>() {
                return num;
            }
        }
    }

    0
}

/// Verify all parts are present.
pub fn verify_parts<P: AsRef<Path>>(first_part: P) -> Result<ToolOutput> {
    let parts = find_related_parts(first_part)?;

    let mut missing = Vec::new();
    let mut total_size = 0u64;

    for (i, part) in parts.iter().enumerate() {
        if !part.exists() {
            missing.push(format!("Part {} missing: {}", i + 1, part.display()));
        } else if let Ok(meta) = std::fs::metadata(part) {
            total_size += meta.len();
        }
    }

    if missing.is_empty() {
        Ok(ToolOutput::success(format!(
            "All {} parts present ({} bytes total)",
            parts.len(),
            total_size
        ))
        .with_metadata("part_count", parts.len().to_string())
        .with_metadata("total_size", total_size.to_string())
        .with_metadata("valid", "true".to_string()))
    } else {
        Ok(
            ToolOutput::success(format!("Missing parts:\n{}", missing.join("\n")))
                .with_metadata("valid", "false".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_name() {
        assert_eq!(extract_base_name("archive.zip.001"), "archive.zip");
        assert_eq!(extract_base_name("archive.z01"), "archive");
    }

    #[test]
    fn test_is_part_file() {
        assert!(is_part_file("file.001"));
        assert!(is_part_file("file.z01"));
        assert!(!is_part_file("file.txt"));
    }
}
