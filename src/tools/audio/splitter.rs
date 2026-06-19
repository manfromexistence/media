//! Audio splitter.
//!
//! Split audio files by duration, silence, or chapters.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Split method.
#[derive(Debug, Clone)]
pub enum SplitMethod {
    /// Split at fixed time intervals.
    Duration(f64),
    /// Split at silence gaps.
    Silence {
        /// Silence threshold in decibels (e.g., -30.0 dB).
        threshold_db: f32,
        /// Minimum silence duration in seconds to trigger a split.
        min_duration: f64,
    },
    /// Split at specific timestamps.
    Timestamps(Vec<f64>),
    /// Split into N equal parts.
    EqualParts(u32),
}

/// Split options.
#[derive(Debug, Clone)]
pub struct SplitOptions {
    /// How to split the audio.
    pub method: SplitMethod,
    /// Output filename pattern (use {n} for number).
    pub pattern: String,
    /// Zero-pad the numbers.
    pub zero_pad: u32,
}

impl Default for SplitOptions {
    fn default() -> Self {
        Self {
            method: SplitMethod::Duration(60.0),
            pattern: "part_{n}".to_string(),
            zero_pad: 3,
        }
    }
}

impl SplitOptions {
    /// Split every N seconds.
    pub fn every_seconds(duration: f64) -> Self {
        Self {
            method: SplitMethod::Duration(duration),
            ..Default::default()
        }
    }

    /// Split at silence.
    pub fn at_silence(threshold_db: f32, min_duration: f64) -> Self {
        Self {
            method: SplitMethod::Silence {
                threshold_db,
                min_duration,
            },
            ..Default::default()
        }
    }

    /// Split into equal parts.
    pub fn into_parts(count: u32) -> Self {
        Self {
            method: SplitMethod::EqualParts(count),
            ..Default::default()
        }
    }
}

/// Split audio file.
///
/// # Arguments
/// * `input` - Path to input audio file
/// * `output_dir` - Directory for output segments
/// * `options` - Split options
///
/// # Example
/// ```no_run
/// use dx_media::tools::audio::{split_audio, SplitOptions};
///
/// // Split into 60 second chunks
/// split_audio("podcast.mp3", "./segments", SplitOptions::every_seconds(60.0)).unwrap();
/// ```
pub fn split_audio<P: AsRef<Path>>(
    input: P,
    output_dir: P,
    options: SplitOptions,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    match options.method {
        SplitMethod::Duration(duration) => {
            split_by_duration(input_path, output_dir, duration, &options)
        }
        SplitMethod::Silence {
            threshold_db,
            min_duration,
        } => split_by_silence(input_path, output_dir, threshold_db, min_duration, &options),
        SplitMethod::Timestamps(ref timestamps) => {
            split_at_timestamps(input_path, output_dir, timestamps, &options)
        }
        SplitMethod::EqualParts(count) => {
            split_equal_parts(input_path, output_dir, count, &options)
        }
    }
}

/// Split by fixed duration.
fn split_by_duration(
    input: &Path,
    output_dir: &Path,
    duration: f64,
    options: &SplitOptions,
) -> Result<ToolOutput> {
    let extension = input.extension().and_then(|e| e.to_str()).unwrap_or("mp3");

    let pattern = format!(
        "{}/{}.%0{}d.{}",
        output_dir.to_string_lossy(),
        options.pattern.replace("{n}", ""),
        options.zero_pad,
        extension
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-f")
        .arg("segment")
        .arg("-segment_time")
        .arg(format!("{:.3}", duration))
        .arg("-c")
        .arg("copy")
        .arg(&pattern);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Duration split failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    // Count output files
    let count = std::fs::read_dir(output_dir)
        .map(|entries| entries.filter(|e| e.is_ok()).count())
        .unwrap_or(0);

    Ok(ToolOutput::success(format!(
        "Split into {} segments of {:.0}s",
        count, duration
    )))
}

/// Split at silence gaps.
fn split_by_silence(
    input: &Path,
    output_dir: &Path,
    threshold_db: f32,
    min_duration: f64,
    options: &SplitOptions,
) -> Result<ToolOutput> {
    // First detect silence points
    let silences = super::silence::detect_silence(
        input,
        super::silence::SilenceOptions {
            threshold_db,
            min_duration,
            padding: 0.0,
        },
    )?;

    if silences.is_empty() {
        // No silence found, just copy the file
        let extension = input.extension().and_then(|e| e.to_str()).unwrap_or("mp3");
        let output_path = output_dir.join(format!(
            "{}.{}",
            options.pattern.replace("{n}", "001"),
            extension
        ));
        std::fs::copy(input, &output_path).map_err(|e| DxError::FileIo {
            path: output_path,
            message: format!("Failed to copy file: {}", e),
            source: None,
        })?;
        return Ok(ToolOutput::success("No silence found, file copied as-is"));
    }

    // Split at midpoints of silence gaps
    let timestamps: Vec<f64> = silences
        .iter()
        .map(|s| f64::midpoint(s.start, s.end))
        .collect();

    split_at_timestamps(input, output_dir, &timestamps, options)
}

/// Split at specific timestamps.
fn split_at_timestamps(
    input: &Path,
    output_dir: &Path,
    timestamps: &[f64],
    options: &SplitOptions,
) -> Result<ToolOutput> {
    let extension = input.extension().and_then(|e| e.to_str()).unwrap_or("mp3");

    // Get total duration
    let total_duration = super::trimmer::get_audio_duration(input)?;

    let mut segments = Vec::new();
    let mut start = 0.0;

    for (i, &end) in timestamps.iter().enumerate() {
        if end <= start || end > total_duration {
            continue;
        }

        let output_name = options.pattern.replace(
            "{n}",
            &format!("{:0>width$}", i + 1, width = options.zero_pad as usize),
        );
        let output_path = output_dir.join(format!("{}.{}", output_name, extension));

        let mut cmd = crate::tools::ffmpeg::command();
        cmd.arg("-y")
            .arg("-i")
            .arg(input)
            .arg("-ss")
            .arg(format!("{:.3}", start))
            .arg("-t")
            .arg(format!("{:.3}", end - start))
            .arg("-c")
            .arg("copy")
            .arg(&output_path);

        if cmd.output().map_or(false, |o| o.status.success()) {
            segments.push(output_path);
        }

        start = end;
    }

    // Final segment
    if start < total_duration {
        let output_name = options.pattern.replace(
            "{n}",
            &format!(
                "{:0>width$}",
                segments.len() + 1,
                width = options.zero_pad as usize
            ),
        );
        let output_path = output_dir.join(format!("{}.{}", output_name, extension));

        let mut cmd = crate::tools::ffmpeg::command();
        cmd.arg("-y")
            .arg("-i")
            .arg(input)
            .arg("-ss")
            .arg(format!("{:.3}", start))
            .arg("-c")
            .arg("copy")
            .arg(&output_path);

        if cmd.output().map_or(false, |o| o.status.success()) {
            segments.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Split into {} segments", segments.len())).with_paths(segments))
}

/// Split into equal parts.
fn split_equal_parts(
    input: &Path,
    output_dir: &Path,
    count: u32,
    options: &SplitOptions,
) -> Result<ToolOutput> {
    let total_duration = super::trimmer::get_audio_duration(input)?;
    let segment_duration = total_duration / count as f64;

    let timestamps: Vec<f64> = (1..count).map(|i| i as f64 * segment_duration).collect();

    split_at_timestamps(input, output_dir, &timestamps, options)
}

/// Split audio at chapter markers (if present in metadata).
pub fn split_by_chapters<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    // Extract chapters using ffprobe
    let mut cmd = crate::tools::ffprobe::command();
    cmd.arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_chapters")
        .arg(input_path);

    let output = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run ffprobe: {}", e),
        source: None,
    })?;
    crate::tools::ffprobe::ensure_success(&output, "reading audio chapters")?;

    let json_str = String::from_utf8_lossy(&output.stdout);

    // Parse chapters
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| DxError::Config {
            message: format!("Failed to parse chapters: {}", e),
            source: None,
        })?;

    let chapters = parsed
        .get("chapters")
        .and_then(|c| c.as_array())
        .ok_or_else(|| DxError::Config {
            message: "No chapters found in file".to_string(),
            source: None,
        })?;

    if chapters.is_empty() {
        return Err(DxError::Config {
            message: "No chapters found in file".to_string(),
            source: None,
        });
    }

    let extension = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp3");

    let mut segments = Vec::new();

    for (i, chapter) in chapters.iter().enumerate() {
        let start = chapter
            .get("start_time")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let end = chapter
            .get("end_time")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let default_title = format!("chapter_{}", i + 1);
        let title = chapter
            .get("tags")
            .and_then(|t| t.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or(&default_title);

        // Clean title for filename
        let clean_title: String = title
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        let output_path =
            output_dir.join(format!("{:02}_{}.{}", i + 1, clean_title.trim(), extension));

        let mut cmd = crate::tools::ffmpeg::command();
        cmd.arg("-y")
            .arg("-i")
            .arg(input_path)
            .arg("-ss")
            .arg(format!("{:.3}", start))
            .arg("-t")
            .arg(format!("{:.3}", end - start))
            .arg("-c")
            .arg("copy")
            .arg(&output_path);

        if cmd.output().map_or(false, |o| o.status.success()) {
            segments.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Split into {} chapters", segments.len())).with_paths(segments))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_options() {
        let duration = SplitOptions::every_seconds(30.0);
        assert!(matches!(duration.method, SplitMethod::Duration(30.0)));

        let parts = SplitOptions::into_parts(5);
        assert!(matches!(parts.method, SplitMethod::EqualParts(5)));
    }
}
