//! Audio merger.
//!
//! Combine multiple audio files into one.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Merge method.
#[derive(Debug, Clone, Copy, Default)]
pub enum MergeMethod {
    /// Concatenate sequentially (one after another).
    #[default]
    Concatenate,
    /// Mix together (play simultaneously).
    Mix,
    /// Crossfade between tracks.
    Crossfade,
}

/// Merge options.
#[derive(Debug, Clone)]
pub struct MergeOptions {
    /// How to combine the audio files.
    pub method: MergeMethod,
    /// Crossfade duration in seconds (for Crossfade method).
    pub crossfade_duration: f64,
    /// Normalize all inputs to same volume before merging.
    pub normalize: bool,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            method: MergeMethod::Concatenate,
            crossfade_duration: 2.0,
            normalize: false,
        }
    }
}

/// Merge multiple audio files.
///
/// # Arguments
/// * `inputs` - Paths to audio files to merge
/// * `output` - Path for merged output
///
/// # Example
/// ```no_run
/// use dx_media::tools::audio::merge_audio;
///
/// merge_audio(&["track1.mp3", "track2.mp3", "track3.mp3"], "combined.mp3").unwrap();
/// ```
pub fn merge_audio<P: AsRef<Path>>(inputs: &[P], output: P) -> Result<ToolOutput> {
    merge_audio_with_options(inputs, output, MergeOptions::default())
}

/// Merge with options.
pub fn merge_audio_with_options<P: AsRef<Path>>(
    inputs: &[P],
    output: P,
    options: MergeOptions,
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
        MergeMethod::Concatenate => concatenate_audio(inputs, output_path),
        MergeMethod::Mix => mix_audio(inputs, output_path),
        MergeMethod::Crossfade => crossfade_audio(inputs, output_path, options.crossfade_duration),
    }
}

/// Concatenate audio files sequentially.
fn concatenate_audio<P: AsRef<Path>>(inputs: &[P], output: &Path) -> Result<ToolOutput> {
    // Create concat demuxer file list
    let temp_dir = std::env::temp_dir();
    let list_path = temp_dir.join(format!("concat_list_{}.txt", std::process::id()));

    let list_content: String = inputs
        .iter()
        .map(|p| {
            format!(
                "file '{}'",
                p.as_ref().to_string_lossy().replace('\'', "'\\''")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(&list_path, &list_content).map_err(|e| DxError::FileIo {
        path: list_path.clone(),
        message: format!("Failed to write concat list: {}", e),
        source: None,
    })?;

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&list_path)
        .arg("-c")
        .arg("copy")
        .arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    // Clean up temp file
    let _ = std::fs::remove_file(&list_path);

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Concatenation failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Concatenated {} audio files", inputs.len()),
        output,
    ))
}

/// Mix audio files together (play simultaneously).
fn mix_audio<P: AsRef<Path>>(inputs: &[P], output: &Path) -> Result<ToolOutput> {
    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y");

    // Add all inputs
    for input in inputs {
        cmd.arg("-i").arg(input.as_ref());
    }

    // Build amix filter
    let filter = format!(
        "amix=inputs={}:duration=longest:dropout_transition=2",
        inputs.len()
    );

    cmd.arg("-filter_complex").arg(&filter).arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Audio mixing failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Mixed {} audio files", inputs.len()),
        output,
    ))
}

/// Crossfade between audio files.
fn crossfade_audio<P: AsRef<Path>>(
    inputs: &[P],
    output: &Path,
    fade_duration: f64,
) -> Result<ToolOutput> {
    if inputs.len() < 2 {
        return concatenate_audio(inputs, output);
    }

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y");

    // Add all inputs
    for input in inputs {
        cmd.arg("-i").arg(input.as_ref());
    }

    // Build crossfade filter chain
    let mut filter_parts = Vec::new();
    let mut last_output = "[0:a]".to_string();

    for i in 1..inputs.len() {
        let output_label = if i == inputs.len() - 1 {
            "[out]".to_string()
        } else {
            format!("[a{}]", i)
        };

        filter_parts.push(format!(
            "{}[{}:a]acrossfade=d={}:c1=tri:c2=tri{}",
            last_output, i, fade_duration, output_label
        ));

        last_output = output_label;
    }

    let filter = filter_parts.join(";");

    cmd.arg("-filter_complex")
        .arg(&filter)
        .arg("-map")
        .arg("[out]")
        .arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Crossfade failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!(
            "Crossfaded {} audio files ({}s transitions)",
            inputs.len(),
            fade_duration
        ),
        output,
    ))
}

/// Overlay audio on top of another (e.g., voice over music).
pub fn overlay_audio<P: AsRef<Path>>(
    background: P,
    overlay: P,
    output: P,
    overlay_volume: f32,
) -> Result<ToolOutput> {
    let bg_path = background.as_ref();
    let ov_path = overlay.as_ref();
    let output_path = output.as_ref();

    // Lower background volume when overlay is present
    let filter = format!(
        "[0:a]volume=0.3[bg];[1:a]volume={}[ov];[bg][ov]amix=inputs=2:duration=first[out]",
        overlay_volume
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(bg_path)
        .arg("-i")
        .arg(ov_path)
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-map")
        .arg("[out]")
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Audio overlay failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Overlaid audio tracks",
        output_path,
    ))
}

/// Append silence to audio.
pub fn append_silence<P: AsRef<Path>>(input: P, output: P, duration: f64) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let filter = format!("[0:a]apad=pad_dur={}[out]", duration);

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-map")
        .arg("[out]")
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Appending silence failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Appended {:.1}s of silence", duration),
        output_path,
    ))
}

/// Prepend silence to audio.
pub fn prepend_silence<P: AsRef<Path>>(input: P, output: P, duration: f64) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let filter = format!("adelay={}s:all=1", duration);

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-af")
        .arg(&filter)
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Prepending silence failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Prepended {:.1}s of silence", duration),
        output_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_options() {
        let opts = MergeOptions::default();
        assert!(matches!(opts.method, MergeMethod::Concatenate));
        assert_eq!(opts.crossfade_duration, 2.0);
    }
}
