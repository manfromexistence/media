//! Video resolution scaling tool.
//!
//! Change video resolution (upscale or downscale).

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Common video resolutions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// 426x240 (240p)
    R240p,
    /// 640x360 (360p)
    R360p,
    /// 854x480 (480p)
    R480p,
    /// 1280x720 (720p HD)
    R720p,
    /// 1920x1080 (1080p Full HD)
    R1080p,
    /// 2560x1440 (1440p 2K)
    R1440p,
    /// 3840x2160 (2160p 4K UHD)
    R4k,
    /// Custom resolution
    Custom(u32, u32),
}

impl Resolution {
    /// Get width and height.
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::R240p => (426, 240),
            Self::R360p => (640, 360),
            Self::R480p => (854, 480),
            Self::R720p => (1280, 720),
            Self::R1080p => (1920, 1080),
            Self::R1440p => (2560, 1440),
            Self::R4k => (3840, 2160),
            Self::Custom(w, h) => (*w, *h),
        }
    }

    /// Get resolution name.
    pub fn name(&self) -> String {
        match self {
            Self::R240p => "240p".to_string(),
            Self::R360p => "360p".to_string(),
            Self::R480p => "480p".to_string(),
            Self::R720p => "720p HD".to_string(),
            Self::R1080p => "1080p Full HD".to_string(),
            Self::R1440p => "1440p 2K".to_string(),
            Self::R4k => "4K UHD".to_string(),
            Self::Custom(w, h) => format!("{}x{}", w, h),
        }
    }

    /// Parse from string (e.g., "1080p", "720", "1920x1080").
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.to_lowercase().trim().to_string();
        match s.as_str() {
            "240p" | "240" => Some(Self::R240p),
            "360p" | "360" => Some(Self::R360p),
            "480p" | "480" | "sd" => Some(Self::R480p),
            "720p" | "720" | "hd" => Some(Self::R720p),
            "1080p" | "1080" | "fullhd" | "fhd" => Some(Self::R1080p),
            "1440p" | "1440" | "2k" | "qhd" => Some(Self::R1440p),
            "2160p" | "2160" | "4k" | "uhd" => Some(Self::R4k),
            _ => {
                // Try parsing custom resolution (WxH format)
                if s.contains('x') {
                    let parts: Vec<&str> = s.split('x').collect();
                    if parts.len() == 2 {
                        let w: u32 = parts[0].parse().ok()?;
                        let h: u32 = parts[1].parse().ok()?;
                        return Some(Self::Custom(w, h));
                    }
                }
                None
            }
        }
    }
}

/// Scaling algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleAlgorithm {
    /// Bilinear (fast).
    Bilinear,
    /// Bicubic (balanced).
    #[default]
    Bicubic,
    /// Lanczos (high quality).
    Lanczos,
    /// Spline (smooth).
    Spline,
    /// Area (good for downscaling).
    Area,
}

impl ScaleAlgorithm {
    /// Get FFmpeg flag name.
    pub fn ffmpeg_flag(&self) -> &'static str {
        match self {
            Self::Bilinear => "bilinear",
            Self::Bicubic => "bicubic",
            Self::Lanczos => "lanczos",
            Self::Spline => "spline",
            Self::Area => "area",
        }
    }
}

/// Video scaling options.
#[derive(Debug, Clone)]
pub struct ScaleOptions {
    /// Target width.
    pub width: u32,
    /// Target height.
    pub height: u32,
    /// Maintain aspect ratio.
    pub keep_aspect: bool,
    /// Scaling algorithm.
    pub algorithm: ScaleAlgorithm,
    /// Video quality (CRF for x264).
    pub quality: u8,
    /// Audio bitrate.
    pub audio_bitrate: Option<String>,
}

impl Default for ScaleOptions {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            keep_aspect: true,
            algorithm: ScaleAlgorithm::default(),
            quality: 18,
            audio_bitrate: Some("128k".to_string()),
        }
    }
}

impl ScaleOptions {
    /// Create options from resolution preset.
    pub fn from_resolution(res: Resolution) -> Self {
        let (w, h) = res.dimensions();
        Self {
            width: w,
            height: h,
            ..Default::default()
        }
    }

    /// Create options with custom dimensions.
    pub fn custom(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Set scaling algorithm.
    pub fn with_algorithm(mut self, algo: ScaleAlgorithm) -> Self {
        self.algorithm = algo;
        self
    }

    /// Set quality (CRF 0-51, lower = better).
    pub fn with_quality(mut self, crf: u8) -> Self {
        self.quality = crf.clamp(0, 51);
        self
    }

    /// Force exact dimensions (may stretch).
    pub fn force_dimensions(mut self) -> Self {
        self.keep_aspect = false;
        self
    }
}

/// Scale video to specified dimensions.
///
/// # Arguments
/// * `input` - Path to input video
/// * `output` - Path for scaled output
/// * `width` - Target width
/// * `height` - Target height
///
/// # Example
/// ```no_run
/// use dx_media::tools::video::scale_video;
///
/// // Scale to 720p
/// scale_video("4k_video.mp4", "720p_video.mp4", 1280, 720).unwrap();
/// ```
pub fn scale_video<P: AsRef<Path>>(
    input: P,
    output: P,
    width: u32,
    height: u32,
) -> Result<ToolOutput> {
    scale_video_with_options(input, output, ScaleOptions::custom(width, height))
}

/// Scale video with detailed options.
pub fn scale_video_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: ScaleOptions,
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

    // Build scale filter
    let scale_filter = if options.keep_aspect {
        // Scale while maintaining aspect ratio, padding if necessary
        format!(
            "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:black,setsar=1",
            options.width, options.height, options.width, options.height
        )
    } else {
        format!(
            "scale={}:{}:flags={}",
            options.width,
            options.height,
            options.algorithm.ffmpeg_flag()
        )
    };

    let mut cmd = crate::tools::ffmpeg::command();
    cmd.arg("-y")
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(&scale_filter)
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg(options.quality.to_string())
        .arg("-preset")
        .arg("medium");

    if let Some(audio_br) = &options.audio_bitrate {
        cmd.arg("-c:a").arg("aac").arg("-b:a").arg(audio_br);
    } else {
        cmd.arg("-c:a").arg("copy");
    }

    cmd.arg(output_path);

    let output_result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run FFmpeg: {}", e),
        source: None,
    })?;

    if !output_result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Scaling failed: {}",
                String::from_utf8_lossy(&output_result.stderr)
            ),
            source: None,
        });
    }

    let output_size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    Ok(ToolOutput::success_with_path(
        format!(
            "Scaled to {}x{} ({} bytes)",
            options.width, options.height, output_size
        ),
        output_path,
    ))
}

/// Scale to a resolution preset.
pub fn scale_to_resolution<P: AsRef<Path>>(
    input: P,
    output: P,
    resolution: Resolution,
) -> Result<ToolOutput> {
    scale_video_with_options(input, output, ScaleOptions::from_resolution(resolution))
}

/// Scale to 720p.
pub fn scale_to_720p<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    scale_to_resolution(input, output, Resolution::R720p)
}

/// Scale to 1080p.
pub fn scale_to_1080p<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    scale_to_resolution(input, output, Resolution::R1080p)
}

/// Scale to 4K.
pub fn scale_to_4k<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    scale_to_resolution(input, output, Resolution::R4k)
}

/// Batch scale multiple videos.
pub fn batch_scale<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    resolution: Resolution,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut scaled = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("video");
        let extension = input_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("mp4");

        let (w, h) = resolution.dimensions();
        let output_path = output_dir.join(format!("{}_{}x{}.{}", file_stem, w, h, extension));

        if scale_to_resolution(input_path, &output_path, resolution).is_ok() {
            scaled.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!(
        "Scaled {} videos to {}",
        scaled.len(),
        resolution.name()
    ))
    .with_paths(scaled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution() {
        assert_eq!(Resolution::R1080p.dimensions(), (1920, 1080));
        assert_eq!(Resolution::R4k.dimensions(), (3840, 2160));
    }

    #[test]
    fn test_resolution_parse() {
        assert_eq!(Resolution::from_str("1080p"), Some(Resolution::R1080p));
        assert_eq!(Resolution::from_str("720"), Some(Resolution::R720p));
        assert_eq!(Resolution::from_str("4k"), Some(Resolution::R4k));
    }

    #[test]
    fn test_custom_resolution() {
        if let Some(Resolution::Custom(w, h)) = Resolution::from_str("1600x900") {
            assert_eq!(w, 1600);
            assert_eq!(h, 900);
        } else {
            panic!("Failed to parse custom resolution");
        }
    }
}
