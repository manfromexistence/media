//! Video processing tools.
//!
//! This module provides 10 video manipulation tools:
//! 1. Format Transcoder - Convert between video formats
//! 2. Audio Extractor - Extract audio from video
//! 3. Video Trimmer - Cut video segments
//! 4. GIF Maker - Convert video clips to GIF
//! 5. Thumbnail Generator - Extract video frames
//! 6. Resolution Scaler - Change video resolution
//! 7. Video Concatenator - Join multiple videos
//! 8. Mute Video - Remove audio track
//! 9. Video Watermark - Add text/image watermarks
//! 10. Speed Changer - Adjust playback speed
//! 11. Subtitle Handler - Add/extract subtitles

mod audio_extract;
mod concatenate;
mod gif_maker;
mod mute;
mod scaler;
mod speed;
mod subtitle;
mod thumbnail;
mod transcoder;
mod trimmer;
mod watermark;

pub use audio_extract::*;
pub use concatenate::*;
pub use gif_maker::*;
pub use mute::*;
pub use scaler::*;
pub use speed::*;
pub use subtitle::*;
pub use thumbnail::*;
pub use transcoder::*;
pub use trimmer::*;
pub use watermark::*;

use crate::error::Result;
use std::path::Path;

/// Video tools collection.
pub struct VideoTools;

impl VideoTools {
    /// Create a new VideoTools instance.
    pub fn new() -> Self {
        Self
    }

    /// Transcode video to different format.
    pub fn transcode<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        options: TranscodeOptions,
    ) -> Result<super::ToolOutput> {
        transcoder::transcode_video(input, output, options)
    }

    /// Extract audio from video.
    pub fn extract_audio<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        format: AudioFormat,
    ) -> Result<super::ToolOutput> {
        audio_extract::extract_audio(input, output, format)
    }

    /// Trim video to specified duration.
    pub fn trim<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        start: f64,
        end: f64,
    ) -> Result<super::ToolOutput> {
        trimmer::trim_video(input, output, start, end)
    }

    /// Convert video to GIF.
    pub fn to_gif<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        options: GifOptions,
    ) -> Result<super::ToolOutput> {
        gif_maker::video_to_gif(input, output, options)
    }

    /// Extract thumbnail from video.
    pub fn thumbnail<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        timestamp: f64,
    ) -> Result<super::ToolOutput> {
        thumbnail::extract_thumbnail(input, output, timestamp)
    }

    /// Scale video resolution.
    pub fn scale<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        width: u32,
        height: u32,
    ) -> Result<super::ToolOutput> {
        scaler::scale_video(input, output, width, height)
    }

    /// Concatenate multiple videos.
    pub fn concatenate<P: AsRef<Path>>(
        &self,
        inputs: &[P],
        output: P,
    ) -> Result<super::ToolOutput> {
        concatenate::concatenate_videos(inputs, output)
    }

    /// Remove audio from video.
    pub fn mute<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        mute::mute_video(input, output)
    }

    /// Add text watermark to video.
    pub fn add_watermark<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        text: &str,
    ) -> Result<super::ToolOutput> {
        watermark::add_text_watermark(input, output, text)
    }

    /// Change video playback speed.
    pub fn change_speed<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        factor: f32,
    ) -> Result<super::ToolOutput> {
        speed::change_speed(input, output, factor)
    }

    /// Burn subtitles into video.
    pub fn burn_subtitles<P: AsRef<Path>>(
        &self,
        input: P,
        subtitles: P,
        output: P,
    ) -> Result<super::ToolOutput> {
        subtitle::burn_subtitles(input, subtitles, output)
    }
}

impl Default for VideoTools {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if FFmpeg is available on the system.
pub fn check_ffmpeg() -> bool {
    crate::tools::ffmpeg::command()
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get FFmpeg version string.
pub fn ffmpeg_version() -> Option<String> {
    crate::tools::ffmpeg::command()
        .arg("-version")
        .output()
        .ok()
        .and_then(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            output.lines().next().map(|s| s.to_string())
        })
}
