//! Tests for audio tools.
//!
//! Tests are organized by tool category. Tests that require external dependencies
//! (FFmpeg) are marked with `#[ignore]` and can be run with:
//! `cargo test -- --ignored`

mod common;

use common::TestFixture;
use dx_media::tools::audio;

// =============================================================================
// 22. converter - Audio format conversion
// =============================================================================

#[test]
fn test_audio_output_format_enum() {
    assert_eq!(format!("{:?}", audio::AudioOutputFormat::Mp3), "Mp3");
    assert_eq!(format!("{:?}", audio::AudioOutputFormat::Wav), "Wav");
    assert_eq!(format!("{:?}", audio::AudioOutputFormat::Flac), "Flac");
    assert_eq!(format!("{:?}", audio::AudioOutputFormat::Ogg), "Ogg");
    assert_eq!(format!("{:?}", audio::AudioOutputFormat::Aac), "Aac");
}

#[test]
fn test_convert_options() {
    let options = audio::ConvertOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_convert_audio() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_audio("test.mp3");
    let output = fixture.path("output.wav");

    let result = audio::convert_audio(&input, &output, audio::ConvertOptions::default());
    assert!(
        result.is_ok(),
        "Convert audio should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 23. normalize - Audio normalization
// =============================================================================

#[test]
fn test_normalize_method_enum() {
    assert_eq!(format!("{:?}", audio::NormalizeMethod::Peak), "Peak");
    assert_eq!(format!("{:?}", audio::NormalizeMethod::Rms), "Rms");
    assert_eq!(
        format!("{:?}", audio::NormalizeMethod::Loudness),
        "Loudness"
    );
}

#[test]
fn test_normalize_options() {
    let options = audio::NormalizeOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

// =============================================================================
// 25. merger - Audio merging
// =============================================================================

#[test]
fn test_merge_method_enum() {
    assert_eq!(
        format!("{:?}", audio::MergeMethod::Concatenate),
        "Concatenate"
    );
    assert_eq!(format!("{:?}", audio::MergeMethod::Mix), "Mix");
}

#[test]
fn test_merge_options() {
    let options = audio::MergeOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

// =============================================================================
// 26. spectrum - Audio visualization
// =============================================================================

#[test]
fn test_spectrum_type_enum() {
    assert_eq!(format!("{:?}", audio::SpectrumType::Waveform), "Waveform");
    assert_eq!(
        format!("{:?}", audio::SpectrumType::Spectrogram),
        "Spectrogram"
    );
    assert_eq!(
        format!("{:?}", audio::SpectrumType::FrequencyBars),
        "FrequencyBars"
    );
}

#[test]
fn test_spectrum_options() {
    let options = audio::SpectrumOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

// =============================================================================
// 27. metadata - Audio metadata
// =============================================================================

#[test]
fn test_audio_metadata_struct() {
    let metadata = audio::AudioMetadata::default();
    assert!(metadata.title.is_none(), "Default title should be None");
    assert!(metadata.artist.is_none(), "Default artist should be None");
}

// =============================================================================
// 28. silence - Silence detection/removal
// =============================================================================

#[test]
fn test_silence_options() {
    let options = audio::SilenceOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

// =============================================================================
// 29. splitter - Audio splitting
// =============================================================================

#[test]
fn test_split_method_enum() {
    let methods = [
        audio::SplitMethod::Duration(60.0),
        audio::SplitMethod::Silence {
            threshold_db: -40.0,
            min_duration: 0.5,
        },
        audio::SplitMethod::Timestamps(vec![10.0, 30.0, 60.0]),
        audio::SplitMethod::EqualParts(5),
    ];
    for method in methods {
        assert!(!format!("{:?}", method).is_empty());
    }
}

#[test]
fn test_split_options() {
    let options = audio::SplitOptions {
        method: audio::SplitMethod::Duration(30.0),
        pattern: "part_{n}".to_string(),
        zero_pad: 3,
    };
    assert_eq!(options.zero_pad, 3);
    assert_eq!(options.pattern, "part_{n}");
}

// =============================================================================
// 30. effects - Audio effects
// =============================================================================

#[test]
fn test_audio_effect_enum() {
    let effects = [
        audio::AudioEffect::Speed(1.5),
        audio::AudioEffect::Pitch(1.2),
        audio::AudioEffect::Echo {
            delay: 0.5,
            decay: 0.5,
        },
        audio::AudioEffect::Reverb {
            room_size: 0.8,
            damping: 0.5,
        },
        audio::AudioEffect::LowPass(3000),
        audio::AudioEffect::HighPass(100),
        audio::AudioEffect::BandPass {
            low: 100,
            high: 3000,
        },
        audio::AudioEffect::BassBoost(5.0),
        audio::AudioEffect::TrebleBoost(3.0),
        audio::AudioEffect::Compressor {
            threshold: -20.0,
            ratio: 4.0,
        },
        audio::AudioEffect::Distortion(0.5),
        audio::AudioEffect::Flanger,
        audio::AudioEffect::Phaser,
    ];
    for effect in effects {
        assert!(!format!("{:?}", effect).is_empty());
    }
}

// =============================================================================
// 31. speech - Speech recognition
// =============================================================================

#[test]
fn test_transcribe_options() {
    let options = audio::TranscribeOptions::default();
    assert!(
        !options.language.is_empty(),
        "Default language should be set"
    );
}

#[test]
fn test_transcription_result() {
    let result = audio::TranscriptionResult {
        text: "Hello World".to_string(),
        segments: vec![],
        detected_language: Some("en".to_string()),
        confidence: 0.95,
    };
    assert_eq!(result.text, "Hello World");
    assert_eq!(result.confidence, 0.95);
}

#[test]
fn test_transcription_segment() {
    let segment = audio::TranscriptionSegment {
        start: 0.0,
        end: 5.0,
        text: "Hello".to_string(),
        speaker: Some("Speaker 1".to_string()),
    };
    assert_eq!(segment.text, "Hello");
    assert_eq!(segment.start, 0.0);
    assert_eq!(segment.end, 5.0);
}

#[test]
fn test_check_ffmpeg_audio() {
    let result = std::panic::catch_unwind(audio::check_ffmpeg_audio);
    assert!(result.is_ok());
}
