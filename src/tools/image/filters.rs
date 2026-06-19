//! Image filters tool.
//!
//! Apply filters and effects to images using ImageMagick.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Image filter type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    /// Convert to grayscale.
    Grayscale,
    /// Apply sepia tone.
    Sepia,
    /// Invert colors.
    Invert,
    /// Apply blur.
    Blur,
    /// Apply sharpen.
    Sharpen,
    /// Apply emboss effect.
    Emboss,
    /// Apply edge detection.
    Edge,
    /// Apply oil paint effect.
    OilPaint,
    /// Apply charcoal effect.
    Charcoal,
    /// Apply sketch effect.
    Sketch,
    /// Apply vignette.
    Vignette,
    /// Apply posterize.
    Posterize,
    /// Apply solarize.
    Solarize,
}

impl Filter {
    fn to_args(&self) -> Vec<&str> {
        match self {
            Self::Grayscale => vec!["-colorspace", "Gray"],
            Self::Sepia => vec!["-sepia-tone", "80%"],
            Self::Invert => vec!["-negate"],
            Self::Blur => vec!["-blur", "0x3"],
            Self::Sharpen => vec!["-sharpen", "0x1"],
            Self::Emboss => vec!["-emboss", "0x1"],
            Self::Edge => vec!["-edge", "1"],
            Self::OilPaint => vec!["-paint", "4"],
            Self::Charcoal => vec!["-charcoal", "2"],
            Self::Sketch => vec!["-sketch", "0x10+120"],
            Self::Vignette => vec!["-vignette", "0x5"],
            Self::Posterize => vec!["-posterize", "4"],
            Self::Solarize => vec!["-solarize", "50%"],
        }
    }
}

/// Apply a filter to an image.
///
/// # Arguments
/// * `input` - Path to the input image
/// * `output` - Path to the output image
/// * `filter` - Filter to apply
///
/// # Example
/// ```no_run
/// use dx_media::tools::image::filters::{apply_filter, Filter};
///
/// apply_filter("photo.jpg", "grayscale.jpg", Filter::Grayscale).unwrap();
/// ```
pub fn apply_filter<P: AsRef<Path>>(input: P, output: P, filter: Filter) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let mut args = vec!["convert", input_path.to_str().unwrap_or("")];
    args.extend(filter.to_args());
    args.push(output_path.to_str().unwrap_or(""));

    let status = Command::new("magick")
        .args(&args)
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick filter command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Applied {:?} filter", filter),
        output_path,
    ))
}

/// Apply grayscale filter.
pub fn grayscale<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    apply_filter(input, output, Filter::Grayscale)
}

/// Apply sepia filter.
pub fn sepia<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    apply_filter(input, output, Filter::Sepia)
}

/// Apply blur filter with radius.
pub fn blur<P: AsRef<Path>>(input: P, output: P, radius: f32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let blur_arg = format!("0x{}", radius);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-blur",
            &blur_arg,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick blur command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Applied blur with radius {}", radius),
        output_path,
    ))
}

/// Apply sharpen filter.
pub fn sharpen<P: AsRef<Path>>(input: P, output: P, amount: f32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let sharpen_arg = format!("0x{}", amount);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-sharpen",
            &sharpen_arg,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick sharpen command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Applied sharpen with amount {}", amount),
        output_path,
    ))
}

/// Adjust brightness.
pub fn brightness<P: AsRef<Path>>(input: P, output: P, percent: i32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let brightness_arg = format!("{}%", 100 + percent);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-modulate",
            &brightness_arg,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick brightness command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Adjusted brightness by {}%", percent),
        output_path,
    ))
}

/// Adjust contrast.
pub fn contrast<P: AsRef<Path>>(input: P, output: P, percent: i32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let contrast_arg = format!("{}x{}%", percent, percent);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-brightness-contrast",
            &contrast_arg,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick contrast command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Adjusted contrast by {}%", percent),
        output_path,
    ))
}

/// Rotate image.
pub fn rotate<P: AsRef<Path>>(input: P, output: P, degrees: f32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let rotate_arg = degrees.to_string();

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-rotate",
            &rotate_arg,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick rotate command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Rotated by {} degrees", degrees),
        output_path,
    ))
}

/// Flip image horizontally.
pub fn flip_horizontal<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-flop",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick flip command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        "Flipped horizontally",
        output_path,
    ))
}

/// Flip image vertically.
pub fn flip_vertical<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-flip",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick flip command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        "Flipped vertically",
        output_path,
    ))
}

/// Crop image.
pub fn crop<P: AsRef<Path>>(
    input: P,
    output: P,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let crop_arg = format!("{}x{}+{}+{}", width, height, x, y);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-crop",
            &crop_arg,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick crop command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Cropped to {}x{} at ({}, {})", width, height, x, y),
        output_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_args() {
        assert_eq!(Filter::Grayscale.to_args(), vec!["-colorspace", "Gray"]);
        assert_eq!(Filter::Invert.to_args(), vec!["-negate"]);
    }
}
