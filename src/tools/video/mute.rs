//! Video audio muting tool.
//!
//! Remove audio track from video files.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Audio removal options.
#[derive(Debug, Clone, Default)]
pub struct MuteOptions {
    /// Keep original video codec (stream copy).
    pub copy_video: bool,
    /// Video quality (CRF) if re-encoding.
    pub quality: u8,
}

impl MuteOptions {
    /// Create options with stream copy (fastest).
    pub fn fast() -> Self {
        Self {
            copy_video: true,
            quality: 18,
        }
    }

    /// Create options with re-encoding.
    pub fn reencode(quality: u8) -> Self {
        Self {
            copy_video: false,
            quality,
        }
    }
}

/// Remove audio from a video file.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for muted output
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::mute_video;
///
/// mute_video("video_with_audio.mp4", "silent.mp4").unwrap();
/// ```
pub fn mute_video<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    mute_video_with_options(input, output, MuteOptions::fast())
}

/// Remove audio with options.
pub fn mute_video_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: MuteOptions,
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
    cmd.arg("-y").arg("-i").arg(input_path).arg("-an"); // No audio

    if options.copy_video {
        cmd.arg("-c:v").arg("copy");
    } else {
        cmd.arg("-c:v")
            .arg("libx264")
            .arg("-crf")
            .arg(options.quality.to_string());
    }

    cmd.arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Muting failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    let output_size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    Ok(ToolOutput::success_with_path(
        format!("Removed audio track ({} bytes)", output_size),
        output_path,
    ))
}

/// Replace audio track with new audio.
///
/// # Arguments
/// * `video` - Path to input video
/// * `audio` - Path to new audio file
/// * `output` - Path for output video
pub fn replace_audio<P: AsRef<Path>>(video: P, audio: P, output: P) -> Result<ToolOutput> {
    let video_path = video.as_ref();
    let audio_path = audio.as_ref();
    let output_path = output.as_ref();

    if !video_path.exists() {
        return Err(DxError::FileIo {
            path: video_path.to_path_buf(),
            message: "Video file not found".to_string(),
            source: None,
        });
    }

    if !audio_path.exists() {
        return Err(DxError::FileIo {
            path: audio_path.to_path_buf(),
            message: "Audio file not found".to_string(),
            source: None,
        });
    }

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(video_path)
        .arg("-i")
        .arg(audio_path)
        .arg("-map")
        .arg("0:v:0") // First video stream from first input
        .arg("-map")
        .arg("1:a:0") // First audio stream from second input
        .arg("-c:v")
        .arg("copy")
        .arg("-c:a")
        .arg("aac")
        .arg("-shortest") // Cut to shortest stream
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Audio replacement failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Replaced audio track",
        output_path,
    ))
}

/// Add audio track to video (mix with existing).
pub fn add_audio<P: AsRef<Path>>(video: P, audio: P, output: P, volume: f32) -> Result<ToolOutput> {
    let video_path = video.as_ref();
    let audio_path = audio.as_ref();
    let output_path = output.as_ref();

    let volume = volume.clamp(0.0, 2.0);

    let filter = format!(
        "[0:a][1:a]amix=inputs=2:duration=first:weights=1 {}[aout]",
        volume
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(video_path)
        .arg("-i")
        .arg(audio_path)
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-map")
        .arg("0:v:0")
        .arg("-map")
        .arg("[aout]")
        .arg("-c:v")
        .arg("copy")
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
                "Audio mixing failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Added audio track",
        output_path,
    ))
}

/// Adjust audio volume in video.
pub fn adjust_volume<P: AsRef<Path>>(input: P, output: P, volume: f32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let volume_filter = format!("volume={}", volume);

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-c:v")
        .arg("copy")
        .arg("-af")
        .arg(&volume_filter)
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
                "Volume adjustment failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Adjusted volume to {}x", volume),
        output_path,
    ))
}

/// Batch mute multiple videos.
pub fn batch_mute<P: AsRef<Path>>(inputs: &[P], output_dir: P) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut muted = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("video");
        let extension = input_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("mp4");
        let output_path = output_dir.join(format!("{}_silent.{}", file_stem, extension));

        if mute_video(input_path, &output_path).is_ok() {
            muted.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Muted {} videos", muted.len())).with_paths(muted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mute_options() {
        let fast = MuteOptions::fast();
        assert!(fast.copy_video);

        let reencode = MuteOptions::reencode(23);
        assert!(!reencode.copy_video);
        assert_eq!(reencode.quality, 23);
    }
}
