//! Video watermark tool.
//!
//! Add text or image watermarks to videos.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Watermark position.
#[derive(Debug, Clone, Copy, Default)]
pub enum WatermarkPosition {
    /// Top left corner.
    TopLeft,
    /// Top right corner.
    TopRight,
    /// Bottom left corner.
    #[default]
    BottomLeft,
    /// Bottom right corner.
    BottomRight,
    /// Center of frame.
    Center,
    /// Custom position (x, y).
    Custom(i32, i32),
}

impl WatermarkPosition {
    /// Get FFmpeg overlay position string for image watermark.
    fn to_overlay_position(&self, padding: u32) -> String {
        match self {
            WatermarkPosition::TopLeft => format!("{}:{}", padding, padding),
            WatermarkPosition::TopRight => format!("W-w-{}:{}", padding, padding),
            WatermarkPosition::BottomLeft => format!("{}:H-h-{}", padding, padding),
            WatermarkPosition::BottomRight => format!("W-w-{}:H-h-{}", padding, padding),
            WatermarkPosition::Center => "overlay=(W-w)/2:(H-h)/2".to_string(),
            WatermarkPosition::Custom(x, y) => format!("{}:{}", x, y),
        }
    }

    /// Get FFmpeg drawtext position for text watermark.
    fn to_drawtext_position(&self, padding: u32) -> String {
        match self {
            WatermarkPosition::TopLeft => format!("x={}:y={}", padding, padding),
            WatermarkPosition::TopRight => format!("x=w-text_w-{}:y={}", padding, padding),
            WatermarkPosition::BottomLeft => format!("x={}:y=h-text_h-{}", padding, padding),
            WatermarkPosition::BottomRight => {
                format!("x=w-text_w-{}:y=h-text_h-{}", padding, padding)
            }
            WatermarkPosition::Center => "x=(w-text_w)/2:y=(h-text_h)/2".to_string(),
            WatermarkPosition::Custom(x, y) => format!("x={}:y={}", x, y),
        }
    }
}

/// Text watermark options.
#[derive(Debug, Clone)]
pub struct TextWatermarkOptions {
    /// Text to display.
    pub text: String,
    /// Font size.
    pub font_size: u32,
    /// Font color (hex or name).
    pub color: String,
    /// Background color (optional).
    pub background: Option<String>,
    /// Position on video.
    pub position: WatermarkPosition,
    /// Padding from edges.
    pub padding: u32,
    /// Opacity (0.0 - 1.0).
    pub opacity: f32,
    /// Font family (must be installed).
    pub font: Option<String>,
}

impl Default for TextWatermarkOptions {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 24,
            color: "white".to_string(),
            background: None,
            position: WatermarkPosition::BottomRight,
            padding: 10,
            opacity: 0.8,
            font: None,
        }
    }
}

/// Image watermark options.
#[derive(Debug, Clone)]
pub struct ImageWatermarkOptions {
    /// Position on video.
    pub position: WatermarkPosition,
    /// Padding from edges.
    pub padding: u32,
    /// Scale watermark (1.0 = original size).
    pub scale: f32,
    /// Opacity (0.0 - 1.0).
    pub opacity: f32,
}

impl Default for ImageWatermarkOptions {
    fn default() -> Self {
        Self {
            position: WatermarkPosition::BottomRight,
            padding: 10,
            scale: 1.0,
            opacity: 0.8,
        }
    }
}

/// Add text watermark to video.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for watermarked output
/// * `text` - Watermark text
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::add_text_watermark;
///
/// add_text_watermark("video.mp4", "output.mp4", "© 2025 My Company").unwrap();
/// ```
pub fn add_text_watermark<P: AsRef<Path>>(input: P, output: P, text: &str) -> Result<ToolOutput> {
    let options = TextWatermarkOptions {
        text: text.to_string(),
        ..Default::default()
    };
    add_text_watermark_with_options(input, output, options)
}

/// Add text watermark with full options.
pub fn add_text_watermark_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: TextWatermarkOptions,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    // Escape text for FFmpeg
    let escaped_text = options.text.replace('\'', "\\'").replace(':', "\\:");

    let position = options.position.to_drawtext_position(options.padding);
    let alpha = (options.opacity * 255.0) as u8;

    let mut filter = format!(
        "drawtext=text='{}':fontsize={}:fontcolor={}@0x{:02X}:{}",
        escaped_text, options.font_size, options.color, alpha, position
    );

    if let Some(ref font) = options.font {
        filter.push_str(&format!(":font='{}'", font));
    }

    if let Some(ref bg) = options.background {
        filter.push_str(&format!(":box=1:boxcolor={}@0.5:boxborderw=5", bg));
    }

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(&filter)
        .arg("-c:a")
        .arg("copy")
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Text watermark failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Added text watermark: {}", options.text),
        output_path,
    ))
}

/// Add image watermark to video.
///
/// # Arguments
/// * `input` - Path to input video
/// * `watermark` - Path to watermark image (PNG with transparency recommended)
/// * `output` - Path for watermarked output
pub fn add_image_watermark<P: AsRef<Path>>(
    input: P,
    watermark: P,
    output: P,
) -> Result<ToolOutput> {
    add_image_watermark_with_options(input, watermark, output, ImageWatermarkOptions::default())
}

/// Add image watermark with full options.
pub fn add_image_watermark_with_options<P: AsRef<Path>>(
    input: P,
    watermark: P,
    output: P,
    options: ImageWatermarkOptions,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let watermark_path = watermark.as_ref();
    let output_path = output.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input video not found".to_string(),
            source: None,
        });
    }

    if !watermark_path.exists() {
        return Err(DxError::FileIo {
            path: watermark_path.to_path_buf(),
            message: "Watermark image not found".to_string(),
            source: None,
        });
    }

    let position = options.position.to_overlay_position(options.padding);

    // Build complex filter
    let mut filter = String::new();

    // Scale watermark if needed
    if (options.scale - 1.0).abs() > 0.01 {
        filter.push_str(&format!(
            "[1:v]scale=iw*{}:ih*{}[wm];",
            options.scale, options.scale
        ));
    }

    // Apply opacity if needed
    let wm_input = if (options.scale - 1.0).abs() > 0.01 {
        "[wm]"
    } else {
        "[1:v]"
    };

    if options.opacity < 1.0 {
        filter.push_str(&format!(
            "{}format=rgba,colorchannelmixer=aa={}[wmop];[0:v][wmop]overlay={}",
            wm_input, options.opacity, position
        ));
    } else if (options.scale - 1.0).abs() > 0.01 {
        filter.push_str(&format!("[0:v][wm]overlay={}", position));
    } else {
        filter = format!("overlay={}", position);
    }

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-i")
        .arg(watermark_path);

    if filter.contains('[') {
        cmd.arg("-filter_complex").arg(&filter);
    } else {
        cmd.arg("-filter_complex").arg(&filter);
    }

    cmd.arg("-c:a").arg("copy").arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Image watermark failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Added image watermark",
        output_path,
    ))
}

/// Add animated watermark (moves across screen).
pub fn add_animated_watermark<P: AsRef<Path>>(
    input: P,
    watermark: P,
    output: P,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let watermark_path = watermark.as_ref();
    let output_path = output.as_ref();

    // Scrolling watermark from right to left
    let filter = "overlay=x='W-mod(t*100,W+w)':y=10";

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-i")
        .arg(watermark_path)
        .arg("-filter_complex")
        .arg(filter)
        .arg("-c:a")
        .arg("copy")
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Animated watermark failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Added animated watermark",
        output_path,
    ))
}

/// Add timestamp watermark (current time displayed).
pub fn add_timestamp_watermark<P: AsRef<Path>>(
    input: P,
    output: P,
    format: Option<&str>,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let time_format = format.unwrap_or("%Y-%m-%d %H\\:%M\\:%S");

    let filter = format!(
        "drawtext=text='%{{localtime\\:{}}}':fontsize=16:fontcolor=white:x=10:y=h-th-10:box=1:boxcolor=black@0.5:boxborderw=3",
        time_format
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(&filter)
        .arg("-c:a")
        .arg("copy")
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Timestamp watermark failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Added timestamp watermark",
        output_path,
    ))
}

/// Batch add watermark to multiple videos.
pub fn batch_watermark<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    text: &str,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut processed = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("output.mp4");
        let output_path = output_dir.join(format!("wm_{}", file_name));

        if add_text_watermark(input_path, &output_path, text).is_ok() {
            processed.push(output_path);
        }
    }

    Ok(
        ToolOutput::success(format!("Watermarked {} videos", processed.len()))
            .with_paths(processed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_position() {
        let pos = WatermarkPosition::BottomRight;
        assert!(pos.to_overlay_position(10).contains("W-w"));

        let text_pos = WatermarkPosition::Center;
        assert!(text_pos.to_drawtext_position(10).contains("text_w"));
    }

    #[test]
    fn test_text_options() {
        let options = TextWatermarkOptions::default();
        assert_eq!(options.font_size, 24);
        assert_eq!(options.color, "white");
    }
}
