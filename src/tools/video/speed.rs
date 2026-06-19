//! Video speed adjustment tool.
//!
//! Change video playback speed (speed up or slow down).

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Speed adjustment options.
#[derive(Debug, Clone)]
pub struct SpeedOptions {
    /// Speed factor (0.5 = half speed, 2.0 = double speed).
    pub factor: f32,
    /// Adjust audio pitch to match speed.
    pub maintain_pitch: bool,
    /// Video quality (CRF) for output.
    pub quality: u8,
    /// Interpolate frames for smooth slow motion.
    pub interpolate: bool,
}

impl Default for SpeedOptions {
    fn default() -> Self {
        Self {
            factor: 1.0,
            maintain_pitch: true,
            quality: 18,
            interpolate: false,
        }
    }
}

impl SpeedOptions {
    /// Create for 2x speed.
    pub fn double() -> Self {
        Self {
            factor: 2.0,
            ..Default::default()
        }
    }

    /// Create for half speed (slow motion).
    pub fn half() -> Self {
        Self {
            factor: 0.5,
            ..Default::default()
        }
    }

    /// Create custom speed.
    pub fn with_factor(factor: f32) -> Self {
        Self {
            factor: factor.clamp(0.1, 10.0),
            ..Default::default()
        }
    }

    /// Enable frame interpolation for smooth slow motion.
    pub fn with_interpolation(mut self) -> Self {
        self.interpolate = true;
        self
    }
}

/// Change video playback speed.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for output video
/// * `factor` - Speed factor (0.5 = half, 2.0 = double)
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::change_speed;
///
/// // Double the speed
/// change_speed("video.mp4", "fast.mp4", 2.0).unwrap();
///
/// // Slow motion (half speed)
/// change_speed("video.mp4", "slow.mp4", 0.5).unwrap();
/// ```
pub fn change_speed<P: AsRef<Path>>(input: P, output: P, factor: f32) -> Result<ToolOutput> {
    let options = SpeedOptions::with_factor(factor);
    change_speed_with_options(input, output, options)
}

/// Change speed with full options.
pub fn change_speed_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: SpeedOptions,
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

    let factor = options.factor.clamp(0.1, 10.0);

    // Video: setpts divides by factor (lower factor = slower)
    // For 2x speed, setpts=0.5*PTS
    // For 0.5x speed, setpts=2*PTS
    let pts_factor = 1.0 / factor;

    let mut video_filter = format!("setpts={}*PTS", pts_factor);

    // Add frame interpolation for slow motion
    if options.interpolate && factor < 1.0 {
        video_filter = format!(
            "setpts={}*PTS,minterpolate='mi_mode=mci:mc_mode=aobmc:vsbmc=1:fps=60'",
            pts_factor
        );
    }

    // Audio: atempo has limited range (0.5 to 2.0)
    // Chain multiple atempo filters for extreme speeds
    let audio_filter = build_atempo_filter(factor, options.maintain_pitch);

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(&video_filter);

    if !audio_filter.is_empty() {
        cmd.arg("-af").arg(&audio_filter);
    } else {
        // Remove audio for extreme speeds
        cmd.arg("-an");
    }

    cmd.arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg(options.quality.to_string())
        .arg("-preset")
        .arg("medium")
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Speed change failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    let speed_desc = if factor > 1.0 {
        format!("{}x faster", factor)
    } else if factor < 1.0 {
        format!("{}x slower", 1.0 / factor)
    } else {
        "unchanged".to_string()
    };

    Ok(ToolOutput::success_with_path(
        format!("Changed speed: {}", speed_desc),
        output_path,
    ))
}

/// Build atempo filter chain for audio speed adjustment.
fn build_atempo_filter(factor: f32, maintain_pitch: bool) -> String {
    if !maintain_pitch {
        // Without pitch correction, use asetrate (changes pitch)
        return format!("asetrate=44100*{},aresample=44100", factor);
    }

    // atempo only supports 0.5 to 2.0, chain for more extreme values
    let mut remaining = factor;
    let mut filters = Vec::new();

    if !(0.5..=2.0).contains(&remaining) {
        // For extreme speeds outside atempo range
        while remaining > 2.0 {
            filters.push("atempo=2.0".to_string());
            remaining /= 2.0;
        }
        while remaining < 0.5 {
            filters.push("atempo=0.5".to_string());
            remaining /= 0.5;
        }
    }

    if (remaining - 1.0).abs() > 0.01 {
        filters.push(format!("atempo={:.2}", remaining));
    }

    filters.join(",")
}

/// Create slow motion video.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for slow motion output
/// * `factor` - Slowdown factor (2 = half speed, 4 = quarter speed)
pub fn slow_motion<P: AsRef<Path>>(input: P, output: P, factor: f32) -> Result<ToolOutput> {
    let options = SpeedOptions {
        factor: 1.0 / factor.clamp(1.0, 10.0),
        interpolate: true,
        ..Default::default()
    };
    change_speed_with_options(input, output, options)
}

/// Create timelapse from video.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for timelapse output
/// * `factor` - Speedup factor (10 = 10x faster)
pub fn timelapse<P: AsRef<Path>>(input: P, output: P, factor: f32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let factor = factor.clamp(2.0, 100.0);
    let pts = 1.0 / factor;

    // For timelapse, drop audio and use stream copy where possible
    let filter = format!("setpts={}*PTS", pts);

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(&filter)
        .arg("-an") // No audio for timelapse
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg("18")
        .arg("-preset")
        .arg("fast")
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Timelapse creation failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Created {}x timelapse", factor),
        output_path,
    ))
}

/// Reverse video playback.
pub fn reverse_video<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
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
        .arg("-vf")
        .arg("reverse")
        .arg("-af")
        .arg("areverse")
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg("18")
        .arg("-preset")
        .arg("medium")
        .arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Video reversal failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Reversed video playback",
        output_path,
    ))
}

/// Create boomerang effect (plays forward then backward).
pub fn boomerang<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Complex filter to play forward then reverse
    let filter_complex = "[0:v]split[v1][v2];[v2]reverse[vr];[v1][vr]concat=n=2:v=1:a=0[outv];[0:a]asplit[a1][a2];[a2]areverse[ar];[a1][ar]concat=n=2:v=0:a=1[outa]";

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-filter_complex")
        .arg(filter_complex)
        .arg("-map")
        .arg("[outv]")
        .arg("-map")
        .arg("[outa]")
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
                "Boomerang effect failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Created boomerang effect",
        output_path,
    ))
}

/// Batch speed adjustment.
pub fn batch_speed<P: AsRef<Path>>(inputs: &[P], output_dir: P, factor: f32) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut processed = Vec::new();
    let speed_label = if factor > 1.0 { "fast" } else { "slow" };

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
        let output_path = output_dir.join(format!("{}_{}.{}", file_stem, speed_label, extension));

        if change_speed(input_path, &output_path, factor).is_ok() {
            processed.push(output_path);
        }
    }

    Ok(
        ToolOutput::success(format!("Adjusted speed for {} videos", processed.len()))
            .with_paths(processed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_options() {
        let double = SpeedOptions::double();
        assert_eq!(double.factor, 2.0);

        let half = SpeedOptions::half();
        assert_eq!(half.factor, 0.5);
    }

    #[test]
    fn test_atempo_filter() {
        let filter = build_atempo_filter(1.5, true);
        assert!(filter.contains("atempo"));

        let filter_extreme = build_atempo_filter(4.0, true);
        assert!(filter_extreme.contains("2.0"));
    }
}
