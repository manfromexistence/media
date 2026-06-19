//! EXIF metadata handling tool.
//!
//! Read and modify image metadata using exiftool command.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// EXIF metadata information.
#[derive(Debug, Clone, Default)]
pub struct ExifInfo {
    /// All metadata fields.
    pub fields: HashMap<String, String>,
}

impl ExifInfo {
    /// Get a field value.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.fields.get(key)
    }

    /// Check if any GPS data is present.
    pub fn has_gps(&self) -> bool {
        self.fields.keys().any(|k| k.contains("GPS"))
    }

    /// Check if the image has any metadata.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Read EXIF metadata from an image.
///
/// Uses `exiftool` command.
///
/// # Arguments
/// * `input` - Path to the image file
///
/// # Example
/// ```no_run
/// use dx_media::tools::image::exif::read_exif;
///
/// let info = read_exif("photo.jpg").unwrap();
/// ```
pub fn read_exif<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    let output = Command::new("exiftool")
        .arg(input_path.to_str().unwrap_or(""))
        .output()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute exiftool: {}", e),
        })?;

    if !output.status.success() {
        return Err(DxError::Internal {
            message: "exiftool command failed".to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(ToolOutput::success(stdout))
}

/// Read EXIF metadata as JSON.
pub fn read_exif_json<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    let output = Command::new("exiftool")
        .args(["-json", input_path.to_str().unwrap_or("")])
        .output()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute exiftool: {}", e),
        })?;

    if !output.status.success() {
        return Err(DxError::Internal {
            message: "exiftool JSON command failed".to_string(),
        });
    }

    let json = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(ToolOutput::success(json))
}

/// Strip all metadata from an image.
///
/// # Example
/// ```no_run
/// use dx_media::tools::image::exif::strip_metadata;
///
/// strip_metadata("photo.jpg", "clean.jpg").unwrap();
/// ```
pub fn strip_metadata<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Copy file first
    std::fs::copy(input_path, output_path).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to copy file: {}", e),
        source: None,
    })?;

    let status = Command::new("exiftool")
        .args([
            "-all=",
            "-overwrite_original",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute exiftool: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "exiftool strip command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        "Stripped all metadata",
        output_path,
    ))
}

/// Strip GPS data only.
pub fn strip_gps<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Copy file first
    std::fs::copy(input_path, output_path).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to copy file: {}", e),
        source: None,
    })?;

    let status = Command::new("exiftool")
        .args([
            "-gps:all=",
            "-overwrite_original",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute exiftool: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "exiftool GPS strip command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        "Stripped GPS metadata",
        output_path,
    ))
}

/// Set copyright metadata.
pub fn set_copyright<P: AsRef<Path>>(input: P, output: P, copyright: &str) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Copy file first
    std::fs::copy(input_path, output_path).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to copy file: {}", e),
        source: None,
    })?;

    let copyright_arg = format!("-Copyright={}", copyright);

    let status = Command::new("exiftool")
        .args([
            &copyright_arg,
            "-overwrite_original",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute exiftool: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "exiftool copyright command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Set copyright to: {}", copyright),
        output_path,
    ))
}

/// Set artist/author metadata.
pub fn set_artist<P: AsRef<Path>>(input: P, output: P, artist: &str) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Copy file first
    std::fs::copy(input_path, output_path).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to copy file: {}", e),
        source: None,
    })?;

    let artist_arg = format!("-Artist={}", artist);

    let status = Command::new("exiftool")
        .args([
            &artist_arg,
            "-overwrite_original",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute exiftool: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "exiftool artist command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Set artist to: {}", artist),
        output_path,
    ))
}

/// Copy metadata from one image to another.
pub fn copy_metadata<P: AsRef<Path>>(source: P, target: P) -> Result<ToolOutput> {
    let source_path = source.as_ref();
    let target_path = target.as_ref();

    let status = Command::new("exiftool")
        .args([
            "-TagsFromFile",
            source_path.to_str().unwrap_or(""),
            "-all:all",
            "-overwrite_original",
            target_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute exiftool: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "exiftool copy metadata command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        "Copied metadata from source to target",
        target_path,
    ))
}

/// Batch strip metadata from multiple files.
pub fn batch_strip_metadata<P: AsRef<Path>>(inputs: &[P], output_dir: P) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut processed = 0;
    for input in inputs {
        let input_path = input.as_ref();
        let filename = input_path.file_name().unwrap_or_default();
        let output_path = output_dir.join(filename);

        strip_metadata(input_path, &output_path)?;
        processed += 1;
    }

    Ok(
        ToolOutput::success(format!("Stripped metadata from {} files", processed))
            .with_metadata("count", processed.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exif_info_empty() {
        let info = ExifInfo::default();
        assert!(info.is_empty());
    }

    #[test]
    fn test_exif_info_has_gps() {
        let mut info = ExifInfo::default();
        info.fields
            .insert("GPSLatitude".to_string(), "40.7128".to_string());
        assert!(info.has_gps());
    }
}
