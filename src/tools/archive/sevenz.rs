//! 7z archive operations.
//!
//! Create and extract 7z archives.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// 7z compression level.
#[derive(Debug, Clone, Copy, Default)]
pub enum SevenZipLevel {
    /// Store only (no compression).
    Store,
    /// Fast compression.
    Fast,
    /// Normal compression.
    #[default]
    Normal,
    /// Maximum compression.
    Maximum,
    /// Ultra compression.
    Ultra,
}

impl SevenZipLevel {
    /// Get numeric level (0-9).
    pub fn level(&self) -> u32 {
        match self {
            SevenZipLevel::Store => 0,
            SevenZipLevel::Fast => 1,
            SevenZipLevel::Normal => 5,
            SevenZipLevel::Maximum => 7,
            SevenZipLevel::Ultra => 9,
        }
    }
}

/// 7z archive options.
#[derive(Debug, Clone)]
pub struct SevenZipOptions {
    /// Compression level.
    pub level: SevenZipLevel,
    /// Encrypt filenames.
    pub encrypt_names: bool,
    /// Solid archive.
    pub solid: bool,
    /// Password for encryption.
    pub password: Option<String>,
}

impl Default for SevenZipOptions {
    fn default() -> Self {
        Self {
            level: SevenZipLevel::Normal,
            encrypt_names: false,
            solid: true,
            password: None,
        }
    }
}

/// Create 7z archive.
///
/// # Arguments
/// * `inputs` - Files and directories to add
/// * `output` - Output 7z path
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::sevenz;
///
/// sevenz::create_7z(&["file1.txt", "dir/"], "archive.7z").unwrap();
/// ```
pub fn create_7z<P: AsRef<Path>>(inputs: &[P], output: P) -> Result<ToolOutput> {
    create_7z_with_options(inputs, output, SevenZipOptions::default())
}

/// Create 7z with options.
pub fn create_7z_with_options<P: AsRef<Path>>(
    inputs: &[P],
    output: P,
    options: SevenZipOptions,
) -> Result<ToolOutput> {
    let output_path = output.as_ref();

    let mut cmd = Command::new("7z");
    cmd.arg("a")
        .arg("-t7z")
        .arg(format!("-mx={}", options.level.level()));

    if options.solid {
        cmd.arg("-ms=on");
    } else {
        cmd.arg("-ms=off");
    }

    if let Some(ref password) = options.password {
        cmd.arg(format!("-p{}", password));
        if options.encrypt_names {
            cmd.arg("-mhe=on");
        }
    }

    cmd.arg(output_path);

    for input in inputs {
        cmd.arg(input.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!("7z failed: {}", String::from_utf8_lossy(&result.stderr)),
            source: None,
        });
    }

    let size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    Ok(ToolOutput::success_with_path(
        format!("Created 7z archive ({} bytes)", size),
        output_path,
    ))
}

/// Extract 7z archive.
///
/// # Arguments
/// * `input` - 7z file path
/// * `output_dir` - Directory to extract to
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::sevenz;
///
/// sevenz::extract_7z("archive.7z", "extracted/").unwrap();
/// ```
pub fn extract_7z<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    extract_7z_with_password(input, output_dir, None)
}

/// Extract 7z with password.
pub fn extract_7z_with_password<P: AsRef<Path>>(
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

    let mut cmd = Command::new("7z");
    cmd.arg("x")
        .arg("-y")
        .arg(format!("-o{}", output_dir.to_string_lossy()));

    if let Some(pwd) = password {
        cmd.arg(format!("-p{}", pwd));
    }

    cmd.arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "7z extraction failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Extracted 7z archive",
        output_dir,
    ))
}

/// Test 7z archive integrity.
pub fn test_7z<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    test_7z_with_password(input, None)
}

/// Test 7z with password.
pub fn test_7z_with_password<P: AsRef<Path>>(
    input: P,
    password: Option<&str>,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    let mut cmd = Command::new("7z");
    cmd.arg("t");

    if let Some(pwd) = password {
        cmd.arg(format!("-p{}", pwd));
    }

    cmd.arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if result.status.success() {
        Ok(ToolOutput::success("Archive integrity OK").with_metadata("valid", "true".to_string()))
    } else {
        Ok(ToolOutput::success("Archive integrity FAILED")
            .with_metadata("valid", "false".to_string()))
    }
}

/// List 7z archive contents.
pub fn list_7z<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    list_7z_with_password(input, None)
}

/// List 7z with password.
pub fn list_7z_with_password<P: AsRef<Path>>(
    input: P,
    password: Option<&str>,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    let mut cmd = Command::new("7z");
    cmd.arg("l");

    if let Some(pwd) = password {
        cmd.arg(format!("-p{}", pwd));
    }

    cmd.arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Failed to list archive".to_string(),
            source: None,
        });
    }

    let output = String::from_utf8_lossy(&result.stdout);

    Ok(ToolOutput::success(output.to_string()).with_metadata("format", "7z".to_string()))
}

/// Update 7z archive.
pub fn update_7z<P: AsRef<Path>>(archive: P, files: &[P]) -> Result<ToolOutput> {
    let archive_path = archive.as_ref();

    if !archive_path.exists() {
        return Err(DxError::FileIo {
            path: archive_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    let mut cmd = Command::new("7z");
    cmd.arg("u").arg(archive_path);

    for file in files {
        cmd.arg(file.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Failed to update archive".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Updated archive with {} files", files.len()),
        archive_path,
    ))
}

/// Delete files from 7z archive.
pub fn delete_from_7z<P: AsRef<Path>>(archive: P, files: &[&str]) -> Result<ToolOutput> {
    let archive_path = archive.as_ref();

    if !archive_path.exists() {
        return Err(DxError::FileIo {
            path: archive_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    let mut cmd = Command::new("7z");
    cmd.arg("d").arg(archive_path);

    for file in files {
        cmd.arg(file);
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Failed to delete from archive".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Deleted {} files from archive", files.len()),
        archive_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_level() {
        assert_eq!(SevenZipLevel::Store.level(), 0);
        assert_eq!(SevenZipLevel::Ultra.level(), 9);
    }
}
