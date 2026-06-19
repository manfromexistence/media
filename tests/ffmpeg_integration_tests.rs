//! Integration tests for FFmpeg-based tools.
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
fn test_video_transcode_mp4() {
    assert_ffmpeg_available();

    use dx_media::tools::video::{TranscodeOptions, VideoFormat, transcode_video};

    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");

    let output = fixture.path("output.mp4");
    let options = TranscodeOptions::new(VideoFormat::Mp4);

    let result = transcode_video(&input, &output, options);
    assert!(
        result.is_ok(),
        "Transcode should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Output video should exist");
}

#[test]
fn test_audio_convert_mp3() {
    assert_ffmpeg_available();

    use dx_media::tools::audio::{ConvertOptions, convert_audio};

    let fixture = TestFixture::new();
    let input = fixture.create_test_audio("test.wav");

    let output = fixture.path("output.mp3");
    let options = ConvertOptions::mp3(192);

    let result = convert_audio(&input, &output, options);
    assert!(
        result.is_ok(),
        "Audio conversion should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Output audio should exist");
}

#[test]
fn test_video_extract_audio() {
    assert_ffmpeg_available();

    use dx_media::tools::video::{AudioFormat, extract_audio};

    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");

    let output = fixture.path("audio.mp3");

    let result = extract_audio(&input, &output, AudioFormat::Mp3);
    assert!(
        result.is_ok(),
        "Audio extraction should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Output audio should exist");
}

#[test]
fn test_video_trim() {
    assert_ffmpeg_available();

    use dx_media::tools::video::trim_video;

    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");

    let output = fixture.path("trimmed.mp4");

    let result = trim_video(&input, &output, 0.0, 1.0);
    assert!(
        result.is_ok(),
        "Video trim should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Trimmed video should exist");
}

#[test]
fn test_video_to_gif() {
    assert_ffmpeg_available();

    use dx_media::tools::video::{GifOptions, video_to_gif};

    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");

    let output = fixture.path("output.gif");
    let options = GifOptions::with_width(320);

    let result = video_to_gif(&input, &output, options);
    assert!(
        result.is_ok(),
        "GIF creation should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "GIF should exist");
}

#[test]
fn test_video_extract_thumbnail() {
    assert_ffmpeg_available();

    use dx_media::tools::video::extract_thumbnail;

    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");

    let output = fixture.path("thumb.jpg");

    let result = extract_thumbnail(&input, &output, 0.5);
    assert!(
        result.is_ok(),
        "Thumbnail extraction should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Thumbnail should exist");
}

#[test]
fn test_video_scale() {
    assert_ffmpeg_available();

    use dx_media::tools::video::scale_video;

    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");

    let output = fixture.path("scaled.mp4");

    let result = scale_video(&input, &output, 160, 120);
    assert!(
        result.is_ok(),
        "Video scaling should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Scaled video should exist");
}

#[test]
fn test_video_concatenate() {
    assert_ffmpeg_available();

    use dx_media::tools::video::concatenate_videos;

    let fixture = TestFixture::new();
    let video1 = fixture.create_test_video("test1.mp4");
    let video2 = fixture.create_test_video("test2.mp4");

    let output = fixture.path("concatenated.mp4");

    let result = concatenate_videos(&[&video1, &video2], &output);
    assert!(
        result.is_ok(),
        "Video concatenation should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Concatenated video should exist");
}

#[test]
fn test_video_mute() {
    assert_ffmpeg_available();

    use dx_media::tools::video::mute_video;

    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");

    let output = fixture.path("muted.mp4");

    let result = mute_video(&input, &output);
    assert!(
        result.is_ok(),
        "Video muting should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Muted video should exist");
}

#[test]
fn test_video_watermark() {
    assert_ffmpeg_available();

    use dx_media::tools::video::add_text_watermark;

    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");

    let output = fixture.path("watermarked.mp4");

    let result = add_text_watermark(&input, &output, "TEST");
    assert!(
        result.is_ok(),
        "Video watermarking should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Watermarked video should exist");
}

#[test]
fn test_video_speed_change() {
    assert_ffmpeg_available();

    use dx_media::tools::video::change_speed;

    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");

    let output = fixture.path("fast.mp4");

    let result = change_speed(&input, &output, 2.0);
    assert!(
        result.is_ok(),
        "Video speed change should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Speed-changed video should exist");
}
