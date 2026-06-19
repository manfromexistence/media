//! Common test utilities.

#![allow(dead_code)]

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn ffmpeg_command() -> Command {
    Command::new(
        std::env::var_os("DX_MEDIA_FFMPEG_BIN").unwrap_or_else(|| OsString::from("ffmpeg")),
    )
}

/// Test fixture for managing temporary files.
pub struct TestFixture {
    pub temp_dir: TempDir,
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("Failed to create temp dir"),
        }
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.temp_dir.path().join(name)
    }

    pub fn create_temp_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.path(name);
        fs::write(&path, content).expect("Failed to write temp file");
        path
    }

    /// Create a real test image using the `image` crate.
    /// Creates a 100x100 PNG with a gradient pattern.
    #[cfg(feature = "image-core")]
    pub fn create_test_image(&self, name: &str) -> PathBuf {
        use image::{ImageBuffer, Rgb};

        let path = self.path(name);

        // Create a 100x100 RGB image with a gradient
        let img = ImageBuffer::from_fn(100, 100, |x, y| {
            let r = (x * 255 / 100) as u8;
            let g = (y * 255 / 100) as u8;
            let b = 128;
            Rgb([r, g, b])
        });

        img.save(&path).expect("Failed to save test image");
        path
    }

    /// Fallback for when image-core feature is not enabled.
    #[cfg(not(feature = "image-core"))]
    pub fn create_test_image(&self, name: &str) -> PathBuf {
        // Create a minimal valid PNG file
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49,
            0x44, 0x41, 0x54, // IDAT chunk
            0x08, 0x99, 0x63, 0xF8, 0xFF, 0xFF, 0x3F, 0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC,
            0x59, 0xE7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
            0xAE, 0x42, 0x60, 0x82,
        ];
        self.create_temp_file(name, &png_data)
    }

    /// Create a test text file.
    pub fn create_test_text_file(&self, name: &str, content: &str) -> PathBuf {
        self.create_temp_file(name, content.as_bytes())
    }

    /// Create a real test audio file using FFmpeg.
    /// Generates a 1-second sine wave at 440Hz (A4 note).
    pub fn create_test_audio(&self, name: &str) -> PathBuf {
        let path = self.path(name);

        // Use FFmpeg to generate a test audio file
        let output = ffmpeg_command()
            .args([
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=1",
                "-y",
                path.to_str().unwrap(),
            ])
            .output();

        match output {
            Ok(result) if result.status.success() => path,
            _ => {
                // Fallback: create a minimal WAV file header
                let wav_data = self.create_minimal_wav();
                self.create_temp_file(name, &wav_data)
            }
        }
    }

    /// Create a real test video file using FFmpeg.
    /// Generates a 2-second video with a color pattern AND audio.
    pub fn create_test_video(&self, name: &str) -> PathBuf {
        let path = self.path(name);

        // Use FFmpeg to generate a test video file with audio
        let output = ffmpeg_command()
            .args([
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=2:size=320x240:rate=30",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=2",
                "-pix_fmt",
                "yuv420p",
                "-y",
                path.to_str().unwrap(),
            ])
            .output();

        match output {
            Ok(result) if result.status.success() => path,
            _ => {
                // Fallback: create a placeholder file
                eprintln!("Warning: FFmpeg not available, creating placeholder video");
                self.create_temp_file(name, b"placeholder video")
            }
        }
    }

    /// Create a minimal valid WAV file (1 second of silence).
    fn create_minimal_wav(&self) -> Vec<u8> {
        let sample_rate = 44100u32;
        let num_samples = sample_rate; // 1 second
        let num_channels = 1u16;
        let bits_per_sample = 16u16;
        let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample) / 8;
        let block_align = num_channels * bits_per_sample / 8;
        let data_size = num_samples * u32::from(block_align);

        let mut wav = Vec::new();

        // RIFF header
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_size).to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // audio format (PCM)
        wav.extend_from_slice(&num_channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());

        // Silent audio data (all zeros)
        wav.resize(wav.len() + data_size as usize, 0);

        wav
    }

    /// Check if FFmpeg is available.
    pub fn has_ffmpeg() -> bool {
        ffmpeg_command()
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Check if ImageMagick is available.
    pub fn has_imagemagick() -> bool {
        std::process::Command::new("magick")
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
