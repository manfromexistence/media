//! Native audio processing using pure Rust crates.
//!
//! This module provides high-performance native Rust audio handling
//! as an alternative to FFmpeg.
//!
//! Enable with the `audio-core` feature flag for decoding.
//! Enable with the `audio-tags` feature flag for metadata editing.

#[cfg(any(feature = "audio-core", feature = "audio-tags"))]
use std::collections::HashMap;
use std::path::Path;

use crate::tools::ToolOutput;
#[cfg(any(feature = "audio-core", feature = "audio-tags"))]
use crate::tools::ToolReceipt;

/// Audio format information.
#[derive(Debug, Clone)]
pub struct AudioInfo {
    /// Duration in seconds.
    pub duration_secs: Option<f64>,
    /// Sample rate in Hz.
    pub sample_rate: Option<u32>,
    /// Number of channels.
    pub channels: Option<u8>,
    /// Bits per sample.
    pub bits_per_sample: Option<u32>,
    /// Codec name.
    pub codec: Option<String>,
    /// Bitrate in kbps.
    pub bitrate_kbps: Option<u32>,
    /// File size in bytes.
    pub file_size: u64,
}

/// Audio metadata/tags for native processing.
#[derive(Debug, Clone, Default)]
pub struct NativeAudioTags {
    /// Track title.
    pub title: Option<String>,
    /// Artist name.
    pub artist: Option<String>,
    /// Album name.
    pub album: Option<String>,
    /// Release year.
    pub year: Option<u32>,
    /// Track number on the album.
    pub track: Option<u32>,
    /// Music genre.
    pub genre: Option<String>,
    /// User comment or notes.
    pub comment: Option<String>,
    /// Album artist (may differ from track artist).
    pub album_artist: Option<String>,
}

/// Get audio file information using symphonia.
#[cfg(feature = "audio-core")]
pub fn audio_info_native(input: impl AsRef<Path>) -> std::io::Result<AudioInfo> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let input = input.as_ref();
    let file = std::fs::File::open(input)?;
    let file_size = file.metadata()?.len();

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = input.extension() {
        hint.with_extension(&ext.to_string_lossy());
    }

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let format = probed.format;

    let mut info = AudioInfo {
        duration_secs: None,
        sample_rate: None,
        channels: None,
        bits_per_sample: None,
        codec: None,
        bitrate_kbps: None,
        file_size,
    };

    // Get track info
    if let Some(track) = format.default_track() {
        let params = &track.codec_params;

        info.sample_rate = params.sample_rate;
        info.channels = params.channels.map(|c| c.count() as u8);
        info.bits_per_sample = params.bits_per_sample;

        // Calculate duration
        if let (Some(time_base), Some(n_frames)) = (params.time_base, params.n_frames) {
            let duration = n_frames as f64 * time_base.numer as f64 / time_base.denom as f64;
            info.duration_secs = Some(duration);
        }
    }

    Ok(info)
}

/// Read audio metadata/tags using lofty.
#[cfg(feature = "audio-tags")]
pub fn read_audio_metadata_native(input: impl AsRef<Path>) -> std::io::Result<NativeAudioTags> {
    use lofty::prelude::*;
    use lofty::probe::Probe;

    let input = input.as_ref();

    let tagged_file = Probe::open(input)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?
        .read()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let mut metadata = NativeAudioTags::default();

    if let Some(tag) = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
    {
        metadata.title = tag.title().map(|s| s.to_string());
        metadata.artist = tag.artist().map(|s| s.to_string());
        metadata.album = tag.album().map(|s| s.to_string());
        metadata.year = tag.year();
        metadata.track = tag.track();
        metadata.genre = tag.genre().map(|s| s.to_string());
        metadata.comment = tag.comment().map(|s| s.to_string());
    }

    Ok(metadata)
}

/// Write audio metadata/tags using lofty.
#[cfg(feature = "audio-tags")]
pub fn write_audio_metadata_native(
    input: impl AsRef<Path>,
    metadata: &NativeAudioTags,
) -> std::io::Result<ToolOutput> {
    use lofty::prelude::*;
    use lofty::probe::Probe;
    use lofty::tag::Tag;

    let input = input.as_ref();

    let mut tagged_file = Probe::open(input)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?
        .read()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    // Get or create primary tag
    let tag = match tagged_file.primary_tag_mut() {
        Some(t) => t,
        None => {
            // Determine appropriate tag type based on file
            let tag_type = tagged_file.primary_tag_type();
            tagged_file.insert_tag(Tag::new(tag_type));
            tagged_file.primary_tag_mut().unwrap()
        }
    };

    // Update fields
    if let Some(ref title) = metadata.title {
        tag.set_title(title.clone());
    }
    if let Some(ref artist) = metadata.artist {
        tag.set_artist(artist.clone());
    }
    if let Some(ref album) = metadata.album {
        tag.set_album(album.clone());
    }
    if let Some(year) = metadata.year {
        tag.set_year(year);
    }
    if let Some(track) = metadata.track {
        tag.set_track(track);
    }
    if let Some(ref genre) = metadata.genre {
        tag.set_genre(genre.clone());
    }
    if let Some(ref comment) = metadata.comment {
        tag.set_comment(comment.clone());
    }

    // Save changes
    tag.save_to_path(input, lofty::config::WriteOptions::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let mut result_metadata = HashMap::new();
    result_metadata.insert("file".to_string(), input.display().to_string());

    Ok(
        ToolOutput::success(format!("Updated metadata for {}", input.display()))
            .with_paths(vec![input.to_path_buf()])
            .with_receipt(
                ToolReceipt::local("audio.metadata.write").with_source(input.display().to_string()),
            )
            .with_output_type_validation(input, crate::types::MediaType::Audio)
            .with_metadata_entries(result_metadata),
    )
}

/// Read WAV file using hound.
#[cfg(feature = "audio-core")]
pub fn read_wav_native(input: impl AsRef<Path>) -> std::io::Result<(Vec<i16>, u32, u16)> {
    use hound::WavReader;

    let input = input.as_ref();
    let reader = WavReader::open(input)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    let channels = spec.channels;

    let samples: Vec<i16> = reader
        .into_samples::<i16>()
        .filter_map(|s| s.ok())
        .collect();

    Ok((samples, sample_rate, channels))
}

/// Write WAV file using hound.
#[cfg(feature = "audio-core")]
pub fn write_wav_native(
    output: impl AsRef<Path>,
    samples: &[i16],
    sample_rate: u32,
    channels: u16,
) -> std::io::Result<ToolOutput> {
    use hound::{WavSpec, WavWriter};

    let output = output.as_ref();

    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(output, spec)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    for sample in samples {
        writer
            .write_sample(*sample)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    }

    writer
        .finalize()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let mut metadata = HashMap::new();
    metadata.insert("sample_rate".to_string(), sample_rate.to_string());
    metadata.insert("channels".to_string(), channels.to_string());
    metadata.insert("samples".to_string(), samples.len().to_string());

    Ok(
        ToolOutput::success_with_path(format!("Created WAV file: {}", output.display()), output)
            .with_receipt(ToolReceipt::local("audio.wav.write"))
            .with_output_type_validation(output, crate::types::MediaType::Audio)
            .with_metadata_entries(metadata),
    )
}

/// Normalize audio (simple peak normalization for WAV).
#[cfg(feature = "audio-core")]
pub fn normalize_wav_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    target_peak: f32,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let (samples, sample_rate, channels) = read_wav_native(input)?;

    // Find current peak
    let current_peak = samples.iter().map(|s| s.abs()).max().unwrap_or(0) as f32 / i16::MAX as f32;

    if current_peak == 0.0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Audio is silent",
        ));
    }

    // Calculate gain
    let gain = target_peak / current_peak;

    // Apply normalization
    let normalized: Vec<i16> = samples
        .iter()
        .map(|s| {
            let normalized = (*s as f32 * gain) as i32;
            normalized.clamp(i16::MIN as i32, i16::MAX as i32) as i16
        })
        .collect();

    write_wav_native(output, &normalized, sample_rate, channels)?;

    let mut metadata = HashMap::new();
    metadata.insert("original_peak".to_string(), format!("{:.3}", current_peak));
    metadata.insert("target_peak".to_string(), format!("{:.3}", target_peak));
    metadata.insert("gain".to_string(), format!("{:.3}", gain));

    Ok(ToolOutput::success_with_path(
        format!(
            "Normalized {} -> {} (gain: {:.2}x)",
            input.display(),
            output.display(),
            gain
        ),
        output,
    )
    .with_receipt(
        ToolReceipt::local("audio.normalize.native").with_source(input.display().to_string()),
    )
    .with_output_type_validation(output, crate::types::MediaType::Audio)
    .with_metadata_entries(metadata))
}

/// Trim WAV audio.
#[cfg(feature = "audio-core")]
pub fn trim_wav_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    start_secs: f64,
    end_secs: Option<f64>,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let (samples, sample_rate, channels) = read_wav_native(input)?;

    let samples_per_frame = channels as usize;
    let start_sample = (start_secs * sample_rate as f64) as usize * samples_per_frame;

    let end_sample = match end_secs {
        Some(end) => (end * sample_rate as f64) as usize * samples_per_frame,
        None => samples.len(),
    };

    if start_sample >= samples.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Start time exceeds audio duration",
        ));
    }

    let trimmed: Vec<i16> = samples[start_sample..end_sample.min(samples.len())].to_vec();

    write_wav_native(output, &trimmed, sample_rate, channels)?;

    let duration = trimmed.len() as f64 / (sample_rate as f64 * channels as f64);

    let mut metadata = HashMap::new();
    metadata.insert("start_secs".to_string(), format!("{:.3}", start_secs));
    metadata.insert(
        "end_secs".to_string(),
        format!("{:.3}", end_secs.unwrap_or(duration)),
    );
    metadata.insert("duration".to_string(), format!("{:.3}", duration));

    Ok(ToolOutput::success_with_path(
        format!(
            "Trimmed {} -> {} ({:.2}s)",
            input.display(),
            output.display(),
            duration
        ),
        output,
    )
    .with_receipt(ToolReceipt::local("audio.trim.native").with_source(input.display().to_string()))
    .with_output_type_validation(output, crate::types::MediaType::Audio)
    .with_metadata_entries(metadata))
}

// Fallback implementations when features are not enabled

/// Gets audio file information using native Rust libraries.
///
/// Returns audio metadata including duration, sample rate, channels, and codec info.
/// Requires the `audio-core` feature to be enabled.
#[cfg(not(feature = "audio-core"))]
pub fn audio_info_native(_input: impl AsRef<Path>) -> std::io::Result<AudioInfo> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native audio processing requires the 'audio-core' feature",
    ))
}

/// Reads audio metadata tags from a file using native Rust libraries.
///
/// Extracts ID3/Vorbis tags including title, artist, album, year, etc.
/// Requires the `audio-tags` feature to be enabled.
#[cfg(not(feature = "audio-tags"))]
pub fn read_audio_metadata_native(_input: impl AsRef<Path>) -> std::io::Result<NativeAudioTags> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Audio metadata requires the 'audio-tags' feature",
    ))
}

/// Writes audio metadata tags to a file using native Rust libraries.
///
/// Updates ID3/Vorbis tags including title, artist, album, year, etc.
/// Requires the `audio-tags` feature to be enabled.
#[cfg(not(feature = "audio-tags"))]
pub fn write_audio_metadata_native(
    _input: impl AsRef<Path>,
    _metadata: &NativeAudioTags,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Audio metadata requires the 'audio-tags' feature",
    ))
}

/// Reads a WAV file and returns raw samples, sample rate, and channel count.
///
/// Requires the `audio-core` feature to be enabled.
#[cfg(not(feature = "audio-core"))]
pub fn read_wav_native(_input: impl AsRef<Path>) -> std::io::Result<(Vec<i16>, u32, u16)> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native audio processing requires the 'audio-core' feature",
    ))
}

/// Writes raw audio samples to a WAV file.
///
/// Requires the `audio-core` feature to be enabled.
#[cfg(not(feature = "audio-core"))]
pub fn write_wav_native(
    _output: impl AsRef<Path>,
    _samples: &[i16],
    _sample_rate: u32,
    _channels: u16,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native audio processing requires the 'audio-core' feature",
    ))
}

/// Normalizes audio to a target peak level.
///
/// Adjusts the volume so the loudest sample reaches the target peak.
/// Requires the `audio-core` feature to be enabled.
#[cfg(not(feature = "audio-core"))]
pub fn normalize_wav_native(
    _input: impl AsRef<Path>,
    _output: impl AsRef<Path>,
    _target_peak: f32,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native audio processing requires the 'audio-core' feature",
    ))
}

/// Trims a WAV file to a specified time range.
///
/// Extracts audio between start and end times (in seconds).
/// Requires the `audio-core` feature to be enabled.
#[cfg(not(feature = "audio-core"))]
pub fn trim_wav_native(
    _input: impl AsRef<Path>,
    _output: impl AsRef<Path>,
    _start_secs: f64,
    _end_secs: Option<f64>,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native audio processing requires the 'audio-core' feature",
    ))
}
