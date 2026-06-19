//! Audio effects processor.
//!
//! Apply various audio effects (EQ, reverb, echo, pitch, etc.)

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Available audio effects.
#[derive(Debug, Clone)]
pub enum AudioEffect {
    /// Change playback speed without affecting pitch.
    Speed(f32),
    /// Change pitch without affecting speed.
    Pitch(f32),
    /// Add echo effect.
    Echo {
        /// Delay time in seconds before echo starts.
        delay: f64,
        /// Decay factor controlling echo fade (0.0-1.0).
        decay: f32,
    },
    /// Add reverb effect.
    Reverb {
        /// Size of the simulated room (0.0-1.0).
        room_size: f32,
        /// High frequency damping factor (0.0-1.0).
        damping: f32,
    },
    /// Low-pass filter.
    LowPass(u32),
    /// High-pass filter.
    HighPass(u32),
    /// Band-pass filter.
    BandPass {
        /// Low frequency cutoff in Hz.
        low: u32,
        /// High frequency cutoff in Hz.
        high: u32,
    },
    /// Bass boost.
    BassBoost(f32),
    /// Treble boost.
    TrebleBoost(f32),
    /// Equalizer (10-band).
    Equalizer(Vec<f32>),
    /// Compression/limiting.
    Compressor {
        /// Threshold level in dB where compression starts.
        threshold: f32,
        /// Compression ratio (e.g., 4.0 means 4:1 compression).
        ratio: f32,
    },
    /// Distortion.
    Distortion(f32),
    /// Flanger effect.
    Flanger,
    /// Phaser effect.
    Phaser,
    /// Chorus effect.
    Chorus,
    /// Noise reduction.
    DeNoise(f32),
    /// Stereo widening.
    StereoWiden(f32),
    /// Reverse audio.
    Reverse,
    /// Custom FFmpeg filter.
    Custom(String),
}

impl AudioEffect {
    /// Get FFmpeg filter string.
    fn to_filter(&self) -> String {
        match self {
            AudioEffect::Speed(factor) => format!("atempo={}", factor.clamp(0.5, 2.0)),
            AudioEffect::Pitch(semitones) => {
                let ratio = 2f32.powf(*semitones / 12.0);
                format!("asetrate=44100*{},aresample=44100", ratio)
            }
            AudioEffect::Echo { delay, decay } => {
                format!("aecho=0.8:0.88:{}:{}", (delay * 1000.0) as u32, decay)
            }
            AudioEffect::Reverb { room_size: _, damping: _ } => {
                "aecho=0.8:0.9:1000|1800:0.3|0.25,highpass=f=200".to_string()
            }
            AudioEffect::LowPass(freq) => format!("lowpass=f={}", freq),
            AudioEffect::HighPass(freq) => format!("highpass=f={}", freq),
            AudioEffect::BandPass { low, high } => {
                format!("highpass=f={},lowpass=f={}", low, high)
            }
            AudioEffect::BassBoost(gain) => format!("bass=g={}", gain),
            AudioEffect::TrebleBoost(gain) => format!("treble=g={}", gain),
            AudioEffect::Equalizer(bands) => {
                // 10-band EQ at standard frequencies
                let freqs = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
                let filters: Vec<String> = bands.iter()
                    .take(10)
                    .enumerate()
                    .filter(|&(_, gain)| gain.abs() > 0.1)
                    .map(|(i, gain)| format!("equalizer=f={}:width_type=o:width=2:g={}", freqs[i], gain))
                    .collect();
                filters.join(",")
            }
            AudioEffect::Compressor { threshold, ratio } => {
                format!("compand=attacks=0.1:decays=0.3:points=-80/-80|{}/-{}|0/-{}", 
                    threshold, threshold / ratio, threshold / ratio)
            }
            AudioEffect::Distortion(amount) => {
                format!("overdrive=gain={}:colour=50", amount * 10.0)
            }
            AudioEffect::Flanger => {
                "flanger=delay=0:depth=2:regen=0:width=71:speed=0.5:shape=triangular:phase=25:interp=linear".to_string()
            }
            AudioEffect::Phaser => {
                "aphaser=in_gain=0.4:out_gain=0.74:delay=3.0:decay=0.4:speed=0.5:type=triangular".to_string()
            }
            AudioEffect::Chorus => {
                "chorus=0.5:0.9:50|60|40:0.4|0.32|0.3:0.25|0.4|0.3:2|2.3|1.3".to_string()
            }
            AudioEffect::DeNoise(strength) => {
                format!("afftdn=nf=-{}:nt=w", (strength * 25.0) as i32)
            }
            AudioEffect::StereoWiden(amount) => {
                format!("stereotools=mlev={}:slev={}", 1.0 - amount, 1.0 + amount)
            }
            AudioEffect::Reverse => "areverse".to_string(),
            AudioEffect::Custom(filter) => filter.clone(),
        }
    }

    /// Get human-readable name.
    pub fn name(&self) -> &str {
        match self {
            AudioEffect::Speed(_) => "Speed Change",
            AudioEffect::Pitch(_) => "Pitch Shift",
            AudioEffect::Echo { .. } => "Echo",
            AudioEffect::Reverb { .. } => "Reverb",
            AudioEffect::LowPass(_) => "Low Pass Filter",
            AudioEffect::HighPass(_) => "High Pass Filter",
            AudioEffect::BandPass { .. } => "Band Pass Filter",
            AudioEffect::BassBoost(_) => "Bass Boost",
            AudioEffect::TrebleBoost(_) => "Treble Boost",
            AudioEffect::Equalizer(_) => "Equalizer",
            AudioEffect::Compressor { .. } => "Compressor",
            AudioEffect::Distortion(_) => "Distortion",
            AudioEffect::Flanger => "Flanger",
            AudioEffect::Phaser => "Phaser",
            AudioEffect::Chorus => "Chorus",
            AudioEffect::DeNoise(_) => "Noise Reduction",
            AudioEffect::StereoWiden(_) => "Stereo Widening",
            AudioEffect::Reverse => "Reverse",
            AudioEffect::Custom(_) => "Custom Effect",
        }
    }
}

/// Apply audio effect.
///
/// # Arguments
/// * `input` - Path to input audio file
/// * `output` - Path for processed output
/// * `effect` - Effect to apply
///
/// # Example
/// ```no_run
/// use dx_media::tools::audio::{apply_effect, AudioEffect};
///
/// // Add echo effect
/// apply_effect("voice.mp3", "echo.mp3", AudioEffect::Echo { delay: 0.5, decay: 0.3 }).unwrap();
///
/// // Bass boost
/// apply_effect("music.mp3", "bass.mp3", AudioEffect::BassBoost(6.0)).unwrap();
/// ```
pub fn apply_effect<P: AsRef<Path>>(
    input: P,
    output: P,
    effect: AudioEffect,
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

    let filter = effect.to_filter();

    if filter.is_empty() {
        return Err(DxError::Config {
            message: "Invalid effect configuration".to_string(),
            source: None,
        });
    }

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
                "Effect application failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Applied {} effect", effect.name()),
        output_path,
    ))
}

/// Apply multiple effects in sequence.
pub fn apply_effects<P: AsRef<Path>>(
    input: P,
    output: P,
    effects: &[AudioEffect],
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

    if effects.is_empty() {
        return Err(DxError::Config {
            message: "No effects provided".to_string(),
            source: None,
        });
    }

    let filters: Vec<String> = effects
        .iter()
        .map(|e| e.to_filter())
        .filter(|f| !f.is_empty())
        .collect();

    let filter_chain = filters.join(",");

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-af")
        .arg(&filter_chain)
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Effects chain failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    let names: Vec<&str> = effects.iter().map(|e| e.name()).collect();
    Ok(ToolOutput::success_with_path(
        format!("Applied {} effects: {}", effects.len(), names.join(", ")),
        output_path,
    ))
}

/// Create telephone/radio voice effect.
pub fn telephone_effect<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    apply_effect(
        input,
        output,
        AudioEffect::BandPass {
            low: 300,
            high: 3400,
        },
    )
}

/// Create underwater effect.
pub fn underwater_effect<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    apply_effects(
        input,
        output,
        &[
            AudioEffect::LowPass(500),
            AudioEffect::Reverb {
                room_size: 0.8,
                damping: 0.5,
            },
        ],
    )
}

/// Create chipmunk (high pitch) effect.
pub fn chipmunk_effect<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    apply_effect(input, output, AudioEffect::Pitch(6.0))
}

/// Create deep voice effect.
pub fn deep_voice_effect<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    apply_effect(input, output, AudioEffect::Pitch(-6.0))
}

/// Create robot voice effect.
pub fn robot_effect<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    apply_effect(
        input,
        output,
        AudioEffect::Custom("afftfilt=real='hypot(re,im)*cos(0)':imag='hypot(re,im)*sin(0)':win_size=512:overlap=0.75".to_string()),
    )
}

/// Batch apply effect to multiple files.
pub fn batch_effect<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    effect: AudioEffect,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut processed = Vec::new();
    let effect_name = effect.name();

    for input in inputs {
        let input_path = input.as_ref();
        let file_name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("audio.mp3");
        let output_path = output_dir.join(format!("fx_{}", file_name));

        if apply_effect(input_path, &output_path, effect.clone()).is_ok() {
            processed.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!(
        "Applied {} to {} files",
        effect_name,
        processed.len()
    ))
    .with_paths(processed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_filters() {
        let speed = AudioEffect::Speed(1.5);
        assert!(speed.to_filter().contains("atempo"));

        let bass = AudioEffect::BassBoost(6.0);
        assert!(bass.to_filter().contains("bass"));
    }

    #[test]
    fn test_effect_names() {
        assert_eq!(AudioEffect::Reverse.name(), "Reverse");
        assert_eq!(AudioEffect::Chorus.name(), "Chorus");
    }
}
