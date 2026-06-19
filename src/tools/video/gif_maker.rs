//! Video to GIF conversion tool.
//!
//! Convert video clips to optimized animated GIFs.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// GIF creation options.
#[derive(Debug, Clone)]
pub struct GifOptions {
    /// Output width (height auto-calculated to maintain aspect ratio).
    pub width: u32,
    /// Frame rate (fps).
    pub fps: u32,
    /// Start time in seconds.
    pub start: Option<f64>,
    /// Duration in seconds.
    pub duration: Option<f64>,
    /// Enable high quality palette generation.
    pub high_quality: bool,
    /// Number of colors (2-256).
    pub colors: u32,
    /// Enable dithering for better gradients.
    pub dither: bool,
    /// Loop count (0 = infinite).
    pub loop_count: i32,
}

impl Default for GifOptions {
    fn default() -> Self {
        Self {
            width: 480,
            fps: 15,
            start: None,
            duration: None,
            high_quality: true,
            colors: 256,
            dither: true,
            loop_count: 0,
        }
    }
}

impl GifOptions {
    /// Create options with specific dimensions.
    pub fn with_width(width: u32) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }

    /// Set frame rate.
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps.clamp(1, 60);
        self
    }

    /// Set time range.
    pub fn with_range(mut self, start: f64, duration: f64) -> Self {
        self.start = Some(start);
        self.duration = Some(duration);
        self
    }

    /// Disable high quality mode (faster, larger file).
    pub fn fast_mode(mut self) -> Self {
        self.high_quality = false;
        self
    }

    /// Set number of colors (fewer = smaller file).
    pub fn with_colors(mut self, colors: u32) -> Self {
        self.colors = colors.clamp(2, 256);
        self
    }

    /// Set loop count.
    pub fn with_loop(mut self, count: i32) -> Self {
        self.loop_count = count;
        self
    }
}

/// Convert video to animated GIF.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for output GIF
/// * `options` - GIF creation options
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::{video_to_gif, GifOptions};
///
/// // Create a 10-second GIF from video
/// let opts = GifOptions::with_width(320)
///     .with_fps(10)
///     .with_range(0.0, 10.0);
/// video_to_gif("video.mp4", "output.gif", opts).unwrap();
/// ```
pub fn video_to_gif<P: AsRef<Path>>(
    input: P,
    output: P,
    options: GifOptions,
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

    if options.high_quality {
        // Two-pass encoding for better quality
        create_gif_high_quality(input_path, output_path, &options)
    } else {
        // Single pass (faster, lower quality)
        create_gif_simple(input_path, output_path, &options)
    }
}

/// Create GIF with high quality two-pass encoding.
fn create_gif_high_quality(
    input: &Path,
    output: &Path,
    options: &GifOptions,
) -> Result<ToolOutput> {
    // Create temporary palette file
    let temp_dir = std::env::temp_dir();
    let palette_path = temp_dir.join(format!("palette_{}.png", std::process::id()));

    // Build filter string
    let filter = build_filter_string(options);
    let palette_filter = format!("{},palettegen=max_colors={}", filter, options.colors);

    // Pass 1: Generate palette
    let mut cmd1 = crate::tools::ffmpeg::command();
    cmd1.arg("-y");

    if let Some(start) = options.start {
        cmd1.arg("-ss").arg(start.to_string());
    }

    cmd1.arg("-i").arg(input);

    if let Some(duration) = options.duration {
        cmd1.arg("-t").arg(duration.to_string());
    }

    cmd1.arg("-vf")
        .arg(&palette_filter)
        .arg("-update")
        .arg("1")
        .arg(&palette_path);

    let output1 = cmd1.output().map_err(|e| DxError::Config {
        message: format!("Failed to generate palette: {}", e),
        source: None,
    })?;

    if !output1.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Palette generation failed: {}",
                String::from_utf8_lossy(&output1.stderr)
            ),
            source: None,
        });
    }

    // Pass 2: Create GIF using palette
    let dither_method = if options.dither { "sierra2_4a" } else { "none" };
    let gif_filter = format!("{}[x];[x][1:v]paletteuse=dither={}", filter, dither_method);

    let mut cmd2 = crate::tools::ffmpeg::command();
    cmd2.arg("-y");

    if let Some(start) = options.start {
        cmd2.arg("-ss").arg(start.to_string());
    }

    cmd2.arg("-i").arg(input);

    if let Some(duration) = options.duration {
        cmd2.arg("-t").arg(duration.to_string());
    }

    cmd2.arg("-i")
        .arg(&palette_path)
        .arg("-lavfi")
        .arg(&gif_filter)
        .arg("-loop")
        .arg(options.loop_count.to_string())
        .arg(output);

    let output2 = cmd2.output().map_err(|e| DxError::Config {
        message: format!("Failed to create GIF: {}", e),
        source: None,
    })?;

    // Clean up palette
    let _ = std::fs::remove_file(&palette_path);

    if !output2.status.success() {
        return Err(DxError::Config {
            message: format!(
                "GIF creation failed: {}",
                String::from_utf8_lossy(&output2.stderr)
            ),
            source: None,
        });
    }

    let output_size = std::fs::metadata(output).map_or(0, |m| m.len());

    Ok(ToolOutput::success_with_path(
        format!("Created high-quality GIF ({} bytes)", output_size),
        output,
    )
    .with_metadata("quality", "high")
    .with_metadata("colors", options.colors.to_string()))
}

/// Create GIF with simple single-pass encoding.
fn create_gif_simple(input: &Path, output: &Path, options: &GifOptions) -> Result<ToolOutput> {
    let filter = format!(
        "{}split[s0][s1];[s0]palettegen=max_colors={}[p];[s1][p]paletteuse",
        build_filter_string(options),
        options.colors
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y");

    if let Some(start) = options.start {
        cmd.arg("-ss").arg(start.to_string());
    }

    cmd.arg("-i").arg(input);

    if let Some(duration) = options.duration {
        cmd.arg("-t").arg(duration.to_string());
    }

    cmd.arg("-vf")
        .arg(&filter)
        .arg("-loop")
        .arg(options.loop_count.to_string())
        .arg(output);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to create GIF: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "GIF creation failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    let output_size = std::fs::metadata(output).map_or(0, |m| m.len());

    Ok(ToolOutput::success_with_path(
        format!("Created GIF ({} bytes)", output_size),
        output,
    ))
}

/// Build FFmpeg filter string for scaling and fps.
fn build_filter_string(options: &GifOptions) -> String {
    format!(
        "fps={},scale={}:-1:flags=lanczos",
        options.fps, options.width
    )
}

/// Quick GIF creation with defaults.
pub fn quick_gif<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    video_to_gif(input, output, GifOptions::default())
}

/// Create GIF from specific time range.
pub fn gif_from_range<P: AsRef<Path>>(
    input: P,
    output: P,
    start: f64,
    duration: f64,
    width: u32,
) -> Result<ToolOutput> {
    let options = GifOptions::with_width(width).with_range(start, duration);
    video_to_gif(input, output, options)
}

/// Create thumbnail GIF (short, looping preview).
pub fn create_preview_gif<P: AsRef<Path>>(
    input: P,
    output: P,
    duration: f64,
) -> Result<ToolOutput> {
    let options = GifOptions::with_width(320)
        .with_fps(10)
        .with_range(0.0, duration.min(5.0))
        .with_colors(128);
    video_to_gif(input, output, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gif_options() {
        let opts = GifOptions::with_width(320)
            .with_fps(10)
            .with_range(5.0, 10.0);

        assert_eq!(opts.width, 320);
        assert_eq!(opts.fps, 10);
        assert_eq!(opts.start, Some(5.0));
        assert_eq!(opts.duration, Some(10.0));
    }

    #[test]
    fn test_filter_string() {
        let opts = GifOptions {
            width: 480,
            fps: 15,
            ..Default::default()
        };
        let filter = build_filter_string(&opts);
        assert!(filter.contains("fps=15"));
        assert!(filter.contains("scale=480:-1"));
    }
}
