//! Audio normalizer.
//!
//! Normalize audio levels for consistent volume.

use crate::deps::check_tool_dependency;
use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use crate::tools::ToolReceipt;
use crate::tools::receipts::local_dependency_output;
use crate::types::MediaType;
use std::path::Path;

/// Normalization method.
#[derive(Debug, Clone, Copy, Default)]
pub enum NormalizeMethod {
    /// Peak normalization - adjust to target peak level.
    #[default]
    Peak,
    /// RMS normalization - adjust to target RMS level.
    Rms,
    /// EBU R128 loudness normalization (broadcast standard).
    Loudness,
    /// Dynamic range compression.
    DynamicRange,
}

/// Audio normalization options.
#[derive(Debug, Clone)]
pub struct NormalizeOptions {
    /// Normalization method.
    pub method: NormalizeMethod,
    /// Target level in dB (e.g., -1.0 for peak, -23.0 for LUFS).
    pub target_level: f32,
    /// Apply limiter to prevent clipping.
    pub limiter: bool,
    /// True peak limit in dB.
    pub true_peak: f32,
}

impl Default for NormalizeOptions {
    fn default() -> Self {
        Self {
            method: NormalizeMethod::Peak,
            target_level: -1.0,
            limiter: true,
            true_peak: -1.0,
        }
    }
}

impl NormalizeOptions {
    /// Peak normalization to -1dB.
    pub fn peak() -> Self {
        Self {
            method: NormalizeMethod::Peak,
            target_level: -1.0,
            ..Default::default()
        }
    }

    /// Broadcast-standard loudness normalization (-23 LUFS).
    pub fn broadcast() -> Self {
        Self {
            method: NormalizeMethod::Loudness,
            target_level: -23.0,
            limiter: true,
            true_peak: -1.0,
        }
    }

    /// Streaming-optimized loudness (-14 LUFS).
    pub fn streaming() -> Self {
        Self {
            method: NormalizeMethod::Loudness,
            target_level: -14.0,
            limiter: true,
            true_peak: -1.0,
        }
    }
}

/// Normalize audio levels.
///
/// # Arguments
/// * `input` - Path to input audio file
/// * `output` - Path for normalized output
/// * `options` - Normalization options
///
/// # Example
/// ```no_run
/// use dx_media::tools::audio::{normalize_audio, NormalizeOptions};
///
/// // Peak normalize
/// normalize_audio("quiet.mp3", "normalized.mp3", NormalizeOptions::peak()).unwrap();
///
/// // Broadcast loudness standard
/// normalize_audio("audio.wav", "broadcast.wav", NormalizeOptions::broadcast()).unwrap();
/// ```
pub fn normalize_audio<P: AsRef<Path>>(
    input: P,
    output: P,
    options: NormalizeOptions,
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
    check_tool_dependency("audio::normalize")?;

    let filter = match options.method {
        NormalizeMethod::Peak => {
            // Two-pass peak normalization
            let level = 10f32.powf(options.target_level / 20.0);
            format!("volume={}:precision=double", level)
        }
        NormalizeMethod::Rms => {
            format!(
                "loudnorm=I={}:TP={}:LRA=11",
                options.target_level, options.true_peak
            )
        }
        NormalizeMethod::Loudness => {
            // EBU R128 loudness normalization
            format!(
                "loudnorm=I={}:TP={}:LRA=11:measured_I=-23:measured_LRA=7:measured_TP=-2:measured_thresh=-33:offset=0:linear=true:print_format=summary",
                options.target_level, options.true_peak
            )
        }
        NormalizeMethod::DynamicRange => {
            "compand=attacks=0:decays=0.5:points=-80/-80|-45/-45|-27/-25|0/-10:gain=5".to_string()
        }
    };

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-af")
        .arg(&filter);

    if options.limiter && matches!(options.method, NormalizeMethod::Peak) {
        // Add limiter for peak normalization
        cmd.arg("-af").arg(format!(
            "{},alimiter=limit={}",
            filter,
            10f32.powf(options.true_peak / 20.0)
        ));
    }

    cmd.arg(output_path);

    let output_result = cmd
        .output()
        .map_err(|e| crate::tools::ffmpeg::run_failed(e, "normalizing audio"))?;
    crate::tools::ffmpeg::ensure_success(&output_result, "normalizing audio")?;

    let method_name = match options.method {
        NormalizeMethod::Peak => "peak",
        NormalizeMethod::Rms => "RMS",
        NormalizeMethod::Loudness => "loudness (EBU R128)",
        NormalizeMethod::DynamicRange => "dynamic range",
    };

    local_dependency_output(
        "audio.normalize",
        "ffmpeg",
        input_path,
        output_path,
        MediaType::Audio,
        format!("Applied {} normalization", method_name),
    )
}

/// Analyze audio levels (first pass for normalization).
pub fn analyze_levels<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }
    check_tool_dependency("audio::analyze_levels")?;

    // Use volumedetect filter
    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-i")
        .arg(input_path)
        .arg("-af")
        .arg("volumedetect")
        .arg("-f")
        .arg("null")
        .arg("-");

    let output = cmd
        .output()
        .map_err(|e| crate::tools::ffmpeg::run_failed(e, "analyzing audio levels"))?;
    crate::tools::ffmpeg::ensure_success(&output, "analyzing audio levels")?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Parse volume stats from stderr
    let mut result = ToolOutput::success("Audio level analysis complete")
        .with_receipt(
            ToolReceipt::local("audio.analyze-levels")
                .with_dependency("ffmpeg")
                .with_source(input_path.display().to_string()),
        )
        .with_metadata("tool.expected_media_type", "audio")
        .with_metadata("tool.type_validation", "not-applicable")
        .with_metadata("tool.type_validation_reason", "metadata-only-analysis");

    for line in stderr.lines() {
        if line.contains("mean_volume:") || line.contains("max_volume:") {
            if let Some(value) = line.split(':').last() {
                let key = if line.contains("mean") {
                    "mean_volume"
                } else {
                    "max_volume"
                };
                result
                    .metadata
                    .insert(key.to_string(), value.trim().to_string());
            }
        }
    }

    Ok(result)
}

/// Adjust volume by dB amount.
pub fn adjust_volume<P: AsRef<Path>>(input: P, output: P, db: f32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let filter = format!("volume={}dB", db);

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
                "Volume adjustment failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    let direction = if db >= 0.0 { "increased" } else { "decreased" };
    Ok(ToolOutput::success_with_path(
        format!("Volume {} by {}dB", direction, db.abs()),
        output_path,
    ))
}

/// Batch normalize multiple files.
pub fn batch_normalize<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    options: NormalizeOptions,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut normalized = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("audio.mp3");
        let output_path = output_dir.join(format!("norm_{}", file_name));

        if normalize_audio(input_path, &output_path, options.clone()).is_ok() {
            normalized.push(output_path);
        }
    }

    Ok(
        ToolOutput::success(format!("Normalized {} files", normalized.len()))
            .with_paths(normalized),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_options() {
        let peak = NormalizeOptions::peak();
        assert_eq!(peak.target_level, -1.0);

        let broadcast = NormalizeOptions::broadcast();
        assert_eq!(broadcast.target_level, -23.0);
    }
}
