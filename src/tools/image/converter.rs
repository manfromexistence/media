//! Image format conversion tool.
//!
//! Convert between image formats using ImageMagick.

use crate::deps::check_tool_dependency;
use crate::error::{DxError, Result};
use crate::tools::{ToolOutput, ToolReceipt};
use crate::types::MediaType;
use std::path::Path;
use std::process::Command;

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFormat {
    /// JPEG format.
    #[default]
    Jpeg,
    /// PNG format.
    Png,
    /// WebP format.
    Webp,
    /// GIF format.
    Gif,
    /// BMP format.
    Bmp,
    /// TIFF format.
    Tiff,
    /// ICO format.
    Ico,
    /// AVIF format.
    Avif,
    /// HEIC format.
    Heic,
}

impl ImageFormat {
    /// Get the file extension for this format.
    pub fn extension(&self) -> &str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Ico => "ico",
            Self::Avif => "avif",
            Self::Heic => "heic",
        }
    }

    /// Parse format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "webp" => Some(Self::Webp),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            "tiff" | "tif" => Some(Self::Tiff),
            "ico" => Some(Self::Ico),
            "avif" => Some(Self::Avif),
            "heic" | "heif" => Some(Self::Heic),
            _ => None,
        }
    }
}

fn bitmap_media_type_for_path(path: &Path) -> MediaType {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| MediaType::Gif.matches_extension(extension))
    {
        MediaType::Gif
    } else {
        MediaType::Image
    }
}

/// Convert an image to a different format.
///
/// Uses ImageMagick's `convert` command.
///
/// # Arguments
/// * `input` - Path to the input image
/// * `output` - Path to the output image (format determined by extension)
///
/// # Errors
///
/// Returns `DxError::MissingDependency` if ImageMagick is not installed.
///
/// # Example
/// ```no_run
/// use dx_media::tools::image::converter::convert;
///
/// convert("photo.jpg", "photo.png").unwrap();
/// ```
pub fn convert<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    // Check for ImageMagick dependency
    check_tool_dependency("image::convert")?;

    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let status = Command::new(crate::deps::image_magick_binary())
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick convert command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!(
            "Converted {} to {}",
            input_path.display(),
            output_path.display()
        ),
        output_path,
    )
    .with_receipt(
        ToolReceipt::local("image.convert")
            .with_source(input_path.display().to_string())
            .with_dependency("imagemagick"),
    )
    .with_output_type_validation(output_path, bitmap_media_type_for_path(output_path)))
}

/// Convert image to a specific format.
pub fn convert_to_format<P: AsRef<Path>>(
    input: P,
    output_dir: P,
    format: ImageFormat,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let output_path = output_dir.join(format!("{}.{}", stem, format.extension()));

    convert(input_path, &output_path)
}

/// Batch convert multiple images.
pub fn convert_batch<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    format: ImageFormat,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut converted = 0;
    for input in inputs {
        let input_path = input.as_ref();
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = output_dir.join(format!("{}.{}", stem, format.extension()));

        convert(input_path, &output_path)?;
        converted += 1;
    }

    Ok(ToolOutput::success(format!(
        "Converted {} images to {} format",
        converted,
        format.extension()
    ))
    .with_metadata("count", converted.to_string())
    .with_metadata("format", format.extension().to_string()))
}

/// Get image information.
pub fn get_info<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    // Check for ImageMagick dependency
    check_tool_dependency("image::convert")?;

    let input_path = input.as_ref();

    let output = Command::new("magick")
        .args(["identify", "-verbose", input_path.to_str().unwrap_or("")])
        .output()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick identify: {}", e),
        })?;

    if !output.status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick identify command failed".to_string(),
        });
    }

    let info = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(ToolOutput::success(info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_extension() {
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Webp.extension(), "webp");
    }

    #[test]
    fn test_format_from_extension() {
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("PNG"), Some(ImageFormat::Png));
    }
}
