//! Audio extraction from video files.
//!
//! Extract audio tracks from video files in various formats.

use crate::deps::check_tool_dependency;
use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use crate::tools::receipts::local_dependency_output;
use crate::types::MediaType;
use std::path::Path;

/// Audio output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioFormat {
    /// MP3 format (lossy, most compatible).
    #[default]
    Mp3,
    /// AAC format (lossy, good quality).
    Aac,
    /// WAV format (uncompressed).
    Wav,
    /// FLAC format (lossless compressed).
    Flac,
    /// OGG Vorbis format.
    Ogg,
    /// M4A format (AAC in MP4 container).
    M4a,
    /// Opus format (modern, efficient).
    Opus,
}

impl AudioFormat {
    /// Get file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Aac => "aac",
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Ogg => "ogg",
            Self::M4a => "m4a",
            Self::Opus => "opus",
        }
    }

    /// Get FFmpeg codec arguments.
    pub fn codec_args(&self) -> Vec<&'static str> {
        match self {
            Self::Mp3 => vec!["-c:a", "libmp3lame"],
            Self::Aac => vec!["-c:a", "aac"],
            Self::Wav => vec!["-c:a", "pcm_s16le"],
            Self::Flac => vec!["-c:a", "flac"],
            Self::Ogg => vec!["-c:a", "libvorbis"],
            Self::M4a => vec!["-c:a", "aac"],
            Self::Opus => vec!["-c:a", "libopus"],
        }
    }

    /// Parse format from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mp3" => Some(Self::Mp3),
            "aac" => Some(Self::Aac),
            "wav" => Some(Self::Wav),
            "flac" => Some(Self::Flac),
            "ogg" => Some(Self::Ogg),
            "m4a" => Some(Self::M4a),
            "opus" => Some(Self::Opus),
            _ => None,
        }
    }
}

/// Audio extraction options.
#[derive(Debug, Clone)]
pub struct AudioExtractOptions {
    /// Output format.
    pub format: AudioFormat,
    /// Audio bitrate (e.g., "128k", "320k").
    pub bitrate: Option<String>,
    /// Sample rate in Hz (e.g., 44100, 48000).
    pub sample_rate: Option<u32>,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: Option<u8>,
    /// Start time in seconds.
    pub start: Option<f64>,
    /// Duration in seconds.
    pub duration: Option<f64>,
}

impl Default for AudioExtractOptions {
    fn default() -> Self {
        Self {
            format: AudioFormat::Mp3,
            bitrate: Some("192k".to_string()),
            sample_rate: None,
            channels: None,
            start: None,
            duration: None,
        }
    }
}

impl AudioExtractOptions {
    /// Create options for specific format.
    pub fn new(format: AudioFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    /// Set audio bitrate.
    pub fn with_bitrate(mut self, bitrate: &str) -> Self {
        self.bitrate = Some(bitrate.to_string());
        self
    }

    /// Set sample rate.
    pub fn with_sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = Some(rate);
        self
    }

    /// Set mono output.
    pub fn mono(mut self) -> Self {
        self.channels = Some(1);
        self
    }

    /// Set stereo output.
    pub fn stereo(mut self) -> Self {
        self.channels = Some(2);
        self
    }

    /// Extract only a portion.
    pub fn with_range(mut self, start: f64, duration: f64) -> Self {
        self.start = Some(start);
        self.duration = Some(duration);
        self
    }
}

/// Extract audio from a video file.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for output audio file
/// * `format` - Audio output format
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::{extract_audio, AudioFormat};
///
/// // Extract MP3 audio from video
/// extract_audio("video.mp4", "audio.mp3", AudioFormat::Mp3).unwrap();
/// ```
pub fn extract_audio<P: AsRef<Path>>(
    input: P,
    output: P,
    format: AudioFormat,
) -> Result<ToolOutput> {
    extract_audio_with_options(input, output, AudioExtractOptions::new(format))
}

/// Extract audio with detailed options.
pub fn extract_audio_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: AudioExtractOptions,
) -> Result<ToolOutput> {
    check_tool_dependency("video::audio_extract")?;

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

    // Time range
    if let Some(start) = options.start {
        cmd.arg("-ss").arg(start.to_string());
    }
    if let Some(duration) = options.duration {
        cmd.arg("-t").arg(duration.to_string());
    }

    // No video
    cmd.arg("-vn");

    // Audio codec
    for arg in options.format.codec_args() {
        cmd.arg(arg);
    }

    // Bitrate
    if let Some(bitrate) = &options.bitrate {
        cmd.arg("-b:a").arg(bitrate);
    }

    // Sample rate
    if let Some(rate) = options.sample_rate {
        cmd.arg("-ar").arg(rate.to_string());
    }

    // Channels
    if let Some(channels) = options.channels {
        cmd.arg("-ac").arg(channels.to_string());
    }

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

    local_dependency_output(
        "video.extract-audio",
        "ffmpeg",
        input_path,
        output_path,
        MediaType::Audio,
        format!(
            "Extracted {} audio ({} bytes)",
            options.format.extension(),
            output_size
        ),
    )
}

/// Extract audio as MP3 (convenience function).
pub fn extract_mp3<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    extract_audio(input, output, AudioFormat::Mp3)
}

/// Extract audio as WAV (uncompressed).
pub fn extract_wav<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    extract_audio(input, output, AudioFormat::Wav)
}

/// Batch extract audio from multiple videos.
pub fn batch_extract_audio<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    format: AudioFormat,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut extracted = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("audio");
        let output_path = output_dir.join(format!("{}.{}", file_stem, format.extension()));

        if extract_audio(input_path, &output_path, format).is_ok() {
            extracted.push(output_path);
        }
    }

    Ok(
        ToolOutput::success(format!("Extracted audio from {} videos", extracted.len()))
            .with_paths(extracted),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format() {
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFormat::from_str("flac"), Some(AudioFormat::Flac));
    }

    #[test]
    fn test_extract_options() {
        let opts = AudioExtractOptions::new(AudioFormat::Wav)
            .mono()
            .with_sample_rate(44100);
        assert_eq!(opts.channels, Some(1));
        assert_eq!(opts.sample_rate, Some(44100));
    }
}
