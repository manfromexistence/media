//! Image resizing tool.
//!
//! Resize images using ImageMagick.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Resize filter/algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResizeFilter {
    /// Nearest neighbor (fast, pixelated).
    Nearest,
    /// Bilinear interpolation.
    Bilinear,
    /// Bicubic interpolation.
    Bicubic,
    /// Lanczos filter (high quality).
    #[default]
    Lanczos,
    /// Mitchell filter.
    Mitchell,
}

impl ResizeFilter {
    fn to_arg(&self) -> &str {
        match self {
            Self::Nearest => "Point",
            Self::Bilinear => "Bilinear",
            Self::Bicubic => "Bicubic",
            Self::Lanczos => "Lanczos",
            Self::Mitchell => "Mitchell",
        }
    }
}

/// Resize options.
#[derive(Debug, Clone)]
pub struct ResizeOptions {
    /// Target width (None to maintain aspect ratio).
    pub width: Option<u32>,
    /// Target height (None to maintain aspect ratio).
    pub height: Option<u32>,
    /// Resize filter.
    pub filter: ResizeFilter,
    /// Maintain aspect ratio.
    pub maintain_aspect: bool,
    /// Only shrink if larger.
    pub only_shrink: bool,
}

impl Default for ResizeOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            filter: ResizeFilter::default(),
            maintain_aspect: true,
            only_shrink: false,
        }
    }
}

/// Resize an image to specific dimensions.
///
/// # Arguments
/// * `input` - Path to the input image
/// * `output` - Path to the output image
/// * `width` - Target width
/// * `height` - Target height
///
/// # Example
/// ```no_run
/// use dx_media::tools::image::resizer::resize;
///
/// resize("large.jpg", "small.jpg", 800, 600).unwrap();
/// ```
pub fn resize<P: AsRef<Path>>(input: P, output: P, width: u32, height: u32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let size = format!("{}x{}", width, height);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-resize",
            &size,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick resize command failed".to_string(),
        });
    }

    Ok(
        ToolOutput::success_with_path(format!("Resized to {}x{}", width, height), output_path)
            .with_metadata("width", width.to_string())
            .with_metadata("height", height.to_string()),
    )
}

/// Resize with options.
pub fn resize_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: ResizeOptions,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let mut args = vec![
        "convert".to_string(),
        input_path.to_str().unwrap_or("").to_string(),
        "-filter".to_string(),
        options.filter.to_arg().to_string(),
    ];

    let size = match (options.width, options.height) {
        (Some(w), Some(h)) => {
            if options.maintain_aspect {
                format!("{}x{}", w, h)
            } else {
                format!("{}x{}!", w, h)
            }
        }
        (Some(w), None) => format!("{}x", w),
        (None, Some(h)) => format!("x{}", h),
        (None, None) => {
            return Err(DxError::Config {
                message: "Either width or height must be specified".to_string(),
                source: None,
            });
        }
    };

    let resize_flag = if options.only_shrink {
        "-resize"
    } else {
        "-resize"
    };
    let size_arg = if options.only_shrink {
        format!("{}\\>", size)
    } else {
        size
    };

    args.push(resize_flag.to_string());
    args.push(size_arg);
    args.push(output_path.to_str().unwrap_or("").to_string());

    let status = Command::new("magick")
        .args(&args)
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick resize command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path("Image resized", output_path))
}

/// Resize to fit within maximum dimensions.
pub fn resize_fit<P: AsRef<Path>>(
    input: P,
    output: P,
    max_width: u32,
    max_height: u32,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let size = format!("{}x{}", max_width, max_height);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-resize",
            &size,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick resize command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Resized to fit within {}x{}", max_width, max_height),
        output_path,
    ))
}

/// Scale image by percentage.
pub fn scale<P: AsRef<Path>>(input: P, output: P, percentage: u32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let scale_arg = format!("{}%", percentage);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-resize",
            &scale_arg,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick scale command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Scaled to {}%", percentage),
        output_path,
    ))
}

/// Create thumbnail.
pub fn thumbnail<P: AsRef<Path>>(input: P, output: P, size: u32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let size_arg = format!("{}x{}", size, size);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-thumbnail",
            &size_arg,
            "-gravity",
            "center",
            "-extent",
            &size_arg,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick thumbnail command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Created {}x{} thumbnail", size, size),
        output_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_arg() {
        assert_eq!(ResizeFilter::Lanczos.to_arg(), "Lanczos");
        assert_eq!(ResizeFilter::Nearest.to_arg(), "Point");
    }
}
