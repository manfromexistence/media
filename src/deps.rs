//! Dependency checking for external tools.
//!
//! This module provides functionality to check for external dependencies
//! like FFmpeg, ImageMagick, and Tesseract that are required by various tools.

use crate::error::DxError;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::process::Command;

const MAGICK_ENV: &str = "DX_MEDIA_MAGICK_BIN";

/// External dependency information.
#[derive(Debug, Clone)]
pub struct DependencyInfo {
    /// Human-readable name of the dependency.
    pub name: &'static str,
    /// Binary name to check for.
    pub binary: &'static str,
    /// Flag to get version information.
    pub version_flag: &'static str,
    /// Installation hint for the user.
    pub install_hint: &'static str,
    /// Tools that require this dependency.
    pub required_by: &'static [&'static str],
}

/// Known external dependencies used by dx-media tools.
pub const DEPENDENCIES: &[DependencyInfo] = &[
    DependencyInfo {
        name: "FFmpeg",
        binary: "ffmpeg",
        version_flag: "-version",
        install_hint: "Install via: brew install ffmpeg (macOS), apt install ffmpeg (Ubuntu), choco install ffmpeg (Windows)",
        required_by: &[
            "video::transcode",
            "video::trim",
            "video::concatenate",
            "video::scale",
            "video::thumbnail",
            "video::gif",
            "video::watermark",
            "video::mute",
            "video::speed",
            "video::subtitle",
            "video::audio_extract",
            "audio::convert",
            "audio::trim",
            "audio::merge",
            "audio::normalize",
            "audio::analyze_levels",
            "audio::effects",
            "audio::spectrum",
            "audio::silence",
            "audio::split",
            "audio::prepare_for_transcription",
            "audio::extract_speech_segments",
        ],
    },
    DependencyInfo {
        name: "FFprobe",
        binary: "ffprobe",
        version_flag: "-version",
        install_hint: "Install with FFmpeg: brew install ffmpeg (macOS), apt install ffmpeg (Ubuntu), choco install ffmpeg (Windows)",
        required_by: &[
            "video::thumbnail",
            "video::subtitle",
            "audio::metadata",
            "audio::trim",
            "audio::split",
        ],
    },
    DependencyInfo {
        name: "ImageMagick",
        binary: "magick",
        version_flag: "-version",
        install_hint: "Install via: brew install imagemagick (macOS), apt install imagemagick (Ubuntu), choco install imagemagick (Windows)",
        required_by: &[
            "image::convert",
            "image::resize",
            "image::compress",
            "image::watermark",
            "image::filter",
            "image::icons",
        ],
    },
    DependencyInfo {
        name: "Tesseract",
        binary: "tesseract",
        version_flag: "--version",
        install_hint: "Install via: brew install tesseract (macOS), apt install tesseract-ocr (Ubuntu), choco install tesseract (Windows)",
        required_by: &["image::ocr"],
    },
    DependencyInfo {
        name: "ExifTool",
        binary: "exiftool",
        version_flag: "-ver",
        install_hint: "Install via: brew install exiftool (macOS), apt install libimage-exiftool-perl (Ubuntu)",
        required_by: &["image::exif"],
    },
    DependencyInfo {
        name: "Ghostscript",
        binary: "gs",
        version_flag: "--version",
        install_hint: "Install via: brew install ghostscript (macOS), apt install ghostscript (Ubuntu), choco install ghostscript (Windows)",
        required_by: &["document::pdf_compress", "document::pdf_to_image"],
    },
    DependencyInfo {
        name: "7-Zip",
        binary: "7z",
        version_flag: "",
        install_hint: "Install via: brew install p7zip (macOS), apt install p7zip-full (Ubuntu), choco install 7zip (Windows)",
        required_by: &["archive::7z"],
    },
    DependencyInfo {
        name: "UnRAR",
        binary: "unrar",
        version_flag: "",
        install_hint: "Install via: brew install unrar (macOS), apt install unrar (Ubuntu), choco install unrar (Windows)",
        required_by: &["archive::rar"],
    },
];

/// Result of checking a single dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCheckResult {
    /// Name of the dependency.
    pub name: String,
    /// Whether the dependency is available.
    pub available: bool,
    /// Version string if available.
    pub version: Option<String>,
    /// Error message if not available.
    pub error: Option<String>,
}

/// Report of all dependency checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyReport {
    /// Results for each dependency.
    pub results: Vec<DependencyCheckResult>,
}

impl DependencyReport {
    /// Check if all dependencies are available.
    #[must_use]
    pub fn all_available(&self) -> bool {
        self.results.iter().all(|r| r.available)
    }

    /// Get list of missing dependency names.
    #[must_use]
    pub fn missing(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter(|r| !r.available)
            .map(|r| r.name.as_str())
            .collect()
    }

    /// Get list of available dependency names.
    #[must_use]
    pub fn available(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter(|r| r.available)
            .map(|r| r.name.as_str())
            .collect()
    }

    /// Get the count of available dependencies.
    #[must_use]
    pub fn available_count(&self) -> usize {
        self.results.iter().filter(|r| r.available).count()
    }

    /// Get the total count of dependencies.
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.results.len()
    }
}

/// Check if a specific dependency is available.
///
/// # Arguments
///
/// * `dep` - The dependency information to check.
///
/// # Returns
///
/// Returns `Ok(version_string)` if the dependency is available,
/// or `Err(DxError::MissingDependency)` if not found.
///
/// # Examples
///
/// ```rust,no_run
/// use dx_media::deps::{check_dependency, DEPENDENCIES};
///
/// let ffmpeg = &DEPENDENCIES[0]; // FFmpeg
/// match check_dependency(ffmpeg) {
///     Ok(version) => println!("FFmpeg version: {}", version),
///     Err(e) => println!("FFmpeg not found: {}", e),
/// }
/// ```
pub fn check_dependency(dep: &DependencyInfo) -> Result<String, DxError> {
    let mut cmd = Command::new(dependency_binary(dep));

    // Only add version flag if it's not empty
    if !dep.version_flag.is_empty() {
        cmd.arg(dep.version_flag);
    }

    match cmd.output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(version) = first_non_empty_line(&stdout) {
                return Ok(version.to_string());
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            if let Some(version) = first_non_empty_line(&stderr) {
                return Ok(version.to_string());
            }

            Ok("unknown version".to_string())
        }
        Ok(_) => Err(DxError::missing_dependency(dep.name, dep.install_hint)),
        Err(_) => Err(DxError::missing_dependency(dep.name, dep.install_hint)),
    }
}

fn dependency_binary(dep: &DependencyInfo) -> OsString {
    match dep.binary {
        "ffmpeg" => crate::tools::ffmpeg::binary(),
        "ffprobe" => crate::tools::ffprobe::binary(),
        "magick" => image_magick_binary(),
        binary => OsString::from(binary),
    }
}

pub(crate) fn image_magick_binary() -> OsString {
    std::env::var_os(MAGICK_ENV).unwrap_or_else(|| OsString::from("magick"))
}

fn first_non_empty_line(output: &str) -> Option<&str> {
    output.lines().find(|line| !line.trim().is_empty())
}

/// Check all known dependencies and return a report.
///
/// # Returns
///
/// A `DependencyReport` containing the status of all dependencies.
///
/// # Examples
///
/// ```rust,no_run
/// use dx_media::deps::check_all_dependencies;
///
/// let report = check_all_dependencies();
/// println!("Available: {}/{}", report.available_count(), report.total_count());
///
/// for missing in report.missing() {
///     println!("Missing: {}", missing);
/// }
/// ```
#[must_use]
pub fn check_all_dependencies() -> DependencyReport {
    let results = DEPENDENCIES
        .iter()
        .map(|dep| match check_dependency(dep) {
            Ok(version) => DependencyCheckResult {
                name: dep.name.to_string(),
                available: true,
                version: Some(version),
                error: None,
            },
            Err(e) => DependencyCheckResult {
                name: dep.name.to_string(),
                available: false,
                version: None,
                error: Some(e.to_string()),
            },
        })
        .collect();

    DependencyReport { results }
}

/// Find the dependency info for a given tool name.
///
/// # Arguments
///
/// * `tool_name` - The tool name (e.g., "video::transcode", "image::ocr").
///
/// # Returns
///
/// The `DependencyInfo` if found, or `None` if no dependency is required.
#[must_use]
pub fn find_dependency_for_tool(tool_name: &str) -> Option<&'static DependencyInfo> {
    DEPENDENCIES
        .iter()
        .find(|dep| dep.required_by.contains(&tool_name))
}

/// Check if a specific tool's dependency is available.
///
/// # Arguments
///
/// * `tool_name` - The tool name (e.g., "video::transcode", "image::ocr").
///
/// # Returns
///
/// `Ok(())` if the dependency is available or no dependency is required,
/// `Err(DxError::MissingDependency)` if the required dependency is missing.
pub fn check_tool_dependency(tool_name: &str) -> Result<(), DxError> {
    if let Some(dep) = find_dependency_for_tool(tool_name) {
        check_dependency(dep)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_dependency_info_structure() {
        // Verify all dependencies have required fields
        for dep in DEPENDENCIES {
            assert!(!dep.name.is_empty(), "Dependency name should not be empty");
            assert!(!dep.binary.is_empty(), "Binary name should not be empty");
            assert!(
                !dep.install_hint.is_empty(),
                "Install hint should not be empty"
            );
            assert!(
                !dep.required_by.is_empty(),
                "Required by should not be empty"
            );
        }
    }

    #[test]
    fn test_check_all_dependencies_returns_report() {
        let report = check_all_dependencies();

        // Should have results for all dependencies
        assert_eq!(report.results.len(), DEPENDENCIES.len());

        // Each result should have a name
        for result in &report.results {
            assert!(!result.name.is_empty());
        }
    }

    #[test]
    fn test_dependency_report_methods() {
        let report = DependencyReport {
            results: vec![
                DependencyCheckResult {
                    name: "Test1".to_string(),
                    available: true,
                    version: Some("1.0".to_string()),
                    error: None,
                },
                DependencyCheckResult {
                    name: "Test2".to_string(),
                    available: false,
                    version: None,
                    error: Some("Not found".to_string()),
                },
            ],
        };

        assert!(!report.all_available());
        assert_eq!(report.missing(), vec!["Test2"]);
        assert_eq!(report.available(), vec!["Test1"]);
        assert_eq!(report.available_count(), 1);
        assert_eq!(report.total_count(), 2);
    }

    #[test]
    fn test_find_dependency_for_tool() {
        // FFmpeg tools
        let dep = find_dependency_for_tool("video::transcode");
        assert!(dep.is_some());
        assert_eq!(dep.unwrap().name, "FFmpeg");

        let dep = find_dependency_for_tool("audio::extract_speech_segments");
        assert!(dep.is_some());
        assert_eq!(dep.unwrap().name, "FFmpeg");

        let dep = find_dependency_for_tool("audio::prepare_for_transcription");
        assert!(dep.is_some());
        assert_eq!(dep.unwrap().name, "FFmpeg");

        let dep = find_dependency_for_tool("audio::analyze_levels");
        assert!(dep.is_some());
        assert_eq!(dep.unwrap().name, "FFmpeg");

        // ImageMagick tools
        let dep = find_dependency_for_tool("image::convert");
        assert!(dep.is_some());
        assert_eq!(dep.unwrap().name, "ImageMagick");

        // Tesseract tools
        let dep = find_dependency_for_tool("image::ocr");
        assert!(dep.is_some());
        assert_eq!(dep.unwrap().name, "Tesseract");

        // Unknown tool
        let dep = find_dependency_for_tool("unknown::tool");
        assert!(dep.is_none());
    }

    #[test]
    fn test_check_tool_dependency_unknown_tool() {
        // Unknown tools should return Ok (no dependency required)
        let result = check_tool_dependency("unknown::tool");
        assert!(result.is_ok());
    }

    #[test]
    fn dependency_checker_rejects_nonzero_stdout() {
        let binary = fake_dependency_binary("nonzero_stdout", FakeDependencyOutput::Stdout);
        let dep = fake_dependency_info(leak_path(binary));

        assert!(matches!(
            check_dependency(&dep),
            Err(DxError::MissingDependency { .. })
        ));
    }

    #[test]
    fn dependency_checker_rejects_nonzero_stderr() {
        let binary = fake_dependency_binary("nonzero_stderr", FakeDependencyOutput::Stderr);
        let dep = fake_dependency_info(leak_path(binary));

        assert!(matches!(
            check_dependency(&dep),
            Err(DxError::MissingDependency { .. })
        ));
    }

    fn fake_dependency_info(binary: &'static str) -> DependencyInfo {
        DependencyInfo {
            name: "Fake FFmpeg",
            binary,
            version_flag: "-version",
            install_hint: "Install fake ffmpeg",
            required_by: &["video::fake"],
        }
    }

    enum FakeDependencyOutput {
        Stdout,
        Stderr,
    }

    fn fake_dependency_binary(name: &str, output: FakeDependencyOutput) -> PathBuf {
        let dir = tempfile::tempdir().expect("temp dir should be created");

        #[cfg(windows)]
        {
            let path = dir.path().join(format!("{name}.cmd"));
            let command = match output {
                FakeDependencyOutput::Stdout => "@echo fake ffmpeg version\n@exit /b 1\n",
                FakeDependencyOutput::Stderr => "@echo fake ffmpeg version 1>&2\n@exit /b 1\n",
            };
            fs::write(&path, command).expect("fake command should be written");
            std::mem::forget(dir);
            path
        }

        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = dir.path().join(name);
            let command = match output {
                FakeDependencyOutput::Stdout => "#!/bin/sh\necho 'fake ffmpeg version'\nexit 1\n",
                FakeDependencyOutput::Stderr => {
                    "#!/bin/sh\necho 'fake ffmpeg version' >&2\nexit 1\n"
                }
            };
            fs::write(&path, command).expect("fake command should be written");
            let mut permissions = fs::metadata(&path)
                .expect("fake command metadata should be readable")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).expect("fake command should be executable");
            std::mem::forget(dir);
            path
        }
    }

    fn leak_path(path: PathBuf) -> &'static str {
        Box::leak(path.to_string_lossy().to_string().into_boxed_str())
    }
}
