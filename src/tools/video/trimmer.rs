//! Video trimming tool.
//!
//! Cut video segments without re-encoding (stream copy) or with re-encoding.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Trim mode options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrimMode {
    /// Stream copy (fast, no quality loss, but may have keyframe issues).
    #[default]
    Copy,
    /// Re-encode (slower, but precise cuts).
    Reencode,
}

/// Video trimming options.
#[derive(Debug, Clone)]
pub struct TrimOptions {
    /// Start time in seconds.
    pub start: f64,
    /// End time in seconds (or None to trim to end).
    pub end: Option<f64>,
    /// Duration in seconds (alternative to end).
    pub duration: Option<f64>,
    /// Trim mode.
    pub mode: TrimMode,
    /// Seek to keyframe (faster but less precise).
    pub keyframe_seek: bool,
}

impl TrimOptions {
    /// Create trim options with start and end times.
    pub fn new(start: f64, end: f64) -> Self {
        Self {
            start,
            end: Some(end),
            duration: None,
            mode: TrimMode::Copy,
            keyframe_seek: false,
        }
    }

    /// Create trim options with start and duration.
    pub fn with_duration(start: f64, duration: f64) -> Self {
        Self {
            start,
            end: None,
            duration: Some(duration),
            mode: TrimMode::Copy,
            keyframe_seek: false,
        }
    }

    /// Set trim mode.
    pub fn with_mode(mut self, mode: TrimMode) -> Self {
        self.mode = mode;
        self
    }

    /// Enable keyframe seeking (faster but less precise).
    pub fn with_keyframe_seek(mut self) -> Self {
        self.keyframe_seek = true;
        self
    }
}

/// Trim a video to specified time range.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for trimmed output
/// * `start` - Start time in seconds
/// * `end` - End time in seconds
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::trim_video;
///
/// // Extract 30 seconds starting at 1 minute
/// trim_video("full.mp4", "clip.mp4", 60.0, 90.0).unwrap();
/// ```
pub fn trim_video<P: AsRef<Path>>(input: P, output: P, start: f64, end: f64) -> Result<ToolOutput> {
    trim_video_with_options(input, output, TrimOptions::new(start, end))
}

/// Trim video with detailed options.
pub fn trim_video_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: TrimOptions,
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
    cmd.arg("-y");

    // Keyframe seeking (before input for faster seek)
    if options.keyframe_seek {
        cmd.arg("-ss").arg(format_time(options.start));
    }

    cmd.arg("-i").arg(input_path);

    // Accurate seeking (after input for precise cuts)
    if !options.keyframe_seek {
        cmd.arg("-ss").arg(format_time(options.start));
    }

    // Duration or end time
    if let Some(duration) = options.duration {
        cmd.arg("-t").arg(format_time(duration));
    } else if let Some(end) = options.end {
        let duration = end - options.start;
        if duration > 0.0 {
            cmd.arg("-t").arg(format_time(duration));
        }
    }

    // Codec settings
    match options.mode {
        TrimMode::Copy => {
            cmd.arg("-c").arg("copy");
        }
        TrimMode::Reencode => {
            // Use default codecs
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-crf")
                .arg("18")
                .arg("-c:a")
                .arg("aac");
        }
    }

    // Avoid negative timestamps
    cmd.arg("-avoid_negative_ts").arg("make_zero");

    cmd.arg(output_path);

    let output = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output.status.success() {
        return Err(DxError::Config {
            message: format!("FFmpeg failed: {}", String::from_utf8_lossy(&output.stderr)),
            source: None,
        });
    }

    let output_size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    let duration = options
        .duration
        .or_else(|| options.end.map(|e| e - options.start))
        .unwrap_or(0.0);

    Ok(ToolOutput::success_with_path(
        format!(
            "Trimmed {:.1}s from {:.1}s ({} bytes)",
            duration, options.start, output_size
        ),
        output_path,
    )
    .with_metadata("start_time", options.start.to_string())
    .with_metadata("duration", duration.to_string()))
}

/// Extract a clip with duration.
pub fn extract_clip<P: AsRef<Path>>(
    input: P,
    output: P,
    start: f64,
    duration: f64,
) -> Result<ToolOutput> {
    trim_video_with_options(input, output, TrimOptions::with_duration(start, duration))
}

/// Split video at specific timestamps.
pub fn split_video<P: AsRef<Path>>(
    input: P,
    output_dir: P,
    split_points: &[f64],
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
        .unwrap_or("part");
    let extension = input_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("mp4");

    let mut parts = Vec::new();
    let mut points = vec![0.0];
    points.extend(split_points);

    for (i, window) in points.windows(2).enumerate() {
        let start = window[0];
        let end = window[1];
        let output_path = output_dir.join(format!("{}_{:03}.{}", file_stem, i + 1, extension));

        if trim_video(input_path, &output_path, start, end).is_ok() {
            parts.push(output_path);
        }
    }

    // Handle last segment (from last split point to end)
    if let Some(&last_point) = split_points.last() {
        let output_path = output_dir.join(format!(
            "{}_{:03}.{}",
            file_stem,
            parts.len() + 1,
            extension
        ));

        // Trim from last point to end (no end time)
        let options = TrimOptions {
            start: last_point,
            end: None,
            duration: None,
            mode: TrimMode::Copy,
            keyframe_seek: false,
        };

        if trim_video_with_options(input_path, &output_path, options).is_ok() {
            parts.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Split video into {} parts", parts.len())).with_paths(parts))
}

/// Format time in seconds to FFmpeg format (HH:MM:SS.mmm).
fn format_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = seconds % 60.0;
    format!("{:02}:{:02}:{:06.3}", hours, minutes, secs)
}

/// Parse time string to seconds.
/// Supports formats: "HH:MM:SS", "MM:SS", "SS", "HH:MM:SS.mmm"
pub fn parse_time(time_str: &str) -> Option<f64> {
    let parts: Vec<&str> = time_str.split(':').collect();

    match parts.len() {
        1 => parts[0].parse().ok(),
        2 => {
            let minutes: f64 = parts[0].parse().ok()?;
            let seconds: f64 = parts[1].parse().ok()?;
            Some(minutes * 60.0 + seconds)
        }
        3 => {
            let hours: f64 = parts[0].parse().ok()?;
            let minutes: f64 = parts[1].parse().ok()?;
            let seconds: f64 = parts[2].parse().ok()?;
            Some(hours * 3600.0 + minutes * 60.0 + seconds)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(0.0), "00:00:00.000");
        assert_eq!(format_time(61.5), "00:01:01.500");
        assert_eq!(format_time(3661.0), "01:01:01.000");
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("30"), Some(30.0));
        assert_eq!(parse_time("1:30"), Some(90.0));
        assert_eq!(parse_time("1:01:30"), Some(3690.0));
        assert_eq!(parse_time("0:00:30.5"), Some(30.5));
    }

    #[test]
    fn test_trim_options() {
        let opts = TrimOptions::new(10.0, 20.0);
        assert_eq!(opts.start, 10.0);
        assert_eq!(opts.end, Some(20.0));
    }
}
