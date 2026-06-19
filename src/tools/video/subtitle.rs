//! Video subtitle tool.
//!
//! Add, extract, and manipulate subtitles in videos.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Subtitle format types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubtitleFormat {
    /// SubRip format (.srt)
    Srt,
    /// Advanced SubStation Alpha (.ass)
    Ass,
    /// WebVTT format (.vtt)
    Vtt,
    /// SubStation Alpha (.ssa)
    Ssa,
}

impl SubtitleFormat {
    /// Get file extension for format.
    pub fn extension(&self) -> &str {
        match self {
            SubtitleFormat::Srt => "srt",
            SubtitleFormat::Ass => "ass",
            SubtitleFormat::Vtt => "vtt",
            SubtitleFormat::Ssa => "ssa",
        }
    }

    /// Detect format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "srt" => Some(SubtitleFormat::Srt),
            "ass" => Some(SubtitleFormat::Ass),
            "vtt" | "webvtt" => Some(SubtitleFormat::Vtt),
            "ssa" => Some(SubtitleFormat::Ssa),
            _ => None,
        }
    }
}

/// Subtitle styling options (for burned-in subtitles).
#[derive(Debug, Clone)]
pub struct SubtitleStyle {
    /// Font family.
    pub font: String,
    /// Font size.
    pub font_size: u32,
    /// Font color (hex, e.g., "FFFFFF" for white).
    pub color: String,
    /// Outline color.
    pub outline_color: String,
    /// Outline width.
    pub outline_width: u32,
    /// Shadow offset.
    pub shadow: u32,
    /// Bold text.
    pub bold: bool,
    /// Margin from bottom (pixels).
    pub margin_bottom: u32,
}

impl Default for SubtitleStyle {
    fn default() -> Self {
        Self {
            font: "Arial".to_string(),
            font_size: 24,
            color: "FFFFFF".to_string(),
            outline_color: "000000".to_string(),
            outline_width: 2,
            shadow: 1,
            bold: false,
            margin_bottom: 20,
        }
    }
}

/// Add subtitles to video (burn-in/hardcode).
///
/// # Arguments
/// * `video` - Path to input video
/// * `subtitles` - Path to subtitle file (.srt, .ass, .vtt)
/// * `output` - Path for output video
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::burn_subtitles;
///
/// burn_subtitles("video.mp4", "subs.srt", "video_with_subs.mp4").unwrap();
/// ```
pub fn burn_subtitles<P: AsRef<Path>>(video: P, subtitles: P, output: P) -> Result<ToolOutput> {
    burn_subtitles_with_style(video, subtitles, output, SubtitleStyle::default())
}

/// Burn subtitles with custom styling.
pub fn burn_subtitles_with_style<P: AsRef<Path>>(
    video: P,
    subtitles: P,
    output: P,
    style: SubtitleStyle,
) -> Result<ToolOutput> {
    let video_path = video.as_ref();
    let subtitles_path = subtitles.as_ref();
    let output_path = output.as_ref();

    if !video_path.exists() {
        return Err(DxError::FileIo {
            path: video_path.to_path_buf(),
            message: "Video file not found".to_string(),
            source: None,
        });
    }

    if !subtitles_path.exists() {
        return Err(DxError::FileIo {
            path: subtitles_path.to_path_buf(),
            message: "Subtitle file not found".to_string(),
            source: None,
        });
    }

    // Escape path for FFmpeg filter (Windows paths need special handling)
    let sub_path_str = subtitles_path
        .to_string_lossy()
        .replace('\\', "/")
        .replace(':', "\\:");

    // Build subtitle filter
    let filter = format!(
        "subtitles='{}':force_style='FontName={},FontSize={},PrimaryColour=&H{},OutlineColour=&H{},Outline={},Shadow={},Bold={},MarginV={}'",
        sub_path_str,
        style.font,
        style.font_size,
        style.color,
        style.outline_color,
        style.outline_width,
        style.shadow,
        i32::from(style.bold),
        style.margin_bottom
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(video_path)
        .arg("-vf")
        .arg(&filter)
        .arg("-c:a")
        .arg("copy")
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg("18")
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Subtitle burn-in failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Burned subtitles into video",
        output_path,
    ))
}

/// Add soft subtitles (as separate stream, can be toggled).
pub fn add_soft_subtitles<P: AsRef<Path>>(
    video: P,
    subtitles: P,
    output: P,
    language: Option<&str>,
) -> Result<ToolOutput> {
    let video_path = video.as_ref();
    let subtitles_path = subtitles.as_ref();
    let output_path = output.as_ref();

    if !video_path.exists() {
        return Err(DxError::FileIo {
            path: video_path.to_path_buf(),
            message: "Video file not found".to_string(),
            source: None,
        });
    }

    if !subtitles_path.exists() {
        return Err(DxError::FileIo {
            path: subtitles_path.to_path_buf(),
            message: "Subtitle file not found".to_string(),
            source: None,
        });
    }

    let lang = language.unwrap_or("eng");

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(video_path)
        .arg("-i")
        .arg(subtitles_path)
        .arg("-c")
        .arg("copy")
        .arg("-c:s")
        .arg("mov_text") // For MP4 compatibility
        .arg("-metadata:s:s:0")
        .arg(format!("language={}", lang))
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Soft subtitle addition failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Added soft subtitles",
        output_path,
    ))
}

/// Extract subtitles from video.
///
/// # Arguments
/// * `input` - Path to input video with embedded subtitles
/// * `output` - Path for extracted subtitle file
/// * `stream_index` - Subtitle stream index (0 for first)
pub fn extract_subtitles<P: AsRef<Path>>(
    input: P,
    output: P,
    stream_index: u32,
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

    let map_arg = format!("0:s:{}", stream_index);

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-map")
        .arg(&map_arg)
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Subtitle extraction failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Extracted subtitle stream {}", stream_index),
        output_path,
    ))
}

/// Convert subtitle format.
pub fn convert_subtitles<P: AsRef<Path>>(
    input: P,
    output: P,
    format: SubtitleFormat,
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
    cmd.arg("-y").arg("-i").arg(input_path);

    // Set output codec based on format
    match format {
        SubtitleFormat::Srt => cmd.arg("-c:s").arg("srt"),
        SubtitleFormat::Ass => cmd.arg("-c:s").arg("ass"),
        SubtitleFormat::Vtt => cmd.arg("-c:s").arg("webvtt"),
        SubtitleFormat::Ssa => cmd.arg("-c:s").arg("ssa"),
    };

    cmd.arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Subtitle conversion failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Converted subtitles to {}", format.extension()),
        output_path,
    ))
}

/// Remove all subtitle streams from video.
pub fn remove_subtitles<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
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
        .arg("-i")
        .arg(input_path)
        .arg("-sn") // No subtitles
        .arg("-c:v")
        .arg("copy")
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
                "Subtitle removal failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Removed subtitle streams",
        output_path,
    ))
}

/// Shift subtitle timing.
pub fn shift_subtitles<P: AsRef<Path>>(input: P, output: P, offset_ms: i64) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    // Read subtitle content
    let content = std::fs::read_to_string(input_path).map_err(|e| DxError::FileIo {
        path: input_path.to_path_buf(),
        message: format!("Failed to read subtitle file: {}", e),
        source: None,
    })?;

    // Detect format and shift timestamps
    let shifted = shift_srt_timestamps(&content, offset_ms);

    std::fs::write(output_path, shifted).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to write subtitle file: {}", e),
        source: None,
    })?;

    let direction = if offset_ms >= 0 {
        "forward"
    } else {
        "backward"
    };
    Ok(ToolOutput::success_with_path(
        format!("Shifted subtitles {}ms {}", offset_ms.abs(), direction),
        output_path,
    ))
}

/// Helper to shift SRT timestamps.
fn shift_srt_timestamps(content: &str, offset_ms: i64) -> String {
    let timestamp_regex = regex::Regex::new(r"(\d{2}):(\d{2}):(\d{2}),(\d{3})").unwrap();

    timestamp_regex
        .replace_all(content, |caps: &regex::Captures| {
            let hours: i64 = caps[1].parse().unwrap_or(0);
            let minutes: i64 = caps[2].parse().unwrap_or(0);
            let seconds: i64 = caps[3].parse().unwrap_or(0);
            let millis: i64 = caps[4].parse().unwrap_or(0);

            let total_ms = hours * 3600000 + minutes * 60000 + seconds * 1000 + millis + offset_ms;
            let total_ms = total_ms.max(0);

            let new_hours = total_ms / 3600000;
            let new_minutes = (total_ms % 3600000) / 60000;
            let new_seconds = (total_ms % 60000) / 1000;
            let new_millis = total_ms % 1000;

            format!(
                "{:02}:{:02}:{:02},{:03}",
                new_hours, new_minutes, new_seconds, new_millis
            )
        })
        .to_string()
}

/// List subtitle streams in a video.
pub fn list_subtitle_streams<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    let mut cmd = crate::tools::ffprobe::command();
    cmd.arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("stream=index,codec_name:stream_tags=language,title")
        .arg("-select_streams")
        .arg("s")
        .arg("-of")
        .arg("compact=p=0:nk=1")
        .arg(input_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run ffprobe: {}", e),
        source: None,
    })?;
    crate::tools::ffprobe::ensure_success(&output_result, "listing subtitle streams")?;

    let streams = String::from_utf8_lossy(&output_result.stdout).to_string();
    let stream_count = streams.lines().count();

    let mut output = ToolOutput::success(format!("Found {} subtitle streams", stream_count));
    output.metadata.insert("streams".to_string(), streams);

    Ok(output)
}

/// Batch burn subtitles into multiple videos.
pub fn batch_burn_subtitles<P: AsRef<Path>>(
    videos: &[(P, P)], // (video, subtitle) pairs
    output_dir: P,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut processed = Vec::new();

    for (video, subtitles) in videos {
        let video_path = video.as_ref();
        let file_name = video_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("output.mp4");
        let output_path = output_dir.join(format!("sub_{}", file_name));

        if burn_subtitles(video_path, subtitles.as_ref(), &output_path).is_ok() {
            processed.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Subtitled {} videos", processed.len())).with_paths(processed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtitle_format() {
        assert_eq!(SubtitleFormat::Srt.extension(), "srt");
        assert_eq!(
            SubtitleFormat::from_extension("vtt"),
            Some(SubtitleFormat::Vtt)
        );
        assert_eq!(SubtitleFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_shift_timestamps() {
        let srt = "1\n00:00:01,000 --> 00:00:03,000\nHello";
        let shifted = shift_srt_timestamps(srt, 1000);
        assert!(shifted.contains("00:00:02,000"));
    }
}
