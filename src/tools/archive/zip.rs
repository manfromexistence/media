//! ZIP archive operations.
//!
//! Create and extract ZIP archives.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Compression level for ZIP.
#[derive(Debug, Clone, Copy, Default)]
pub enum ZipLevel {
    /// No compression (store only).
    Store,
    /// Fast compression.
    Fast,
    /// Normal compression.
    #[default]
    Normal,
    /// Best compression.
    Best,
}

impl ZipLevel {
    /// Get numeric level (0-9).
    pub fn level(&self) -> u32 {
        match self {
            ZipLevel::Store => 0,
            ZipLevel::Fast => 1,
            ZipLevel::Normal => 6,
            ZipLevel::Best => 9,
        }
    }
}

/// ZIP creation options.
#[derive(Debug, Clone)]
pub struct ZipOptions {
    /// Compression level.
    pub level: ZipLevel,
    /// Include hidden files.
    pub include_hidden: bool,
    /// Preserve directory structure.
    pub preserve_structure: bool,
    /// Password for encryption.
    pub password: Option<String>,
}

impl Default for ZipOptions {
    fn default() -> Self {
        Self {
            level: ZipLevel::Normal,
            include_hidden: false,
            preserve_structure: true,
            password: None,
        }
    }
}

/// Create ZIP archive.
///
/// # Arguments
/// * `inputs` - Files and directories to add
/// * `output` - Output ZIP path
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::zip;
///
/// zip::create_zip(&["file1.txt", "dir/"], "archive.zip").unwrap();
/// ```
pub fn create_zip<P: AsRef<Path>>(inputs: &[P], output: P) -> Result<ToolOutput> {
    create_zip_with_options(inputs, output, ZipOptions::default())
}

/// Create ZIP with options.
pub fn create_zip_with_options<P: AsRef<Path>>(
    inputs: &[P],
    output: P,
    options: ZipOptions,
) -> Result<ToolOutput> {
    let output_path = output.as_ref();

    // Try system zip first
    if let Ok(result) = create_with_zip(inputs, output_path, &options) {
        return Ok(result);
    }

    // Try 7z
    if let Ok(result) = create_with_7z(inputs, output_path, &options) {
        return Ok(result);
    }

    // Try PowerShell on Windows
    #[cfg(windows)]
    if let Ok(result) = create_with_powershell(inputs, output_path) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "ZIP creation failed. Install zip or 7z.".to_string(),
        source: None,
    })
}

/// Create using system zip command.
fn create_with_zip<P: AsRef<Path>>(
    inputs: &[P],
    output: &Path,
    options: &ZipOptions,
) -> Result<ToolOutput> {
    let mut cmd = Command::new("zip");
    cmd.arg("-r").arg(format!("-{}", options.level.level()));

    if let Some(ref password) = options.password {
        cmd.arg("-P").arg(password);
    }

    cmd.arg(output);

    for input in inputs {
        cmd.arg(input.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run zip: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!("zip failed: {}", String::from_utf8_lossy(&result.stderr)),
            source: None,
        });
    }

    let size = std::fs::metadata(output).map_or(0, |m| m.len());

    Ok(ToolOutput::success_with_path(
        format!("Created ZIP archive ({} bytes)", size),
        output,
    ))
}

/// Create using 7z command.
fn create_with_7z<P: AsRef<Path>>(
    inputs: &[P],
    output: &Path,
    options: &ZipOptions,
) -> Result<ToolOutput> {
    let mut cmd = Command::new("7z");
    cmd.arg("a").arg(format!("-mx={}", options.level.level()));

    if let Some(ref password) = options.password {
        cmd.arg(format!("-p{}", password));
    }

    cmd.arg(output);

    for input in inputs {
        cmd.arg(input.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "7z failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Created ZIP archive with 7z",
        output,
    ))
}

/// Create using PowerShell.
#[cfg(windows)]
fn create_with_powershell<P: AsRef<Path>>(inputs: &[P], output: &Path) -> Result<ToolOutput> {
    // Build file list
    let files: Vec<String> = inputs
        .iter()
        .map(|p| p.as_ref().to_string_lossy().to_string())
        .collect();

    let script = format!(
        "Compress-Archive -Path '{}' -DestinationPath '{}'",
        files.join("','"),
        output.to_string_lossy()
    );

    let mut cmd = Command::new("powershell");
    cmd.arg("-Command").arg(&script);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run PowerShell: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "PowerShell compression failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Created ZIP archive with PowerShell",
        output,
    ))
}

/// Extract ZIP archive.
///
/// # Arguments
/// * `input` - ZIP file path
/// * `output_dir` - Directory to extract to
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::zip;
///
/// zip::extract_zip("archive.zip", "extracted/").unwrap();
/// ```
pub fn extract_zip<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    extract_zip_with_password(input, output_dir, None)
}

/// Extract ZIP with password.
pub fn extract_zip_with_password<P: AsRef<Path>>(
    input: P,
    output_dir: P,
    password: Option<&str>,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create directory: {}", e),
        source: None,
    })?;

    // Try unzip
    if let Ok(result) = extract_with_unzip(input_path, output_dir, password) {
        return Ok(result);
    }

    // Try 7z
    if let Ok(result) = extract_with_7z(input_path, output_dir, password) {
        return Ok(result);
    }

    // Try PowerShell on Windows
    #[cfg(windows)]
    if password.is_none() {
        if let Ok(result) = extract_with_powershell(input_path, output_dir) {
            return Ok(result);
        }
    }

    Err(DxError::Config {
        message: "ZIP extraction failed. Install unzip or 7z.".to_string(),
        source: None,
    })
}

/// Extract using unzip command.
fn extract_with_unzip(
    input: &Path,
    output_dir: &Path,
    password: Option<&str>,
) -> Result<ToolOutput> {
    let mut cmd = Command::new("unzip");
    cmd.arg("-o"); // Overwrite

    if let Some(pwd) = password {
        cmd.arg("-P").arg(pwd);
    }

    cmd.arg(input).arg("-d").arg(output_dir);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run unzip: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "unzip failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Extracted ZIP archive",
        output_dir,
    ))
}

/// Extract using 7z command.
fn extract_with_7z(input: &Path, output_dir: &Path, password: Option<&str>) -> Result<ToolOutput> {
    let mut cmd = Command::new("7z");
    cmd.arg("x").arg("-y");

    if let Some(pwd) = password {
        cmd.arg(format!("-p{}", pwd));
    }

    cmd.arg(format!("-o{}", output_dir.to_string_lossy()))
        .arg(input);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "7z extraction failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Extracted ZIP archive with 7z",
        output_dir,
    ))
}

/// Extract using PowerShell.
#[cfg(windows)]
fn extract_with_powershell(input: &Path, output_dir: &Path) -> Result<ToolOutput> {
    let script = format!(
        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
        input.to_string_lossy(),
        output_dir.to_string_lossy()
    );

    let mut cmd = Command::new("powershell");
    cmd.arg("-Command").arg(&script);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run PowerShell: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "PowerShell extraction failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Extracted ZIP archive with PowerShell",
        output_dir,
    ))
}

/// Add files to existing ZIP.
pub fn add_to_zip<P: AsRef<Path>>(archive: P, files: &[P]) -> Result<ToolOutput> {
    let archive_path = archive.as_ref();

    if !archive_path.exists() {
        return Err(DxError::FileIo {
            path: archive_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    let mut cmd = Command::new("zip");
    cmd.arg("-u").arg(archive_path);

    for file in files {
        cmd.arg(file.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run zip: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Failed to add files to archive".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Added {} files to archive", files.len()),
        archive_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_level() {
        assert_eq!(ZipLevel::Store.level(), 0);
        assert_eq!(ZipLevel::Best.level(), 9);
    }
}
