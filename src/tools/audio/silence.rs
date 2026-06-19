//! Silence detection and removal.
//!
//! Detect and remove silent parts from audio.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Silence detection options.
#[derive(Debug, Clone)]
pub struct SilenceOptions {
    /// Noise floor in dB (below this is considered silence).
    pub threshold_db: f32,
    /// Minimum silence duration in seconds.
    pub min_duration: f64,
    /// Keep some silence at start/end of segments.
    pub padding: f64,
}

impl Default for SilenceOptions {
    fn default() -> Self {
        Self {
            threshold_db: -50.0,
            min_duration: 0.5,
            padding: 0.1,
        }
    }
}

impl SilenceOptions {
    /// Aggressive silence removal (lower threshold).
    pub fn aggressive() -> Self {
        Self {
            threshold_db: -40.0,
            min_duration: 0.3,
            padding: 0.05,
        }
    }

    /// Conservative silence removal (higher threshold).
    pub fn conservative() -> Self {
        Self {
            threshold_db: -60.0,
            min_duration: 1.0,
            padding: 0.2,
        }
    }
}

/// Silence segment detected in audio.
#[derive(Debug, Clone)]
pub struct SilenceSegment {
    /// Start time in seconds.
    pub start: f64,
    /// End time in seconds.
    pub end: f64,
    /// Duration in seconds.
    pub duration: f64,
}

/// Remove silence from audio.
///
/// # Arguments
/// * `input` - Path to input audio file
/// * `output` - Path for output without silence
/// * `options` - Silence detection options
///
/// # Example
/// ```no_run
/// use dx_media::tools::audio::{remove_silence, SilenceOptions};
///
/// remove_silence("recording.wav", "cleaned.wav", SilenceOptions::default()).unwrap();
/// ```
pub fn remove_silence<P: AsRef<Path>>(
    input: P,
    output: P,
    options: SilenceOptions,
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

    // Use silenceremove filter
    let filter = format!(
        "silenceremove=start_periods=1:start_duration=0:start_threshold={}dB:detection=peak,silenceremove=stop_periods=-1:stop_duration={}:stop_threshold={}dB:detection=peak",
        options.threshold_db, options.min_duration, options.threshold_db
    );

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
                "Silence removal failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    // Calculate size reduction
    let input_size = std::fs::metadata(input_path).map_or(0, |m| m.len());
    let output_size = std::fs::metadata(output_path).map_or(0, |m| m.len());
    let reduction = if input_size > 0 {
        100.0 - (output_size as f64 / input_size as f64 * 100.0)
    } else {
        0.0
    };

    Ok(ToolOutput::success_with_path(
        format!("Removed silence (size reduced by {:.1}%)", reduction),
        output_path,
    ))
}

/// Detect silence segments in audio.
pub fn detect_silence<P: AsRef<Path>>(
    input: P,
    options: SilenceOptions,
) -> Result<Vec<SilenceSegment>> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    let filter = format!(
        "silencedetect=noise={}dB:d={}",
        options.threshold_db, options.min_duration
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-i")
        .arg(input_path)
        .arg("-af")
        .arg(&filter)
        .arg("-f")
        .arg("null")
        .arg("-");

    let output = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse silence detection output
    let mut segments = Vec::new();
    let mut current_start: Option<f64> = None;

    for line in stderr.lines() {
        if line.contains("silence_start:") {
            if let Some(time_str) = line.split("silence_start:").nth(1) {
                if let Ok(time) = time_str
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse::<f64>()
                {
                    current_start = Some(time);
                }
            }
        } else if line.contains("silence_end:") {
            if let Some(start) = current_start {
                if let Some(time_str) = line.split("silence_end:").nth(1) {
                    if let Ok(end) = time_str
                        .split_whitespace()
                        .next()
                        .unwrap_or("0")
                        .parse::<f64>()
                    {
                        segments.push(SilenceSegment {
                            start,
                            end,
                            duration: end - start,
                        });
                    }
                }
                current_start = None;
            }
        }
    }

    Ok(segments)
}

/// Trim leading silence from audio.
pub fn trim_leading_silence<P: AsRef<Path>>(
    input: P,
    output: P,
    options: SilenceOptions,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let filter = format!(
        "silenceremove=start_periods=1:start_threshold={}dB:detection=peak",
        options.threshold_db
    );

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
                "Trimming leading silence failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Trimmed leading silence",
        output_path,
    ))
}

/// Trim trailing silence from audio.
pub fn trim_trailing_silence<P: AsRef<Path>>(
    input: P,
    output: P,
    options: SilenceOptions,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Reverse, trim leading, reverse again
    let filter = format!(
        "areverse,silenceremove=start_periods=1:start_threshold={}dB:detection=peak,areverse",
        options.threshold_db
    );

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
                "Trimming trailing silence failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Trimmed trailing silence",
        output_path,
    ))
}

/// Add silence to start or end of audio.
pub fn add_silence<P: AsRef<Path>>(
    input: P,
    output: P,
    duration: f64,
    at_start: bool,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let filter = if at_start {
        format!("adelay={}s:all=1", duration)
    } else {
        format!("apad=pad_dur={}", duration)
    };

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
                "Adding silence failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    let position = if at_start { "start" } else { "end" };
    Ok(ToolOutput::success_with_path(
        format!("Added {:.1}s silence at {}", duration, position),
        output_path,
    ))
}

/// Calculate total silence duration in audio.
pub fn total_silence_duration<P: AsRef<Path>>(input: P, options: SilenceOptions) -> Result<f64> {
    let segments = detect_silence(input, options)?;
    Ok(segments.iter().map(|s| s.duration).sum())
}

/// Generate silence audio file.
pub fn generate_silence<P: AsRef<Path>>(
    output: P,
    duration: f64,
    sample_rate: u32,
) -> Result<ToolOutput> {
    let output_path = output.as_ref();

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-f")
        .arg("lavfi")
        .arg("-i")
        .arg(format!("anullsrc=r={}:cl=stereo", sample_rate))
        .arg("-t")
        .arg(format!("{:.3}", duration))
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Silence generation failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Generated {:.1}s of silence", duration),
        output_path,
    ))
}

/// Batch remove silence from multiple files.
pub fn batch_remove_silence<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    options: SilenceOptions,
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
            .unwrap_or("audio.mp3");
        let output_path = output_dir.join(format!("clean_{}", file_name));

        if remove_silence(input_path, &output_path, options.clone()).is_ok() {
            processed.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Processed {} files", processed.len())).with_paths(processed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_options() {
        let default = SilenceOptions::default();
        assert_eq!(default.threshold_db, -50.0);

        let aggressive = SilenceOptions::aggressive();
        assert!(aggressive.threshold_db > default.threshold_db);
    }
}
