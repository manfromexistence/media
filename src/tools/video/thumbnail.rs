//! Video thumbnail extraction tool.
//!
//! Extract frames from videos as images for previews and thumbnails.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Thumbnail output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThumbnailFormat {
    /// JPEG format (smaller, lossy).
    #[default]
    Jpeg,
    /// PNG format (lossless).
    Png,
    /// WebP format.
    WebP,
}

impl ThumbnailFormat {
    /// Get file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
        }
    }
}

/// Thumbnail extraction options.
#[derive(Debug, Clone)]
pub struct ThumbnailOptions {
    /// Output format.
    pub format: ThumbnailFormat,
    /// Output width (height auto-calculated).
    pub width: Option<u32>,
    /// Output height (width auto-calculated).
    pub height: Option<u32>,
    /// JPEG quality (1-100).
    pub quality: u8,
    /// Timestamp in seconds to extract.
    pub timestamp: f64,
}

impl Default for ThumbnailOptions {
    fn default() -> Self {
        Self {
            format: ThumbnailFormat::Jpeg,
            width: None,
            height: None,
            quality: 90,
            timestamp: 0.0,
        }
    }
}

impl ThumbnailOptions {
    /// Create options at specific timestamp.
    pub fn at(timestamp: f64) -> Self {
        Self {
            timestamp,
            ..Default::default()
        }
    }

    /// Set output width.
    pub fn with_width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set output height.
    pub fn with_height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set output format.
    pub fn with_format(mut self, format: ThumbnailFormat) -> Self {
        self.format = format;
        self
    }

    /// Set JPEG quality.
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality.clamp(1, 100);
        self
    }
}

/// Extract a thumbnail from a video at a specific timestamp.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for output image
/// * `timestamp` - Time in seconds to extract frame
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::extract_thumbnail;
///
/// // Extract frame at 30 seconds
/// extract_thumbnail("video.mp4", "thumb.jpg", 30.0).unwrap();
/// ```
pub fn extract_thumbnail<P: AsRef<Path>>(
    input: P,
    output: P,
    timestamp: f64,
) -> Result<ToolOutput> {
    extract_thumbnail_with_options(input, output, ThumbnailOptions::at(timestamp))
}

/// Extract thumbnail with detailed options.
pub fn extract_thumbnail_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: ThumbnailOptions,
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

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-ss")
        .arg(options.timestamp.to_string())
        .arg("-i")
        .arg(input_path)
        .arg("-vframes")
        .arg("1");

    // Build scale filter
    let scale_filter = match (options.width, options.height) {
        (Some(w), Some(h)) => Some(format!("scale={}:{}", w, h)),
        (Some(w), None) => Some(format!("scale={}:-1", w)),
        (None, Some(h)) => Some(format!("scale=-1:{}", h)),
        (None, None) => None,
    };

    if let Some(filter) = scale_filter {
        cmd.arg("-vf").arg(filter);
    }

    // Quality settings
    match options.format {
        ThumbnailFormat::Jpeg => {
            cmd.arg("-q:v")
                .arg(((100 - options.quality) / 3 + 1).to_string());
        }
        ThumbnailFormat::Png => {
            // PNG compression level
        }
        ThumbnailFormat::WebP => {
            cmd.arg("-q:v").arg(options.quality.to_string());
        }
    }

    cmd.arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "FFmpeg failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Extracted thumbnail at {:.1}s", options.timestamp),
        output_path,
    ))
}

/// Extract multiple thumbnails at regular intervals.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output_dir` - Directory for output images
/// * `count` - Number of thumbnails to extract
pub fn extract_thumbnails<P: AsRef<Path>>(
    input: P,
    output_dir: P,
    count: usize,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    // Get video duration
    let duration = get_video_duration(input_path)?;
    let interval = duration / (count + 1) as f64;

    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("thumb");

    let mut extracted = Vec::new();

    for i in 1..=count {
        let timestamp = interval * i as f64;
        let output_path = output_dir.join(format!("{}_{:04}.jpg", file_stem, i));

        if extract_thumbnail(input_path, &output_path, timestamp).is_ok() {
            extracted.push(output_path);
        }
    }

    Ok(
        ToolOutput::success(format!("Extracted {} thumbnails", extracted.len()))
            .with_paths(extracted),
    )
}

/// Extract thumbnails at specific timestamps.
pub fn extract_thumbnails_at<P: AsRef<Path>>(
    input: P,
    output_dir: P,
    timestamps: &[f64],
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("thumb");

    let mut extracted = Vec::new();

    for (i, &timestamp) in timestamps.iter().enumerate() {
        let output_path = output_dir.join(format!("{}_{:04}.jpg", file_stem, i + 1));

        if extract_thumbnail(input_path, &output_path, timestamp).is_ok() {
            extracted.push(output_path);
        }
    }

    Ok(
        ToolOutput::success(format!("Extracted {} thumbnails", extracted.len()))
            .with_paths(extracted),
    )
}

/// Create a video contact sheet (grid of thumbnails).
pub fn create_contact_sheet<P: AsRef<Path>>(
    input: P,
    output: P,
    columns: u32,
    rows: u32,
    thumb_width: u32,
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

    let total_frames = columns * rows;
    let duration = get_video_duration(input_path)?;
    let _interval = duration / (total_frames + 1) as f64;

    // Use FFmpeg's tile filter
    let filter = format!(
        "select='not(mod(n,{}))',scale={}:-1,tile={}x{}",
        (duration * 25.0 / total_frames as f64) as u32, // Assuming 25fps
        thumb_width,
        columns,
        rows
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(&filter)
        .arg("-frames:v")
        .arg("1")
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Contact sheet creation failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Created {}x{} contact sheet", columns, rows),
        output_path,
    ))
}

/// Get video duration using FFprobe.
fn get_video_duration(input: &Path) -> Result<f64> {
    let output = crate::tools::ffprobe::command()
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(input)
        .output()
        .map_err(|e| DxError::Config {
            message: format!("Failed to run ffprobe: {}", e),
            source: None,
        })?;

    crate::tools::ffprobe::ensure_success(&output, "reading video duration")?;

    let duration_str = String::from_utf8_lossy(&output.stdout);
    duration_str.trim().parse().map_err(|_| DxError::Config {
        message: "Failed to parse video duration".to_string(),
        source: None,
    })
}

/// Extract the first frame as thumbnail.
pub fn extract_first_frame<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    extract_thumbnail(input, output, 0.0)
}

/// Extract a frame at video midpoint.
pub fn extract_middle_frame<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let duration = get_video_duration(input_path)?;
    extract_thumbnail(input_path, output.as_ref(), duration / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_options() {
        let opts = ThumbnailOptions::at(30.0).with_width(640).with_quality(85);

        assert_eq!(opts.timestamp, 30.0);
        assert_eq!(opts.width, Some(640));
        assert_eq!(opts.quality, 85);
    }

    #[test]
    fn test_thumbnail_format() {
        assert_eq!(ThumbnailFormat::Jpeg.extension(), "jpg");
        assert_eq!(ThumbnailFormat::Png.extension(), "png");
    }
}
