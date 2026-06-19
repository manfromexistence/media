//! Audio metadata reader/writer.
//!
//! Read and edit audio file tags (ID3, Vorbis comments, etc.)

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::collections::HashMap;
use std::path::Path;

/// Audio file metadata.
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    /// Track title.
    pub title: Option<String>,
    /// Artist name.
    pub artist: Option<String>,
    /// Album name.
    pub album: Option<String>,
    /// Album artist.
    pub album_artist: Option<String>,
    /// Track number.
    pub track: Option<u32>,
    /// Total tracks.
    pub total_tracks: Option<u32>,
    /// Disc number.
    pub disc: Option<u32>,
    /// Release year.
    pub year: Option<u32>,
    /// Genre.
    pub genre: Option<String>,
    /// Composer.
    pub composer: Option<String>,
    /// Duration in seconds.
    pub duration: Option<f64>,
    /// Bitrate in kbps.
    pub bitrate: Option<u32>,
    /// Sample rate in Hz.
    pub sample_rate: Option<u32>,
    /// Number of channels.
    pub channels: Option<u8>,
    /// Codec name.
    pub codec: Option<String>,
    /// Additional tags.
    pub extra: HashMap<String, String>,
}

impl AudioMetadata {
    /// Check if metadata is empty.
    pub fn is_empty(&self) -> bool {
        self.title.is_none() && self.artist.is_none() && self.album.is_none()
    }
}

/// Read audio file metadata.
///
/// # Arguments
/// * `input` - Path to audio file
///
/// # Example
/// ```no_run
/// use dx_media::tools::audio::read_metadata;
///
/// let metadata = read_metadata("song.mp3").unwrap();
/// println!("Title: {:?}", metadata.title);
/// println!("Artist: {:?}", metadata.artist);
/// ```
pub fn read_metadata<P: AsRef<Path>>(input: P) -> Result<AudioMetadata> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    let mut cmd = crate::tools::ffprobe::command();
    cmd.arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(input_path);

    let output = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run ffprobe: {}", e),
        source: None,
    })?;
    crate::tools::ffprobe::ensure_success(&output, "reading audio metadata")?;

    let json_str = String::from_utf8_lossy(&output.stdout);

    // Parse JSON response
    parse_ffprobe_json(&json_str)
}

/// Parse ffprobe JSON output into AudioMetadata.
fn parse_ffprobe_json(json: &str) -> Result<AudioMetadata> {
    let parsed: serde_json::Value = serde_json::from_str(json).map_err(|e| DxError::Config {
        message: format!("Failed to parse metadata JSON: {}", e),
        source: None,
    })?;

    let mut metadata = AudioMetadata::default();

    // Parse format section
    if let Some(format) = parsed.get("format") {
        if let Some(tags) = format.get("tags") {
            metadata.title = tags
                .get("title")
                .or(tags.get("TITLE"))
                .and_then(|v| v.as_str())
                .map(String::from);
            metadata.artist = tags
                .get("artist")
                .or(tags.get("ARTIST"))
                .and_then(|v| v.as_str())
                .map(String::from);
            metadata.album = tags
                .get("album")
                .or(tags.get("ALBUM"))
                .and_then(|v| v.as_str())
                .map(String::from);
            metadata.album_artist = tags
                .get("album_artist")
                .or(tags.get("ALBUMARTIST"))
                .and_then(|v| v.as_str())
                .map(String::from);
            metadata.genre = tags
                .get("genre")
                .or(tags.get("GENRE"))
                .and_then(|v| v.as_str())
                .map(String::from);
            metadata.composer = tags
                .get("composer")
                .or(tags.get("COMPOSER"))
                .and_then(|v| v.as_str())
                .map(String::from);

            // Parse track number
            if let Some(track_str) = tags
                .get("track")
                .or(tags.get("TRACK"))
                .and_then(|v| v.as_str())
            {
                if let Some(num) = track_str.split('/').next() {
                    metadata.track = num.parse().ok();
                }
                if let Some(total) = track_str.split('/').nth(1) {
                    metadata.total_tracks = total.parse().ok();
                }
            }

            // Parse year
            metadata.year = tags
                .get("date")
                .or(tags.get("DATE"))
                .or(tags.get("year"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.chars().take(4).collect::<String>().parse().ok());
        }

        // Parse format info
        metadata.duration = format
            .get("duration")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok());

        metadata.bitrate = format
            .get("bit_rate")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok())
            .map(|b| b / 1000);
    }

    // Parse stream info (audio)
    if let Some(streams) = parsed.get("streams").and_then(|s| s.as_array()) {
        for stream in streams {
            if stream.get("codec_type").and_then(|v| v.as_str()) == Some("audio") {
                metadata.sample_rate = stream
                    .get("sample_rate")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok());

                metadata.channels = stream
                    .get("channels")
                    .and_then(|v| v.as_u64())
                    .map(|c| c as u8);

                metadata.codec = stream
                    .get("codec_name")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                break;
            }
        }
    }

    Ok(metadata)
}

/// Write metadata to audio file.
pub fn write_metadata<P: AsRef<Path>>(
    input: P,
    output: P,
    metadata: &AudioMetadata,
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

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y").arg("-i").arg(input_path);

    // Add metadata tags
    if let Some(ref title) = metadata.title {
        cmd.arg("-metadata").arg(format!("title={}", title));
    }
    if let Some(ref artist) = metadata.artist {
        cmd.arg("-metadata").arg(format!("artist={}", artist));
    }
    if let Some(ref album) = metadata.album {
        cmd.arg("-metadata").arg(format!("album={}", album));
    }
    if let Some(ref album_artist) = metadata.album_artist {
        cmd.arg("-metadata")
            .arg(format!("album_artist={}", album_artist));
    }
    if let Some(track) = metadata.track {
        let track_str = if let Some(total) = metadata.total_tracks {
            format!("{}/{}", track, total)
        } else {
            track.to_string()
        };
        cmd.arg("-metadata").arg(format!("track={}", track_str));
    }
    if let Some(year) = metadata.year {
        cmd.arg("-metadata").arg(format!("date={}", year));
    }
    if let Some(ref genre) = metadata.genre {
        cmd.arg("-metadata").arg(format!("genre={}", genre));
    }
    if let Some(ref composer) = metadata.composer {
        cmd.arg("-metadata").arg(format!("composer={}", composer));
    }

    // Add extra tags
    for (key, value) in &metadata.extra {
        cmd.arg("-metadata").arg(format!("{}={}", key, value));
    }

    cmd.arg("-c").arg("copy").arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Metadata write failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Updated audio metadata",
        output_path,
    ))
}

/// Strip all metadata from audio file.
pub fn strip_metadata<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-map_metadata")
        .arg("-1")
        .arg("-c")
        .arg("copy")
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Metadata strip failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Stripped all metadata",
        output_path,
    ))
}

/// Copy metadata from one file to another.
pub fn copy_metadata<P: AsRef<Path>>(source: P, target: P, output: P) -> Result<ToolOutput> {
    let source_metadata = read_metadata(source)?;
    write_metadata(target, output, &source_metadata)
}

/// Add cover art to audio file.
pub fn add_cover_art<P: AsRef<Path>>(input: P, cover: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let cover_path = cover.as_ref();
    let output_path = output.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    if !cover_path.exists() {
        return Err(DxError::FileIo {
            path: cover_path.to_path_buf(),
            message: "Cover image not found".to_string(),
            source: None,
        });
    }

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-i")
        .arg(cover_path)
        .arg("-map")
        .arg("0:a")
        .arg("-map")
        .arg("1:v")
        .arg("-c:a")
        .arg("copy")
        .arg("-c:v")
        .arg("mjpeg")
        .arg("-metadata:s:v")
        .arg("title=Album cover")
        .arg("-metadata:s:v")
        .arg("comment=Cover (front)")
        .arg("-disposition:v")
        .arg("attached_pic")
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Adding cover art failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Added cover art",
        output_path,
    ))
}

/// Extract cover art from audio file.
pub fn extract_cover_art<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-an") // No audio
        .arg("-c:v")
        .arg("copy")
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Cover art extraction failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Extracted cover art",
        output_path,
    ))
}

/// Format metadata as displayable string.
pub fn format_metadata(metadata: &AudioMetadata) -> String {
    let mut lines = Vec::new();

    if let Some(ref title) = metadata.title {
        lines.push(format!("Title: {}", title));
    }
    if let Some(ref artist) = metadata.artist {
        lines.push(format!("Artist: {}", artist));
    }
    if let Some(ref album) = metadata.album {
        lines.push(format!("Album: {}", album));
    }
    if let Some(year) = metadata.year {
        lines.push(format!("Year: {}", year));
    }
    if let Some(ref genre) = metadata.genre {
        lines.push(format!("Genre: {}", genre));
    }
    if let Some(track) = metadata.track {
        let track_str = if let Some(total) = metadata.total_tracks {
            format!("{}/{}", track, total)
        } else {
            track.to_string()
        };
        lines.push(format!("Track: {}", track_str));
    }
    if let Some(duration) = metadata.duration {
        let mins = (duration / 60.0) as u32;
        let secs = (duration % 60.0) as u32;
        lines.push(format!("Duration: {}:{:02}", mins, secs));
    }
    if let Some(bitrate) = metadata.bitrate {
        lines.push(format!("Bitrate: {} kbps", bitrate));
    }
    if let Some(sample_rate) = metadata.sample_rate {
        lines.push(format!("Sample Rate: {} Hz", sample_rate));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_format() {
        let mut metadata = AudioMetadata::default();
        metadata.title = Some("Test Song".to_string());
        metadata.artist = Some("Test Artist".to_string());
        metadata.duration = Some(180.0);

        let formatted = format_metadata(&metadata);
        assert!(formatted.contains("Test Song"));
        assert!(formatted.contains("3:00"));
    }
}
