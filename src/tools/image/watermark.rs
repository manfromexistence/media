//! Watermark tool for images.
//!
//! Add text or image watermarks using ImageMagick.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Watermark position on the image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WatermarkPosition {
    /// Top-left corner.
    TopLeft,
    /// Top-center.
    TopCenter,
    /// Top-right corner.
    TopRight,
    /// Center-left.
    CenterLeft,
    /// Center of image.
    Center,
    /// Center-right.
    CenterRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-center.
    #[default]
    BottomCenter,
    /// Bottom-right corner.
    BottomRight,
}

impl WatermarkPosition {
    fn to_gravity(&self) -> &str {
        match self {
            Self::TopLeft => "NorthWest",
            Self::TopCenter => "North",
            Self::TopRight => "NorthEast",
            Self::CenterLeft => "West",
            Self::Center => "Center",
            Self::CenterRight => "East",
            Self::BottomLeft => "SouthWest",
            Self::BottomCenter => "South",
            Self::BottomRight => "SouthEast",
        }
    }
}

/// Watermark configuration options.
#[derive(Debug, Clone)]
pub struct WatermarkOptions {
    /// Text to overlay.
    pub text: Option<String>,
    /// Path to watermark image.
    pub image_path: Option<std::path::PathBuf>,
    /// Position of the watermark.
    pub position: WatermarkPosition,
    /// Opacity (0-100).
    pub opacity: u8,
    /// Font size for text watermarks.
    pub font_size: u32,
    /// Font color for text watermarks (hex).
    pub color: String,
    /// Margin from edges in pixels.
    pub margin: u32,
}

impl Default for WatermarkOptions {
    fn default() -> Self {
        Self {
            text: None,
            image_path: None,
            position: WatermarkPosition::default(),
            opacity: 50,
            font_size: 24,
            color: "#ffffff".to_string(),
            margin: 10,
        }
    }
}

/// Add a text watermark to an image.
///
/// # Arguments
/// * `input` - Path to the input image
/// * `output` - Path to the output image
/// * `text` - Watermark text
/// * `position` - Position of the watermark
///
/// # Example
/// ```no_run
/// use dx_media::tools::image::watermark::{add_text_watermark, WatermarkPosition};
///
/// add_text_watermark("photo.jpg", "watermarked.jpg", "© 2024", WatermarkPosition::BottomRight).unwrap();
/// ```
pub fn add_text_watermark<P: AsRef<Path>>(
    input: P,
    output: P,
    text: &str,
    position: WatermarkPosition,
) -> Result<ToolOutput> {
    let opts = WatermarkOptions {
        text: Some(text.to_string()),
        position,
        ..Default::default()
    };
    add_watermark_with_options(input, output, opts)
}

/// Add watermark with full options.
pub fn add_watermark_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: WatermarkOptions,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    if let Some(text) = &options.text {
        // Text watermark
        let font_size = options.font_size.to_string();
        let offset = format!("+{}+{}", options.margin, options.margin);

        let status = Command::new("magick")
            .args([
                "convert",
                input_path.to_str().unwrap_or(""),
                "-gravity",
                options.position.to_gravity(),
                "-fill",
                &options.color,
                "-pointsize",
                &font_size,
                "-annotate",
                &offset,
                text,
                output_path.to_str().unwrap_or(""),
            ])
            .status()
            .map_err(|e| DxError::Internal {
                message: format!("Failed to execute ImageMagick: {}", e),
            })?;

        if !status.success() {
            return Err(DxError::Internal {
                message: "ImageMagick text watermark command failed".to_string(),
            });
        }
    } else if let Some(image_path) = &options.image_path {
        // Image watermark
        let dissolve = options.opacity.to_string();
        let offset = format!("+{}+{}", options.margin, options.margin);

        let status = Command::new("magick")
            .args([
                "composite",
                "-dissolve",
                &dissolve,
                "-gravity",
                options.position.to_gravity(),
                "-geometry",
                &offset,
                image_path.to_str().unwrap_or(""),
                input_path.to_str().unwrap_or(""),
                output_path.to_str().unwrap_or(""),
            ])
            .status()
            .map_err(|e| DxError::Internal {
                message: format!("Failed to execute ImageMagick: {}", e),
            })?;

        if !status.success() {
            return Err(DxError::Internal {
                message: "ImageMagick image watermark command failed".to_string(),
            });
        }
    } else {
        return Err(DxError::Config {
            message: "Either text or image_path must be provided".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Watermark added successfully",
        output_path,
    ))
}

/// Add image watermark.
pub fn add_image_watermark<P: AsRef<Path>>(
    input: P,
    output: P,
    watermark: P,
    position: WatermarkPosition,
    opacity: u8,
) -> Result<ToolOutput> {
    let opts = WatermarkOptions {
        image_path: Some(watermark.as_ref().to_path_buf()),
        position,
        opacity,
        ..Default::default()
    };
    add_watermark_with_options(input, output, opts)
}

/// Add tiled watermark.
pub fn add_tiled_watermark<P: AsRef<Path>>(
    input: P,
    output: P,
    watermark: P,
    opacity: u8,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();
    let watermark_path = watermark.as_ref();

    let dissolve = opacity.to_string();

    let status = Command::new("magick")
        .args([
            "composite",
            "-dissolve",
            &dissolve,
            "-tile",
            watermark_path.to_str().unwrap_or(""),
            input_path.to_str().unwrap_or(""),
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick tiled watermark command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        "Tiled watermark added",
        output_path,
    ))
}

/// Batch add watermark to multiple images.
pub fn batch_watermark<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    text: &str,
    position: WatermarkPosition,
) -> Result<ToolOutput> {
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

        add_text_watermark(input_path, &output_path, text, position)?;
        processed += 1;
    }

    Ok(
        ToolOutput::success(format!("Added watermark to {} images", processed))
            .with_metadata("count", processed.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_gravity() {
        assert_eq!(WatermarkPosition::TopLeft.to_gravity(), "NorthWest");
        assert_eq!(WatermarkPosition::Center.to_gravity(), "Center");
        assert_eq!(WatermarkPosition::BottomRight.to_gravity(), "SouthEast");
    }
}
