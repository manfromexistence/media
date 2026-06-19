//! Audio format converter.
//!
//! Convert between audio formats (MP3, WAV, FLAC, OGG, AAC, etc.)

use crate::deps::check_tool_dependency;
use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use crate::tools::receipts::local_dependency_output;
use crate::types::MediaType;
use std::path::Path;

/// Supported audio formats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioOutputFormat {
    /// MP3 audio format - lossy compression, widely supported.
    Mp3,
    /// WAV audio format - uncompressed PCM audio, high quality.
    Wav,
    /// FLAC audio format - lossless compression, audiophile quality.
    Flac,
    /// OGG audio format - open container with Vorbis codec.
    Ogg,
    /// AAC audio format - Advanced Audio Coding, efficient compression.
    Aac,
    /// M4A audio format - MPEG-4 audio container.
    M4a,
    /// WMA audio format - Windows Media Audio.
    Wma,
    /// Opus audio format - modern codec, excellent for voice and music.
    Opus,
}

impl AudioOutputFormat {
    /// Get file extension.
    pub fn extension(&self) -> &str {
        match self {
            AudioOutputFormat::Mp3 => "mp3",
            AudioOutputFormat::Wav => "wav",
            AudioOutputFormat::Flac => "flac",
            AudioOutputFormat::Ogg => "ogg",
            AudioOutputFormat::Aac => "aac",
            AudioOutputFormat::M4a => "m4a",
            AudioOutputFormat::Wma => "wma",
            AudioOutputFormat::Opus => "opus",
        }
    }

    /// Get FFmpeg codec name.
    pub fn codec(&self) -> &str {
        match self {
            AudioOutputFormat::Mp3 => "libmp3lame",
            AudioOutputFormat::Wav => "pcm_s16le",
            AudioOutputFormat::Flac => "flac",
            AudioOutputFormat::Ogg => "libvorbis",
            AudioOutputFormat::Aac => "aac",
            AudioOutputFormat::M4a => "aac",
            AudioOutputFormat::Wma => "wmav2",
            AudioOutputFormat::Opus => "libopus",
        }
    }

    /// Detect from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(AudioOutputFormat::Mp3),
            "wav" => Some(AudioOutputFormat::Wav),
            "flac" => Some(AudioOutputFormat::Flac),
            "ogg" => Some(AudioOutputFormat::Ogg),
            "aac" => Some(AudioOutputFormat::Aac),
            "m4a" => Some(AudioOutputFormat::M4a),
            "wma" => Some(AudioOutputFormat::Wma),
            "opus" => Some(AudioOutputFormat::Opus),
            _ => None,
        }
    }
}

/// Audio conversion options.
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// Output format.
    pub format: AudioOutputFormat,
    /// Bitrate in kbps (e.g., 128, 192, 320).
    pub bitrate: Option<u32>,
    /// Sample rate in Hz (e.g., 44100, 48000).
    pub sample_rate: Option<u32>,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: Option<u8>,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            format: AudioOutputFormat::Mp3,
            bitrate: Some(192),
            sample_rate: None,
            channels: None,
        }
    }
}

impl ConvertOptions {
    /// Create MP3 conversion options.
    pub fn mp3(bitrate: u32) -> Self {
        Self {
            format: AudioOutputFormat::Mp3,
            bitrate: Some(bitrate),
            sample_rate: None,
            channels: None,
        }
    }

    /// Create lossless FLAC options.
    pub fn flac() -> Self {
        Self {
            format: AudioOutputFormat::Flac,
            bitrate: None,
            sample_rate: None,
            channels: None,
        }
    }

    /// Create WAV options.
    pub fn wav(sample_rate: u32) -> Self {
        Self {
            format: AudioOutputFormat::Wav,
            bitrate: None,
            sample_rate: Some(sample_rate),
            channels: None,
        }
    }
}

/// Convert audio file to different format.
///
/// # Arguments
/// * `input` - Path to input audio file
/// * `output` - Path for output file
/// * `options` - Conversion options
///
/// # Errors
///
/// Returns `DxError::MissingDependency` if FFmpeg is not installed.
///
/// # Example
/// ```no_run
/// use dx_media::tools::audio::{convert_audio, ConvertOptions, AudioOutputFormat};
///
/// // Convert to MP3 at 320kbps
/// let options = ConvertOptions::mp3(320);
/// convert_audio("song.wav", "song.mp3", options).unwrap();
/// ```
pub fn convert_audio<P: AsRef<Path>>(
    input: P,
    output: P,
    options: ConvertOptions,
) -> Result<ToolOutput> {
    // Check for FFmpeg dependency
    check_tool_dependency("audio::convert")?;

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
        .arg("-c:a")
        .arg(options.format.codec());

    if let Some(bitrate) = options.bitrate {
        cmd.arg("-b:a").arg(format!("{}k", bitrate));
    }

    if let Some(sample_rate) = options.sample_rate {
        cmd.arg("-ar").arg(sample_rate.to_string());
    }

    if let Some(channels) = options.channels {
        cmd.arg("-ac").arg(channels.to_string());
    }

    cmd.arg(output_path);

    let output_result = cmd
        .output()
        .map_err(|e| crate::tools::ffmpeg::run_failed(e, "converting audio"))?;
    crate::tools::ffmpeg::ensure_success(&output_result, "converting audio")?;

    let output_size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    local_dependency_output(
        "audio.convert",
        "ffmpeg",
        input_path,
        output_path,
        MediaType::Audio,
        format!(
            "Converted to {} ({} bytes)",
            options.format.extension(),
            output_size
        ),
    )
}

/// Convert to MP3 with default settings.
pub fn to_mp3<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    convert_audio(input, output, ConvertOptions::mp3(192))
}

/// Convert to WAV (lossless).
pub fn to_wav<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    convert_audio(input, output, ConvertOptions::wav(44100))
}

/// Convert to FLAC (lossless).
pub fn to_flac<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    convert_audio(input, output, ConvertOptions::flac())
}

/// Batch convert multiple audio files.
pub fn batch_convert<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    options: ConvertOptions,
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
            .unwrap_or("audio");
        let output_path = output_dir.join(format!("{}.{}", file_stem, options.format.extension()));

        if convert_audio(input_path, &output_path, options.clone()).is_ok() {
            converted.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Converted {} files", converted.len())).with_paths(converted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format() {
        assert_eq!(AudioOutputFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioOutputFormat::Mp3.codec(), "libmp3lame");
        assert_eq!(
            AudioOutputFormat::from_extension("flac"),
            Some(AudioOutputFormat::Flac)
        );
    }

    #[test]
    fn test_convert_options() {
        let mp3 = ConvertOptions::mp3(320);
        assert_eq!(mp3.bitrate, Some(320));

        let flac = ConvertOptions::flac();
        assert!(flac.bitrate.is_none());
    }
}
