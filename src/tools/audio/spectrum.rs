//! Audio spectrum analyzer.
//!
//! Generate visual representations of audio (waveforms, spectrograms).

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Spectrum visualization type.
#[derive(Debug, Clone, Copy, Default)]
pub enum SpectrumType {
    /// Waveform visualization.
    #[default]
    Waveform,
    /// Spectrogram (frequency over time).
    Spectrogram,
    /// Frequency spectrum bars.
    FrequencyBars,
    /// Volume histogram.
    Histogram,
}

/// Spectrum generation options.
#[derive(Debug, Clone)]
pub struct SpectrumOptions {
    /// Type of visualization.
    pub spectrum_type: SpectrumType,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Background color (hex).
    pub background_color: String,
    /// Foreground color (hex).
    pub foreground_color: String,
    /// Show axis labels.
    pub show_axis: bool,
}

impl Default for SpectrumOptions {
    fn default() -> Self {
        Self {
            spectrum_type: SpectrumType::Waveform,
            width: 1920,
            height: 1080,
            background_color: "000000".to_string(),
            foreground_color: "00ff00".to_string(),
            show_axis: false,
        }
    }
}

impl SpectrumOptions {
    /// Create waveform options.
    pub fn waveform(width: u32, height: u32) -> Self {
        Self {
            spectrum_type: SpectrumType::Waveform,
            width,
            height,
            ..Default::default()
        }
    }

    /// Create spectrogram options.
    pub fn spectrogram(width: u32, height: u32) -> Self {
        Self {
            spectrum_type: SpectrumType::Spectrogram,
            width,
            height,
            ..Default::default()
        }
    }
}

/// Generate spectrum visualization.
///
/// # Arguments
/// * `input` - Path to audio file
/// * `output` - Path for output image/video
/// * `options` - Visualization options
///
/// # Example
/// ```no_run
/// use dx_media::tools::audio::{generate_spectrum, SpectrumOptions};
///
/// // Generate waveform image
/// let options = SpectrumOptions::waveform(1920, 200);
/// generate_spectrum("song.mp3", "waveform.png", options).unwrap();
/// ```
pub fn generate_spectrum<P: AsRef<Path>>(
    input: P,
    output: P,
    options: SpectrumOptions,
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

    match options.spectrum_type {
        SpectrumType::Waveform => generate_waveform(input_path, output_path, &options),
        SpectrumType::Spectrogram => generate_spectrogram(input_path, output_path, &options),
        SpectrumType::FrequencyBars => generate_frequency_bars(input_path, output_path, &options),
        SpectrumType::Histogram => generate_histogram(input_path, output_path, &options),
    }
}

/// Generate waveform visualization.
fn generate_waveform(input: &Path, output: &Path, options: &SpectrumOptions) -> Result<ToolOutput> {
    let filter = format!(
        "showwavespic=s={}x{}:colors=#{}",
        options.width, options.height, options.foreground_color
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-frames:v")
        .arg("1")
        .arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Waveform generation failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Generated {}x{} waveform", options.width, options.height),
        output,
    ))
}

/// Generate spectrogram visualization.
fn generate_spectrogram(
    input: &Path,
    output: &Path,
    options: &SpectrumOptions,
) -> Result<ToolOutput> {
    let filter = format!(
        "showspectrumpic=s={}x{}:color=intensity:scale=cbrt",
        options.width, options.height
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-frames:v")
        .arg("1")
        .arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Spectrogram generation failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Generated {}x{} spectrogram", options.width, options.height),
        output,
    ))
}

/// Generate frequency bars visualization.
fn generate_frequency_bars(
    input: &Path,
    output: &Path,
    options: &SpectrumOptions,
) -> Result<ToolOutput> {
    let filter = format!(
        "showfreqs=s={}x{}:mode=bar:colors=#{}",
        options.width, options.height, options.foreground_color
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-frames:v")
        .arg("1")
        .arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Frequency bars generation failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Generated frequency bars visualization",
        output,
    ))
}

/// Generate audio histogram.
fn generate_histogram(
    input: &Path,
    output: &Path,
    options: &SpectrumOptions,
) -> Result<ToolOutput> {
    let filter = format!(
        "ahistogram=s={}x{}:scale=log",
        options.width, options.height
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-frames:v")
        .arg("1")
        .arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Histogram generation failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Generated audio histogram",
        output,
    ))
}

/// Generate animated waveform video.
pub fn generate_animated_waveform<P: AsRef<Path>>(
    input: P,
    output: P,
    width: u32,
    height: u32,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let filter = format!(
        "[0:a]showwaves=s={}x{}:mode=cline:rate=25:colors=#00ff00[v]",
        width, height
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-map")
        .arg("[v]")
        .arg("-map")
        .arg("0:a")
        .arg("-c:v")
        .arg("libx264")
        .arg("-c:a")
        .arg("copy")
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Animated waveform generation failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Generated animated waveform video",
        output_path,
    ))
}

/// Generate audio visualizer video (for music videos).
pub fn generate_visualizer<P: AsRef<Path>>(
    input: P,
    output: P,
    width: u32,
    height: u32,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Combine multiple visualizations
    let filter = format!(
        "[0:a]showspectrum=s={}x{}:color=intensity:slide=scroll:scale=cbrt[v]",
        width, height
    );

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-map")
        .arg("[v]")
        .arg("-map")
        .arg("0:a")
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("fast")
        .arg("-c:a")
        .arg("aac")
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Visualizer generation failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Generated audio visualizer video",
        output_path,
    ))
}

/// Batch generate waveforms.
pub fn batch_waveform<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    width: u32,
    height: u32,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let options = SpectrumOptions::waveform(width, height);
    let mut generated = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("audio");
        let output_path = output_dir.join(format!("{}_waveform.png", file_stem));

        if generate_spectrum(input_path, &output_path, options.clone()).is_ok() {
            generated.push(output_path);
        }
    }

    Ok(
        ToolOutput::success(format!("Generated {} waveforms", generated.len()))
            .with_paths(generated),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectrum_options() {
        let waveform = SpectrumOptions::waveform(1920, 200);
        assert_eq!(waveform.width, 1920);
        assert!(matches!(waveform.spectrum_type, SpectrumType::Waveform));
    }
}
