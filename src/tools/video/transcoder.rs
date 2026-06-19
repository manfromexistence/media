//! Video format transcoding tool.
//!
//! Convert videos between formats using FFmpeg.

use crate::deps::check_tool_dependency;
use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use crate::tools::receipts::local_dependency_output;
use crate::types::MediaType;
use std::path::Path;

/// Video output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    /// MP4 (H.264/AAC) - Most compatible.
    Mp4,
    /// WebM (VP9/Opus) - Web optimized.
    WebM,
    /// MKV (Matroska) - Container format.
    Mkv,
    /// AVI - Legacy format.
    Avi,
    /// MOV - QuickTime format.
    Mov,
    /// GIF - Animated image.
    Gif,
}

impl VideoFormat {
    /// Get file extension for format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::WebM => "webm",
            Self::Mkv => "mkv",
            Self::Avi => "avi",
            Self::Mov => "mov",
            Self::Gif => "gif",
        }
    }

    /// Get FFmpeg codec arguments.
    pub fn codec_args(&self) -> Vec<&'static str> {
        match self {
            Self::Mp4 => vec!["-c:v", "libx264", "-c:a", "aac"],
            Self::WebM => vec!["-c:v", "libvpx-vp9", "-c:a", "libopus"],
            Self::Mkv => vec!["-c:v", "libx264", "-c:a", "aac"],
            Self::Avi => vec!["-c:v", "mpeg4", "-c:a", "mp3"],
            Self::Mov => vec!["-c:v", "libx264", "-c:a", "aac"],
            Self::Gif => vec![],
        }
    }

    /// Parse format from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mp4" => Some(Self::Mp4),
            "webm" => Some(Self::WebM),
            "mkv" => Some(Self::Mkv),
            "avi" => Some(Self::Avi),
            "mov" => Some(Self::Mov),
            "gif" => Some(Self::Gif),
            _ => None,
        }
    }
}

/// Video quality presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoQuality {
    /// Low quality, smallest file size.
    Low,
    /// Medium quality, balanced.
    #[default]
    Medium,
    /// High quality, larger file size.
    High,
    /// Very high quality.
    VeryHigh,
    /// Lossless (where supported).
    Lossless,
}

impl VideoQuality {
    /// Get CRF value for x264/x265 (lower = better quality).
    pub fn crf(&self) -> u8 {
        match self {
            Self::Low => 28,
            Self::Medium => 23,
            Self::High => 18,
            Self::VeryHigh => 15,
            Self::Lossless => 0,
        }
    }
}

/// Transcoding options.
#[derive(Debug, Clone)]
pub struct TranscodeOptions {
    /// Output format.
    pub format: VideoFormat,
    /// Quality preset.
    pub quality: VideoQuality,
    /// Target bitrate (optional, overrides quality).
    pub bitrate: Option<String>,
    /// Audio bitrate.
    pub audio_bitrate: Option<String>,
    /// Frame rate (fps).
    pub fps: Option<f32>,
    /// Whether to strip audio.
    pub no_audio: bool,
    /// Additional FFmpeg arguments.
    pub extra_args: Vec<String>,
}

impl Default for TranscodeOptions {
    fn default() -> Self {
        Self {
            format: VideoFormat::Mp4,
            quality: VideoQuality::Medium,
            bitrate: None,
            audio_bitrate: Some("128k".to_string()),
            fps: None,
            no_audio: false,
            extra_args: Vec::new(),
        }
    }
}

impl TranscodeOptions {
    /// Create options for specific format.
    pub fn new(format: VideoFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    /// Set quality level.
    pub fn with_quality(mut self, quality: VideoQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Set target bitrate (e.g., "2M", "5000k").
    pub fn with_bitrate(mut self, bitrate: &str) -> Self {
        self.bitrate = Some(bitrate.to_string());
        self
    }

    /// Set frame rate.
    pub fn with_fps(mut self, fps: f32) -> Self {
        self.fps = Some(fps);
        self
    }

    /// Strip audio track.
    pub fn without_audio(mut self) -> Self {
        self.no_audio = true;
        self
    }
}

/// Transcode a video to a different format.
///
/// Requires FFmpeg to be installed and available in PATH.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for output video
/// * `options` - Transcoding options
///
/// # Errors
///
/// Returns `DxError::MissingDependency` if FFmpeg is not installed.
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::{transcode_video, TranscodeOptions, VideoFormat};
///
/// // Convert MKV to MP4
/// let opts = TranscodeOptions::new(VideoFormat::Mp4);
/// transcode_video("input.mkv", "output.mp4", opts).unwrap();
/// ```
pub fn transcode_video<P: AsRef<Path>>(
    input: P,
    output: P,
    options: TranscodeOptions,
) -> Result<ToolOutput> {
    // Check for FFmpeg dependency
    check_tool_dependency("video::transcode")?;

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
    cmd.arg("-y") // Overwrite output
        .arg("-i")
        .arg(input_path);

    // Add codec arguments
    for arg in options.format.codec_args() {
        cmd.arg(arg);
    }

    // Quality/bitrate settings
    if let Some(bitrate) = &options.bitrate {
        cmd.arg("-b:v").arg(bitrate);
    } else if options.format != VideoFormat::Gif {
        cmd.arg("-crf").arg(options.quality.crf().to_string());
    }

    // Audio settings
    if options.no_audio {
        cmd.arg("-an");
    } else if let Some(audio_bitrate) = &options.audio_bitrate {
        cmd.arg("-b:a").arg(audio_bitrate);
    }

    // Frame rate
    if let Some(fps) = options.fps {
        cmd.arg("-r").arg(fps.to_string());
    }

    // Extra arguments
    for arg in &options.extra_args {
        cmd.arg(arg);
    }

    cmd.arg(output_path);

    let output = cmd
        .output()
        .map_err(|e| crate::tools::ffmpeg::run_failed(e, "transcoding video"))?;
    crate::tools::ffmpeg::ensure_success(&output, "transcoding video")?;

    let output_size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    let expected_type = if options.format == VideoFormat::Gif {
        MediaType::Gif
    } else {
        MediaType::Video
    };

    local_dependency_output(
        "video.transcode",
        "ffmpeg",
        input_path,
        output_path,
        expected_type,
        format!(
            "Transcoded to {} format ({} bytes)",
            options.format.extension(),
            output_size
        ),
    )
}

/// Quick convert to MP4 format.
pub fn to_mp4<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    transcode_video(input, output, TranscodeOptions::new(VideoFormat::Mp4))
}

/// Quick convert to WebM format.
pub fn to_webm<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    transcode_video(input, output, TranscodeOptions::new(VideoFormat::WebM))
}

/// Batch transcode multiple videos.
pub fn batch_transcode<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    options: TranscodeOptions,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut converted = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = output_dir.join(format!("{}.{}", file_stem, options.format.extension()));

        if transcode_video(input_path, &output_path, options.clone()).is_ok() {
            converted.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Converted {} videos", converted.len())).with_paths(converted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_format() {
        assert_eq!(VideoFormat::Mp4.extension(), "mp4");
        assert_eq!(VideoFormat::WebM.extension(), "webm");
        assert_eq!(VideoFormat::from_str("mp4"), Some(VideoFormat::Mp4));
    }

    #[test]
    fn test_quality_crf() {
        assert!(VideoQuality::High.crf() < VideoQuality::Low.crf());
    }
}
