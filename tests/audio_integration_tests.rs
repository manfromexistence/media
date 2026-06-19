//! Integration tests for audio processing tools.
//!
//! These tests require FFmpeg to be installed.

mod common;

use common::TestFixture;

fn assert_ffmpeg_available() {
    assert!(
        TestFixture::has_ffmpeg(),
        "FFmpeg is required for this integration test; set DX_MEDIA_FFMPEG_BIN or run fixture-backed tests instead"
    );
}

#[test]
fn test_audio_trim() {
    assert_ffmpeg_available();

    use dx_media::tools::audio::trim_audio;

    let fixture = TestFixture::new();
    let input = fixture.create_test_audio("test.wav");

    let output = fixture.path("trimmed.wav");

    let result = trim_audio(&input, &output, 0.0, 1.0);
    assert!(
        result.is_ok(),
        "Audio trim should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Trimmed audio should exist");
}

#[test]
fn test_audio_merge() {
    assert_ffmpeg_available();

    use dx_media::tools::audio::merge_audio;

    let fixture = TestFixture::new();
    let audio1 = fixture.create_test_audio("test1.wav");
    let audio2 = fixture.create_test_audio("test2.wav");

    let output = fixture.path("merged.wav");

    let result = merge_audio(&[&audio1, &audio2], &output);
    assert!(
        result.is_ok(),
        "Audio merge should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Merged audio should exist");
}

#[test]
fn test_audio_normalize() {
    assert_ffmpeg_available();

    use dx_media::tools::audio::{NormalizeOptions, normalize_audio};

    let fixture = TestFixture::new();
    let input = fixture.create_test_audio("test.wav");

    let output = fixture.path("normalized.wav");
    let options = NormalizeOptions::default();

    let result = normalize_audio(&input, &output, options);
    assert!(
        result.is_ok(),
        "Audio normalization should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Normalized audio should exist");
}

#[test]
fn test_audio_remove_silence() {
    assert_ffmpeg_available();

    use dx_media::tools::audio::{SilenceOptions, remove_silence};

    let fixture = TestFixture::new();
    let input = fixture.create_test_audio("test.wav");

    let output = fixture.path("no_silence.wav");
    let options = SilenceOptions::default();

    let result = remove_silence(&input, &output, options);
    assert!(
        result.is_ok(),
        "Silence removal should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Audio without silence should exist");
}

#[test]
fn test_audio_split() {
    assert_ffmpeg_available();

    use dx_media::tools::audio::{SplitOptions, split_audio};

    let fixture = TestFixture::new();
    let input = fixture.create_test_audio("test.wav");

    let output_dir = fixture.path("split");
    let options = SplitOptions::every_seconds(1.0);

    let result = split_audio(&input, &output_dir, options);
    assert!(
        result.is_ok(),
        "Audio split should succeed: {:?}",
        result.err()
    );
    assert!(output_dir.exists(), "Split output directory should exist");
}

#[test]
fn test_audio_metadata() {
    assert_ffmpeg_available();

    use dx_media::tools::audio::read_metadata;

    let fixture = TestFixture::new();
    let input = fixture.create_test_audio("test.wav");

    let result = read_metadata(&input);
    assert!(
        result.is_ok(),
        "Metadata reading should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_audio_apply_effect() {
    assert_ffmpeg_available();

    use dx_media::tools::audio::{AudioEffect, apply_effect};

    let fixture = TestFixture::new();
    let input = fixture.create_test_audio("test.wav");

    let output = fixture.path("effect.wav");
    let effect = AudioEffect::Echo {
        delay: 0.5,
        decay: 0.5,
    };

    let result = apply_effect(&input, &output, effect);
    assert!(
        result.is_ok(),
        "Audio effect should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Audio with effect should exist");
}

#[test]
fn test_audio_spectrum() {
    assert_ffmpeg_available();

    use dx_media::tools::audio::{SpectrumOptions, generate_spectrum};

    let fixture = TestFixture::new();
    let input = fixture.create_test_audio("test.wav");

    let output = fixture.path("spectrum.png");
    let options = SpectrumOptions::default();

    let result = generate_spectrum(&input, &output, options);
    assert!(
        result.is_ok(),
        "Spectrum generation should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Spectrum image should exist");
}
