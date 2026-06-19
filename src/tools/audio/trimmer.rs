//! Audio trimmer.
//!
//! Cut and extract audio segments.

use crate::deps::check_tool_dependency;
use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use crate::tools::receipts::local_dependency_output;
use crate::types::MediaType;
use std::path::Path;

/// Trim audio to specified time range.
///
/// # Arguments
/// * `input` - Path to input audio file
/// * `output` - Path for trimmed output
/// * `start` - Start time in seconds
/// * `end` - End time in seconds
///
/// # Example
/// ```no_run
/// use dx_media::tools::audio::trim_audio;
///
/// // Extract 30 seconds starting at 1 minute
/// trim_audio("song.mp3", "clip.mp3", 60.0, 90.0).unwrap();
/// ```
pub fn trim_audio<P: AsRef<Path>>(input: P, output: P, start: f64, end: f64) -> Result<ToolOutput> {
    check_tool_dependency("audio::trim")?;

    let input_path = input.as_ref();
    let output_path = output.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    if start >= end {
        return Err(DxError::Config {
            message: "Start time must be less than end time".to_string(),
            source: None,
        });
    }

    let duration = end - start;

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-ss")
        .arg(format!("{:.3}", start))
        .arg("-t")
        .arg(format!("{:.3}", duration))
        .arg("-c")
        .arg("copy") // Stream copy for fast trimming
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Audio trimming failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    local_dependency_output(
        "audio.trim",
        "ffmpeg",
        input_path,
        output_path,
        MediaType::Audio,
        format!(
            "Trimmed {:.1}s segment ({:.1}s to {:.1}s)",
            duration, start, end
        ),
    )
}

/// Trim from start for specified duration.
pub fn trim_duration<P: AsRef<Path>>(
    input: P,
    output: P,
    start: f64,
    duration: f64,
) -> Result<ToolOutput> {
    trim_audio(input, output, start, start + duration)
}

/// Extract first N seconds.
pub fn extract_beginning<P: AsRef<Path>>(input: P, output: P, duration: f64) -> Result<ToolOutput> {
    trim_audio(input, output, 0.0, duration)
}

/// Extract last N seconds.
pub fn extract_ending<P: AsRef<Path>>(input: P, output: P, duration: f64) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Get total duration
    let total_duration = get_audio_duration(input_path)?;
    let start = (total_duration - duration).max(0.0);

    trim_audio(input_path, output_path, start, total_duration)
}

/// Get audio duration in seconds.
pub fn get_audio_duration<P: AsRef<Path>>(input: P) -> Result<f64> {
    check_tool_dependency("audio::trim")?;

    let input_path = input.as_ref();

    let mut cmd = crate::tools::ffprobe::command();
    cmd.arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(input_path);

    let output = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run ffprobe: {}", e),
        source: None,
    })?;
    crate::tools::ffprobe::ensure_success(&output, "reading audio duration")?;

    let duration_str = String::from_utf8_lossy(&output.stdout);
    duration_str.trim().parse().map_err(|_| DxError::Config {
        message: "Failed to parse duration".to_string(),
        source: None,
    })
}

/// Remove a segment from audio (cut out middle portion).
pub fn cut_segment<P: AsRef<Path>>(
    input: P,
    output: P,
    cut_start: f64,
    cut_end: f64,
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

    let _total_duration = get_audio_duration(input_path)?;

    // Use complex filter to concatenate before and after segments
    let filter = format!(
        "[0:a]atrim=end={},asetpts=PTS-STARTPTS[before];[0:a]atrim=start={},asetpts=PTS-STARTPTS[after];[before][after]concat=n=2:v=0:a=1[out]",
        cut_start, cut_end
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-map")
        .arg("[out]")
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Segment removal failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    let cut_duration = cut_end - cut_start;
    local_dependency_output(
        "audio.cut-segment",
        "ffmpeg",
        input_path,
        output_path,
        MediaType::Audio,
        format!(
            "Removed {:.1}s segment ({:.1}s to {:.1}s)",
            cut_duration, cut_start, cut_end
        ),
    )
}

/// Add fade in at start.
pub fn fade_in<P: AsRef<Path>>(input: P, output: P, duration: f64) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let filter = format!("afade=t=in:st=0:d={}", duration);

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-af")
        .arg(&filter)
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Fade in failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    local_dependency_output(
        "audio.fade-in",
        "ffmpeg",
        input_path,
        output_path,
        MediaType::Audio,
        format!("Added {:.1}s fade in", duration),
    )
}

/// Add fade out at end.
pub fn fade_out<P: AsRef<Path>>(input: P, output: P, duration: f64) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let total_duration = get_audio_duration(input_path)?;
    let fade_start = total_duration - duration;

    let filter = format!("afade=t=out:st={}:d={}", fade_start, duration);

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-af")
        .arg(&filter)
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Fade out failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Added {:.1}s fade out", duration),
        output_path,
    ))
}

/// Add both fade in and fade out.
pub fn add_fades<P: AsRef<Path>>(
    input: P,
    output: P,
    fade_in_duration: f64,
    fade_out_duration: f64,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let total_duration = get_audio_duration(input_path)?;
    let fade_out_start = total_duration - fade_out_duration;

    let filter = format!(
        "afade=t=in:st=0:d={},afade=t=out:st={}:d={}",
        fade_in_duration, fade_out_start, fade_out_duration
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-af")
        .arg(&filter)
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Adding fades failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!(
            "Added {:.1}s fade in and {:.1}s fade out",
            fade_in_duration, fade_out_duration
        ),
        output_path,
    ))
}

/// Batch trim multiple files.
pub fn batch_trim<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    start: f64,
    end: f64,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut trimmed = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("audio.mp3");
        let output_path = output_dir.join(format!("trim_{}", file_name));

        if trim_audio(input_path, &output_path, start, end).is_ok() {
            trimmed.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Trimmed {} files", trimmed.len())).with_paths(trimmed))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_time_validation() {
        // Start must be less than end
        assert!(super::trim_audio("nonexistent.mp3", "out.mp3", 10.0, 5.0).is_err());
    }
}
