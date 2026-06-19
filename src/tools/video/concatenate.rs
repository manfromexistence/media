//! Video concatenation tool.
//!
//! Join multiple video files into a single video.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Concatenation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConcatMethod {
    /// Demuxer method (fast, same codec/resolution required).
    #[default]
    Demuxer,
    /// Filter method (slower, handles different formats).
    Filter,
    /// Protocol method (for compatible formats only).
    Protocol,
}

/// Concatenation options.
#[derive(Debug, Clone)]
pub struct ConcatOptions {
    /// Concatenation method.
    pub method: ConcatMethod,
    /// Re-encode to common format.
    pub reencode: bool,
    /// Target width (if re-encoding).
    pub width: Option<u32>,
    /// Target height (if re-encoding).
    pub height: Option<u32>,
    /// Video codec.
    pub video_codec: String,
    /// Audio codec.
    pub audio_codec: String,
    /// Video quality (CRF).
    pub quality: u8,
}

impl Default for ConcatOptions {
    fn default() -> Self {
        Self {
            method: ConcatMethod::Demuxer,
            reencode: false,
            width: None,
            height: None,
            video_codec: "libx264".to_string(),
            audio_codec: "aac".to_string(),
            quality: 18,
        }
    }
}

impl ConcatOptions {
    /// Use filter method (for different formats).
    pub fn with_filter() -> Self {
        Self {
            method: ConcatMethod::Filter,
            reencode: true,
            ..Default::default()
        }
    }

    /// Enable re-encoding with common resolution.
    pub fn with_reencode(width: u32, height: u32) -> Self {
        Self {
            method: ConcatMethod::Filter,
            reencode: true,
            width: Some(width),
            height: Some(height),
            ..Default::default()
        }
    }
}

/// Concatenate multiple videos into one.
///
/// # Arguments
/// * `inputs` - List of input video paths (in order)
/// * `output` - Path for concatenated output
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::concatenate_videos;
///
/// let inputs = vec!["part1.mp4", "part2.mp4", "part3.mp4"];
/// concatenate_videos(&inputs, "combined.mp4").unwrap();
/// ```
pub fn concatenate_videos<P: AsRef<Path>>(inputs: &[P], output: P) -> Result<ToolOutput> {
    concatenate_with_options(inputs, output, ConcatOptions::default())
}

/// Concatenate videos with options.
pub fn concatenate_with_options<P: AsRef<Path>>(
    inputs: &[P],
    output: P,
    options: ConcatOptions,
) -> Result<ToolOutput> {
    if inputs.is_empty() {
        return Err(DxError::Config {
            message: "No input files provided".to_string(),
            source: None,
        });
    }

    let output_path = output.as_ref();

    // Validate all inputs exist
    for input in inputs {
        let path = input.as_ref();
        if !path.exists() {
            return Err(DxError::FileIo {
                path: path.to_path_buf(),
                message: "Input file not found".to_string(),
                source: None,
            });
        }
    }

    match options.method {
        ConcatMethod::Demuxer => concat_demuxer(inputs, output_path, &options),
        ConcatMethod::Filter => concat_filter(inputs, output_path, &options),
        ConcatMethod::Protocol => concat_protocol(inputs, output_path),
    }
}

/// Concatenate using demuxer (file list method).
fn concat_demuxer<P: AsRef<Path>>(
    inputs: &[P],
    output: &Path,
    options: &ConcatOptions,
) -> Result<ToolOutput> {
    // Create temporary file list
    let temp_dir = std::env::temp_dir();
    let list_path = temp_dir.join(format!("concat_list_{}.txt", std::process::id()));

    // Write file list
    let mut list_content = String::new();
    for input in inputs {
        let path = input.as_ref();
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        };
        // Escape single quotes in path
        let escaped = abs_path.to_string_lossy().replace('\'', "'\\''");
        list_content.push_str(&format!("file '{}'\n", escaped));
    }

    std::fs::write(&list_path, &list_content).map_err(|e| DxError::FileIo {
        path: list_path.clone(),
        message: format!("Failed to write file list: {}", e),
        source: None,
    })?;

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&list_path);

    if options.reencode {
        cmd.arg("-c:v")
            .arg(&options.video_codec)
            .arg("-crf")
            .arg(options.quality.to_string())
            .arg("-c:a")
            .arg(&options.audio_codec);
    } else {
        cmd.arg("-c").arg("copy");
    }

    cmd.arg(output);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    // Clean up temp file
    let _ = std::fs::remove_file(&list_path);

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Concatenation failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    let output_size = std::fs::metadata(output).map_or(0, |m| m.len());

    Ok(ToolOutput::success_with_path(
        format!(
            "Concatenated {} videos ({} bytes)",
            inputs.len(),
            output_size
        ),
        output,
    ))
}

/// Concatenate using filter (for different resolutions/codecs).
fn concat_filter<P: AsRef<Path>>(
    inputs: &[P],
    output: &Path,
    options: &ConcatOptions,
) -> Result<ToolOutput> {
    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y");

    // Add all inputs
    for input in inputs {
        cmd.arg("-i").arg(input.as_ref());
    }

    // Build filter complex
    let n = inputs.len();
    let mut filter_parts = Vec::new();

    // Scale all inputs to same size if specified
    if let (Some(w), Some(h)) = (options.width, options.height) {
        for i in 0..n {
            filter_parts.push(format!("[{}:v]scale={}:{},setsar=1[v{}]", i, w, h, i));
        }

        // Concatenate
        let video_streams: String = (0..n).map(|i| format!("[v{}]", i)).collect();
        let audio_streams: String = (0..n).map(|i| format!("[{}:a]", i)).collect();

        filter_parts.push(format!(
            "{}{}concat=n={}:v=1:a=1[vout][aout]",
            video_streams, audio_streams, n
        ));
    } else {
        // Simple concatenate without scaling
        let streams: String = (0..n).map(|i| format!("[{}:v][{}:a]", i, i)).collect();
        filter_parts.push(format!("{}concat=n={}:v=1:a=1[vout][aout]", streams, n));
    }

    cmd.arg("-filter_complex")
        .arg(filter_parts.join(";"))
        .arg("-map")
        .arg("[vout]")
        .arg("-map")
        .arg("[aout]")
        .arg("-c:v")
        .arg(&options.video_codec)
        .arg("-crf")
        .arg(options.quality.to_string())
        .arg("-c:a")
        .arg(&options.audio_codec)
        .arg(output);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Concatenation failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    let output_size = std::fs::metadata(output).map_or(0, |m| m.len());

    Ok(ToolOutput::success_with_path(
        format!(
            "Concatenated {} videos using filter ({} bytes)",
            inputs.len(),
            output_size
        ),
        output,
    ))
}

/// Concatenate using protocol (for specific formats like MPEG-TS).
fn concat_protocol<P: AsRef<Path>>(inputs: &[P], output: &Path) -> Result<ToolOutput> {
    // Build concat protocol string
    let input_list: Vec<String> = inputs
        .iter()
        .map(|p| p.as_ref().to_string_lossy().to_string())
        .collect();
    let concat_str = format!("concat:{}", input_list.join("|"));

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(&concat_str)
        .arg("-c")
        .arg("copy")
        .arg(output);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Concatenation failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Concatenated {} videos", inputs.len()),
        output,
    ))
}

/// Join videos with crossfade transition.
pub fn join_with_crossfade<P: AsRef<Path>>(
    inputs: &[P],
    output: P,
    crossfade_duration: f64,
) -> Result<ToolOutput> {
    if inputs.len() < 2 {
        return concatenate_videos(inputs, output);
    }

    let output_path = output.as_ref();

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y");

    // Add all inputs
    for input in inputs {
        cmd.arg("-i").arg(input.as_ref());
    }

    // Build xfade filter
    let n = inputs.len();
    let mut filter_parts = Vec::new();

    // First video stays as is
    let mut last_video = "[0:v]".to_string();
    let mut last_audio = "[0:a]".to_string();

    for i in 1..n {
        let out_v = format!("[v{}]", i);
        let out_a = format!("[a{}]", i);

        // Video crossfade
        filter_parts.push(format!(
            "{}[{}:v]xfade=transition=fade:duration={}:offset=0{}",
            last_video, i, crossfade_duration, out_v
        ));

        // Audio crossfade
        filter_parts.push(format!(
            "{}[{}:a]acrossfade=d={}{}",
            last_audio, i, crossfade_duration, out_a
        ));

        last_video = out_v;
        last_audio = out_a;
    }

    cmd.arg("-filter_complex")
        .arg(filter_parts.join(";"))
        .arg("-map")
        .arg(&last_video)
        .arg("-map")
        .arg(&last_audio)
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg("18")
        .arg("-c:a")
        .arg("aac")
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Crossfade concatenation failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Joined {} videos with {}s crossfade", n, crossfade_duration),
        output_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_options() {
        let opts = ConcatOptions::default();
        assert_eq!(opts.method, ConcatMethod::Demuxer);
        assert!(!opts.reencode);
    }

    #[test]
    fn test_concat_options_with_filter() {
        let opts = ConcatOptions::with_filter();
        assert_eq!(opts.method, ConcatMethod::Filter);
        assert!(opts.reencode);
    }
}
