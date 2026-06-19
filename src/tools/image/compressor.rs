//! Image compression tool.
//!
//! Compress images using ImageMagick.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Compression quality level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompressionQuality {
    /// Low quality, high compression.
    Low,
    /// Medium quality.
    #[default]
    Medium,
    /// High quality.
    High,
    /// Maximum quality, minimal compression.
    Maximum,
    /// Custom quality (0-100).
    Custom(u8),
}

impl CompressionQuality {
    fn to_value(&self) -> u8 {
        match self {
            Self::Low => 30,
            Self::Medium => 60,
            Self::High => 80,
            Self::Maximum => 95,
            Self::Custom(v) => *v,
        }
    }
}

/// Compress an image.
///
/// # Arguments
/// * `input` - Path to the input image
/// * `output` - Path to the output image
/// * `quality` - Quality level (0-100)
///
/// # Example
/// ```no_run
/// use dx_media::tools::image::compressor::compress;
///
/// compress("large.jpg", "compressed.jpg", 75).unwrap();
/// ```
pub fn compress<P: AsRef<Path>>(input: P, output: P, quality: u8) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let quality_arg = quality.to_string();

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-quality",
            &quality_arg,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick compress command failed".to_string(),
        });
    }

    // Get file sizes for comparison
    let input_size = std::fs::metadata(input_path).map_or(0, |m| m.len());
    let output_size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    let savings = if input_size > 0 {
        ((input_size - output_size) as f64 / input_size as f64 * 100.0) as i32
    } else {
        0
    };

    Ok(ToolOutput::success_with_path(
        format!(
            "Compressed with quality {}. Size: {} -> {} ({}% saved)",
            quality,
            format_size(input_size),
            format_size(output_size),
            savings
        ),
        output_path,
    )
    .with_metadata("quality", quality.to_string())
    .with_metadata("input_size", input_size.to_string())
    .with_metadata("output_size", output_size.to_string())
    .with_metadata("savings_percent", savings.to_string()))
}

/// Compress with quality level.
pub fn compress_with_level<P: AsRef<Path>>(
    input: P,
    output: P,
    level: CompressionQuality,
) -> Result<ToolOutput> {
    compress(input, output, level.to_value())
}

/// Compress image to target file size.
pub fn compress_to_size<P: AsRef<Path>>(input: P, output: P, target_kb: u64) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let target_bytes = target_kb * 1024;

    // Binary search for quality
    let mut low = 1u8;
    let mut high = 100u8;
    let mut best_quality = 50u8;

    while low <= high {
        let mid = u8::midpoint(low, high);

        // Try this quality
        let temp_output = output_path.with_extension("temp.jpg");
        compress(input_path, &temp_output, mid)?;

        let size = std::fs::metadata(&temp_output).map_or(0, |m| m.len());

        let _ = std::fs::remove_file(&temp_output);

        if size <= target_bytes {
            best_quality = mid;
            low = mid + 1;
        } else {
            high = mid.saturating_sub(1);
        }
    }

    compress(input_path, output_path, best_quality)?;

    let final_size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    Ok(ToolOutput::success_with_path(
        format!(
            "Compressed to {} (target: {}KB, quality: {})",
            format_size(final_size),
            target_kb,
            best_quality
        ),
        output_path,
    )
    .with_metadata("quality", best_quality.to_string())
    .with_metadata("final_size", final_size.to_string()))
}

/// Strip metadata and optimize.
pub fn optimize<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-strip",
            "-interlace",
            "Plane",
            "-sampling-factor",
            "4:2:0",
            "-quality",
            "85",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick optimize command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        "Image optimized (stripped metadata, optimized encoding)",
        output_path,
    ))
}

/// Losslessly optimize PNG.
pub fn optimize_png<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-strip",
            "-define",
            "png:compression-filter=5",
            "-define",
            "png:compression-level=9",
            "-define",
            "png:compression-strategy=1",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick PNG optimize command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path("PNG optimized", output_path))
}

/// Format file size for display.
fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_values() {
        assert_eq!(CompressionQuality::Low.to_value(), 30);
        assert_eq!(CompressionQuality::Medium.to_value(), 60);
        assert_eq!(CompressionQuality::High.to_value(), 80);
        assert_eq!(CompressionQuality::Custom(50).to_value(), 50);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500B");
        assert_eq!(format_size(1536), "1.50KB");
        assert_eq!(format_size(1572864), "1.50MB");
    }
}
