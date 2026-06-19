//! Audio processing tools.
//!
//! This module provides 10 audio manipulation tools:
//! 1. Format Converter - Convert between audio formats
//! 2. Normalizer - Normalize audio levels
//! 3. Trimmer - Cut audio segments
//! 4. Merger - Combine multiple audio files
//! 5. Spectrum Analyzer - Generate spectrum visualizations
//! 6. Metadata Editor - Read/write audio tags
//! 7. Silence Remover - Remove silent parts
//! 8. Splitter - Split audio by silence/duration
//! 9. Effects Processor - Apply audio effects
//! 10. Speech-to-Text - Transcribe audio
//!
//! ## Native Processing
//!
//! Enable the `audio-core` feature for native Rust audio decoding
//! using symphonia and hound. Enable `audio-tags` for native
//! metadata editing using lofty.

mod converter;
mod effects;
mod merger;
mod metadata;
pub mod native;
mod normalize;
mod silence;
mod spectrum;
mod speech;
mod splitter;
mod trimmer;

pub use converter::*;
pub use effects::*;
pub use merger::*;
pub use metadata::*;
pub use native::*;
pub use normalize::*;
pub use silence::*;
pub use spectrum::*;
pub use speech::*;
pub use splitter::*;
pub use trimmer::*;

use crate::error::Result;
use std::path::Path;

/// Audio tools collection.
pub struct AudioTools;

impl AudioTools {
    /// Create a new AudioTools instance.
    pub fn new() -> Self {
        Self
    }

    /// Convert audio format.
    pub fn convert<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        options: ConvertOptions,
    ) -> Result<super::ToolOutput> {
        converter::convert_audio(input, output, options)
    }

    /// Normalize audio levels.
    pub fn normalize<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        options: NormalizeOptions,
    ) -> Result<super::ToolOutput> {
        normalize::normalize_audio(input, output, options)
    }

    /// Trim audio segment.
    pub fn trim<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        start: f64,
        end: f64,
    ) -> Result<super::ToolOutput> {
        trimmer::trim_audio(input, output, start, end)
    }

    /// Merge multiple audio files.
    pub fn merge<P: AsRef<Path>>(&self, inputs: &[P], output: P) -> Result<super::ToolOutput> {
        merger::merge_audio(inputs, output)
    }

    /// Generate spectrum visualization.
    pub fn spectrum<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        options: SpectrumOptions,
    ) -> Result<super::ToolOutput> {
        spectrum::generate_spectrum(input, output, options)
    }

    /// Read audio metadata.
    pub fn metadata<P: AsRef<Path>>(&self, input: P) -> Result<AudioMetadata> {
        metadata::read_metadata(input)
    }

    /// Remove silence from audio.
    pub fn remove_silence<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        options: SilenceOptions,
    ) -> Result<super::ToolOutput> {
        silence::remove_silence(input, output, options)
    }

    /// Split audio file.
    pub fn split<P: AsRef<Path>>(
        &self,
        input: P,
        output_dir: P,
        options: SplitOptions,
    ) -> Result<super::ToolOutput> {
        splitter::split_audio(input, output_dir, options)
    }

    /// Apply audio effect.
    pub fn apply_effect<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        effect: AudioEffect,
    ) -> Result<super::ToolOutput> {
        effects::apply_effect(input, output, effect)
    }

    /// Transcribe audio to text using a configured external speech provider.
    pub fn transcribe<P: AsRef<Path>>(&self, input: P) -> Result<super::ToolOutput> {
        speech::transcribe(input)
    }
}

impl Default for AudioTools {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if FFmpeg is available for audio processing.
pub fn check_ffmpeg_audio() -> bool {
    crate::tools::ffmpeg::command()
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
