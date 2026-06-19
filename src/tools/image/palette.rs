//! Color palette extraction tool.
//!
//! Extract dominant colors from images using ImageMagick.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Extracted color information.
#[derive(Debug, Clone)]
pub struct Color {
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
    /// Percentage of image this color represents.
    pub percentage: f32,
}

impl Color {
    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Convert to RGB string.
    pub fn to_rgb(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }
}

/// Extract color palette from image.
///
/// # Arguments
/// * `input` - Path to the input image
/// * `num_colors` - Number of colors to extract
///
/// # Example
/// ```no_run
/// use dx_media::tools::image::palette::extract_palette;
///
/// let colors = extract_palette("photo.jpg", 5).unwrap();
/// ```
pub fn extract_palette<P: AsRef<Path>>(input: P, num_colors: u32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let colors_arg = num_colors.to_string();

    let output = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-colors",
            &colors_arg,
            "-unique-colors",
            "-depth",
            "8",
            "txt:-",
        ])
        .output()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !output.status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick palette extraction failed".to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let colors = parse_palette_output(&stdout);

    let hex_colors: Vec<String> = colors.iter().map(|c| c.to_hex()).collect();

    Ok(ToolOutput::success(format!(
        "Extracted {} colors:\n{}",
        colors.len(),
        hex_colors.join("\n")
    ))
    .with_metadata("count", colors.len().to_string())
    .with_metadata("colors", hex_colors.join(",")))
}

/// Parse ImageMagick txt output.
fn parse_palette_output(output: &str) -> Vec<Color> {
    let mut colors = Vec::new();

    for line in output.lines() {
        // Skip header line
        if line.starts_with('#') || line.contains("ImageMagick") {
            continue;
        }

        // Parse lines like: "0,0: (255,255,255)  #FFFFFF  white"
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let rgb_str = &line[start + 1..end];
                let parts: Vec<&str> = rgb_str.split(',').collect();

                if parts.len() >= 3 {
                    let r = parts[0].trim().parse().unwrap_or(0);
                    let g = parts[1].trim().parse().unwrap_or(0);
                    let b = parts[2].trim().parse().unwrap_or(0);

                    colors.push(Color {
                        r,
                        g,
                        b,
                        percentage: 0.0,
                    });
                }
            }
        }
    }

    colors
}

/// Extract dominant color.
pub fn extract_dominant_color<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    let output = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-resize",
            "1x1!",
            "-format",
            "%[pixel:u]",
            "info:-",
        ])
        .output()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !output.status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick dominant color extraction failed".to_string(),
        });
    }

    let color = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(ToolOutput::success(format!("Dominant color: {}", color)).with_metadata("color", color))
}

/// Create color swatch image.
pub fn create_swatch<P: AsRef<Path>>(
    input: P,
    output: P,
    num_colors: u32,
    swatch_width: u32,
    swatch_height: u32,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let colors_arg = num_colors.to_string();
    let size = format!("{}x{}", swatch_width, swatch_height);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-colors",
            &colors_arg,
            "-unique-colors",
            "-scale",
            &size,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick swatch creation failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Created color swatch with {} colors", num_colors),
        output_path,
    ))
}

/// Get histogram of colors.
pub fn get_histogram<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    let output = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-format",
            "%c",
            "histogram:info:-",
        ])
        .output()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !output.status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick histogram command failed".to_string(),
        });
    }

    let histogram = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(ToolOutput::success(histogram))
}

/// Generate color palette image.
pub fn generate_palette_image<P: AsRef<Path>>(
    input: P,
    output: P,
    num_colors: u32,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let colors_arg = num_colors.to_string();

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-colors",
            &colors_arg,
            "-unique-colors",
            "-scale",
            "1000%",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick palette image creation failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Generated palette image with {} colors", num_colors),
        output_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_hex() {
        let color = Color {
            r: 255,
            g: 128,
            b: 0,
            percentage: 0.0,
        };
        assert_eq!(color.to_hex(), "#ff8000");
    }

    #[test]
    fn test_color_rgb() {
        let color = Color {
            r: 255,
            g: 128,
            b: 0,
            percentage: 0.0,
        };
        assert_eq!(color.to_rgb(), "rgb(255, 128, 0)");
    }
}
