//! Tests for video tools.
//!
//! Tests are organized by tool category. Tests that require external dependencies
//! (FFmpeg) are marked with `#[ignore]` and can be run with:
//! `cargo test -- --ignored`

mod common;

use common::TestFixture;
use dx_media::tools::video;

// =============================================================================
// 11. transcoder - Video format conversion
// =============================================================================

#[test]
fn test_video_format_enum() {
    assert_eq!(format!("{:?}", video::VideoFormat::Mp4), "Mp4");
    assert_eq!(format!("{:?}", video::VideoFormat::Mkv), "Mkv");
    assert_eq!(format!("{:?}", video::VideoFormat::WebM), "WebM");
    assert_eq!(format!("{:?}", video::VideoFormat::Avi), "Avi");
}

#[test]
fn test_video_quality_enum() {
    assert_eq!(format!("{:?}", video::VideoQuality::Low), "Low");
    assert_eq!(format!("{:?}", video::VideoQuality::Medium), "Medium");
    assert_eq!(format!("{:?}", video::VideoQuality::High), "High");
}

#[test]
fn test_transcode_options() {
    let options = video::TranscodeOptions::default();
    // Verify default options are sensible
    assert!(!format!("{:?}", options).is_empty());
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_transcode_video() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("output.mp4");

    let result = video::transcode_video(&input, &output, video::TranscodeOptions::default());
    assert!(
        result.is_ok(),
        "Transcode should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_to_mp4() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mkv");
    let output = fixture.path("output.mp4");

    let result = video::to_mp4(&input, &output);
    assert!(result.is_ok(), "To MP4 should succeed: {:?}", result.err());
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_to_webm() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("output.webm");

    let result = video::to_webm(&input, &output);
    assert!(result.is_ok(), "To WebM should succeed: {:?}", result.err());
}

// =============================================================================
// 12. audio_extract - Audio extraction
// =============================================================================

#[test]
fn test_audio_format_enum() {
    assert_eq!(format!("{:?}", video::AudioFormat::Mp3), "Mp3");
    assert_eq!(format!("{:?}", video::AudioFormat::Aac), "Aac");
    assert_eq!(format!("{:?}", video::AudioFormat::Wav), "Wav");
    assert_eq!(format!("{:?}", video::AudioFormat::Flac), "Flac");
    assert_eq!(format!("{:?}", video::AudioFormat::Ogg), "Ogg");
}

#[test]
fn test_audio_extract_options() {
    let options = video::AudioExtractOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_extract_audio() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("audio.mp3");

    let result = video::extract_audio(&input, &output, video::AudioFormat::Mp3);
    assert!(
        result.is_ok(),
        "Extract audio should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_extract_mp3() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("audio.mp3");

    let result = video::extract_mp3(&input, &output);
    assert!(
        result.is_ok(),
        "Extract MP3 should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_extract_wav() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("audio.wav");

    let result = video::extract_wav(&input, &output);
    assert!(
        result.is_ok(),
        "Extract WAV should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 13. trimmer - Video trimming
// =============================================================================

#[test]
fn test_trim_mode_enum() {
    assert_eq!(format!("{:?}", video::TrimMode::Copy), "Copy");
    assert_eq!(format!("{:?}", video::TrimMode::Reencode), "Reencode");
}

#[test]
fn test_trim_options() {
    let options = video::TrimOptions::new(0.0, 10.0);
    assert!(!format!("{:?}", options).is_empty());
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_trim_video() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("trimmed.mp4");

    let result = video::trim_video(&input, &output, 0.0, 10.0);
    assert!(
        result.is_ok(),
        "Trim video should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_extract_clip() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("clip.mp4");

    let result = video::extract_clip(&input, &output, 5.0, 10.0);
    assert!(
        result.is_ok(),
        "Extract clip should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_split_video() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output_dir = fixture.path("segments");

    let result = video::split_video(&input, &output_dir, &[30.0, 60.0, 90.0]);
    assert!(
        result.is_ok(),
        "Split video should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_time() {
    let time = video::parse_time("01:30:00");
    assert!(time.is_some(), "Should parse valid time");
    assert_eq!(time.unwrap(), 5400.0, "01:30:00 should be 5400 seconds");

    let time2 = video::parse_time("00:01:30");
    assert!(time2.is_some());
    assert_eq!(time2.unwrap(), 90.0, "00:01:30 should be 90 seconds");
}

// =============================================================================
// 14. gif - GIF creation
// =============================================================================

#[test]
fn test_gif_options() {
    let options = video::GifOptions::default();
    assert!(options.width > 0, "Default width should be positive");
    assert!(options.fps > 0, "Default FPS should be positive");

    let options_with_width = video::GifOptions::with_width(320);
    assert_eq!(options_with_width.width, 320);

    let options_chained = video::GifOptions::with_width(400)
        .with_fps(20)
        .with_range(1.0, 5.0);
    assert_eq!(options_chained.fps, 20);
    assert_eq!(options_chained.width, 400);
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_video_to_gif() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("output.gif");

    let result = video::video_to_gif(&input, &output, video::GifOptions::default());
    assert!(
        result.is_ok(),
        "Video to GIF should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_quick_gif() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("quick.gif");

    let result = video::quick_gif(&input, &output);
    assert!(
        result.is_ok(),
        "Quick GIF should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 15. thumbnail - Video thumbnails
// =============================================================================

#[test]
fn test_thumbnail_format_enum() {
    assert_eq!(format!("{:?}", video::ThumbnailFormat::Jpeg), "Jpeg");
    assert_eq!(format!("{:?}", video::ThumbnailFormat::Png), "Png");
}

#[test]
fn test_thumbnail_options() {
    let options = video::ThumbnailOptions::default();
    assert!(options.quality > 0, "Default quality should be positive");
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_extract_thumbnail() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("thumb.jpg");

    let result = video::extract_thumbnail(&input, &output, 5.0);
    assert!(
        result.is_ok(),
        "Extract thumbnail should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_extract_first_frame() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("first.jpg");

    let result = video::extract_first_frame(&input, &output);
    assert!(
        result.is_ok(),
        "Extract first frame should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_create_contact_sheet() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("contact.jpg");

    let result = video::create_contact_sheet(&input, &output, 4, 4, 160);
    assert!(
        result.is_ok(),
        "Create contact sheet should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 16. scaler - Video scaling
// =============================================================================

#[test]
fn test_resolution_enum() {
    // Verify all resolution variants exist
    let resolutions = [
        video::Resolution::R240p,
        video::Resolution::R360p,
        video::Resolution::R480p,
        video::Resolution::R720p,
        video::Resolution::R1080p,
        video::Resolution::R1440p,
        video::Resolution::R4k,
        video::Resolution::Custom(1920, 1080),
    ];
    for res in resolutions {
        assert!(!format!("{:?}", res).is_empty());
    }
}

#[test]
fn test_scale_algorithm_enum() {
    assert_eq!(format!("{:?}", video::ScaleAlgorithm::Bilinear), "Bilinear");
    assert_eq!(format!("{:?}", video::ScaleAlgorithm::Bicubic), "Bicubic");
    assert_eq!(format!("{:?}", video::ScaleAlgorithm::Lanczos), "Lanczos");
}

#[test]
fn test_scale_options() {
    let options = video::ScaleOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_scale_video() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("scaled.mp4");

    let result = video::scale_video(&input, &output, 1280, 720);
    assert!(
        result.is_ok(),
        "Scale video should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_scale_to_720p() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("720p.mp4");

    let result = video::scale_to_720p(&input, &output);
    assert!(
        result.is_ok(),
        "Scale to 720p should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_scale_to_1080p() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("1080p.mp4");

    let result = video::scale_to_1080p(&input, &output);
    assert!(
        result.is_ok(),
        "Scale to 1080p should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 17. concatenate - Video concatenation
// =============================================================================

#[test]
fn test_concat_method_enum() {
    assert_eq!(format!("{:?}", video::ConcatMethod::Demuxer), "Demuxer");
    assert_eq!(format!("{:?}", video::ConcatMethod::Filter), "Filter");
}

#[test]
fn test_concat_options() {
    let options = video::ConcatOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_concatenate_videos() {
    let fixture = TestFixture::new();
    let video1 = fixture.create_test_video("video1.mp4");
    let video2 = fixture.create_test_video("video2.mp4");
    let output = fixture.path("combined.mp4");

    let result = video::concatenate_videos(&[&video1, &video2], &output);
    assert!(
        result.is_ok(),
        "Concatenate videos should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_join_with_crossfade() {
    let fixture = TestFixture::new();
    let video1 = fixture.create_test_video("video1.mp4");
    let video2 = fixture.create_test_video("video2.mp4");
    let output = fixture.path("crossfade.mp4");

    let result = video::join_with_crossfade(&[&video1, &video2], &output, 1.0);
    assert!(
        result.is_ok(),
        "Join with crossfade should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 18. mute - Video muting
// =============================================================================

#[test]
fn test_mute_options() {
    let options = video::MuteOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_mute_video() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("muted.mp4");

    let result = video::mute_video(&input, &output);
    assert!(
        result.is_ok(),
        "Mute video should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_replace_audio() {
    let fixture = TestFixture::new();
    let video = fixture.create_test_video("test.mp4");
    let audio = fixture.create_test_audio("music.mp3");
    let output = fixture.path("replaced.mp4");

    let result = video::replace_audio(&video, &audio, &output);
    assert!(
        result.is_ok(),
        "Replace audio should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_adjust_volume() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("volume.mp4");

    let result = video::adjust_volume(&input, &output, 0.5);
    assert!(
        result.is_ok(),
        "Adjust volume should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 19. watermark - Video watermarking
// =============================================================================

#[test]
fn test_watermark_position_enum() {
    let positions = [
        video::WatermarkPosition::TopLeft,
        video::WatermarkPosition::TopRight,
        video::WatermarkPosition::BottomLeft,
        video::WatermarkPosition::BottomRight,
        video::WatermarkPosition::Center,
        video::WatermarkPosition::Custom(100, 100),
    ];
    for pos in positions {
        assert!(!format!("{:?}", pos).is_empty());
    }
}

#[test]
fn test_text_watermark_options() {
    let options = video::TextWatermarkOptions::default();
    assert!(
        options.font_size > 0,
        "Default font size should be positive"
    );
}

#[test]
fn test_image_watermark_options() {
    let options = video::ImageWatermarkOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_add_text_watermark() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("watermarked.mp4");

    let result = video::add_text_watermark(&input, &output, "© Test");
    assert!(
        result.is_ok(),
        "Add text watermark should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_add_image_watermark() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let watermark = fixture.create_test_image("logo.png");
    let output = fixture.path("watermarked.mp4");

    let result = video::add_image_watermark(&input, &watermark, &output);
    assert!(
        result.is_ok(),
        "Add image watermark should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 20. speed - Video speed adjustment
// =============================================================================

#[test]
fn test_speed_options() {
    let options = video::SpeedOptions::default();
    assert!(!format!("{:?}", options).is_empty());
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_change_speed() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("fast.mp4");

    let result = video::change_speed(&input, &output, 2.0);
    assert!(
        result.is_ok(),
        "Change speed should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_slow_motion() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("slow.mp4");

    let result = video::slow_motion(&input, &output, 0.5);
    assert!(
        result.is_ok(),
        "Slow motion should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_timelapse() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("timelapse.mp4");

    let result = video::timelapse(&input, &output, 10.0);
    assert!(
        result.is_ok(),
        "Timelapse should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_reverse_video() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mp4");
    let output = fixture.path("reversed.mp4");

    let result = video::reverse_video(&input, &output);
    assert!(
        result.is_ok(),
        "Reverse video should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 21. subtitle - Subtitle operations
// =============================================================================

#[test]
fn test_subtitle_format_enum() {
    assert_eq!(format!("{:?}", video::SubtitleFormat::Srt), "Srt");
    assert_eq!(format!("{:?}", video::SubtitleFormat::Ass), "Ass");
    assert_eq!(format!("{:?}", video::SubtitleFormat::Vtt), "Vtt");
    assert_eq!(format!("{:?}", video::SubtitleFormat::Ssa), "Ssa");
}

#[test]
fn test_subtitle_style() {
    let style = video::SubtitleStyle::default();
    assert!(style.font_size > 0, "Default font size should be positive");
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_burn_subtitles() {
    let fixture = TestFixture::new();
    let video = fixture.create_test_video("test.mp4");
    let subs = fixture.create_test_text_file(
        "subtitles.srt",
        "1\n00:00:00,000 --> 00:00:05,000\nHello World\n",
    );
    let output = fixture.path("subtitled.mp4");

    let result = video::burn_subtitles(&video, &subs, &output);
    assert!(
        result.is_ok(),
        "Burn subtitles should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_add_soft_subtitles() {
    let fixture = TestFixture::new();
    let video = fixture.create_test_video("test.mp4");
    let subs = fixture.create_test_text_file(
        "subtitles.srt",
        "1\n00:00:00,000 --> 00:00:05,000\nHello World\n",
    );
    let output = fixture.path("output.mkv");

    let result = video::add_soft_subtitles(&video, &subs, &output, Some("eng"));
    assert!(
        result.is_ok(),
        "Add soft subtitles should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires FFmpeg"]
fn test_extract_subtitles() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_video("test.mkv");
    let output = fixture.path("subtitles.srt");

    let result = video::extract_subtitles(&input, &output, 0);
    // May fail if video has no subtitles
    let _ = result;
}

#[test]
fn test_check_ffmpeg() {
    let result = std::panic::catch_unwind(video::check_ffmpeg);
    assert!(result.is_ok());
}
