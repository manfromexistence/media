//! RAR archive operations.
//!
//! Extract and list RAR archives.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Extract RAR archive.
///
/// # Arguments
/// * `input` - RAR file path
/// * `output_dir` - Directory to extract to
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::rar;
///
/// rar::extract_rar("archive.rar", "extracted/").unwrap();
/// ```
pub fn extract_rar<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    extract_rar_with_password(input, output_dir, None)
}

/// Extract RAR with password.
pub fn extract_rar_with_password<P: AsRef<Path>>(
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

    // Try unrar first
    if let Ok(result) = extract_with_unrar(input_path, output_dir, password) {
        return Ok(result);
    }

    // Try 7z as fallback
    if let Ok(result) = extract_with_7z(input_path, output_dir, password) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "RAR extraction failed. Install unrar or 7z.".to_string(),
        source: None,
    })
}

/// Extract using unrar.
fn extract_with_unrar(
    input: &Path,
    output_dir: &Path,
    password: Option<&str>,
) -> Result<ToolOutput> {
    let mut cmd = Command::new("unrar");
    cmd.arg("x").arg("-y");

    if let Some(pwd) = password {
        cmd.arg(format!("-p{}", pwd));
    } else {
        cmd.arg("-p-"); // No password
    }

    cmd.arg(input)
        .arg(format!("{}/", output_dir.to_string_lossy()));

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run unrar: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!("unrar failed: {}", String::from_utf8_lossy(&result.stderr)),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Extracted RAR archive",
        output_dir,
    ))
}

/// Extract using 7z.
fn extract_with_7z(input: &Path, output_dir: &Path, password: Option<&str>) -> Result<ToolOutput> {
    let mut cmd = Command::new("7z");
    cmd.arg("x")
        .arg("-y")
        .arg(format!("-o{}", output_dir.to_string_lossy()));

    if let Some(pwd) = password {
        cmd.arg(format!("-p{}", pwd));
    }

    cmd.arg(input);

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
        "Extracted RAR archive with 7z",
        output_dir,
    ))
}

/// List RAR archive contents.
pub fn list_rar<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    list_rar_with_password(input, None)
}

/// List RAR with password.
pub fn list_rar_with_password<P: AsRef<Path>>(
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

    // Try unrar
    let mut cmd = Command::new("unrar");
    cmd.arg("l");

    if let Some(pwd) = password {
        cmd.arg(format!("-p{}", pwd));
    }

    cmd.arg(input_path);

    if let Ok(result) = cmd.output() {
        if result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout);
            return Ok(
                ToolOutput::success(output.to_string()).with_metadata("format", "rar".to_string())
            );
        }
    }

    // Try 7z
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
            message: "Failed to list RAR archive".to_string(),
            source: None,
        });
    }

    let output = String::from_utf8_lossy(&result.stdout);
    Ok(ToolOutput::success(output.to_string()).with_metadata("format", "rar".to_string()))
}

/// Test RAR archive integrity.
pub fn test_rar<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    test_rar_with_password(input, None)
}

/// Test RAR with password.
pub fn test_rar_with_password<P: AsRef<Path>>(
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

    // Try unrar
    let mut cmd = Command::new("unrar");
    cmd.arg("t");

    if let Some(pwd) = password {
        cmd.arg(format!("-p{}", pwd));
    }

    cmd.arg(input_path);

    if let Ok(result) = cmd.output() {
        if result.status.success() {
            return Ok(ToolOutput::success("Archive integrity OK")
                .with_metadata("valid", "true".to_string()));
        }
        return Ok(ToolOutput::success("Archive integrity FAILED")
            .with_metadata("valid", "false".to_string()));
    }

    // Try 7z
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

/// Extract specific files from RAR.
pub fn rar_extract_files<P: AsRef<Path>>(
    archive: P,
    files: &[&str],
    output_dir: P,
) -> Result<ToolOutput> {
    let archive_path = archive.as_ref();
    let output_dir = output_dir.as_ref();

    if !archive_path.exists() {
        return Err(DxError::FileIo {
            path: archive_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create directory: {}", e),
        source: None,
    })?;

    // Try unrar
    let mut cmd = Command::new("unrar");
    cmd.arg("x").arg("-y").arg(archive_path);

    for file in files {
        cmd.arg(file);
    }

    cmd.arg(format!("{}/", output_dir.to_string_lossy()));

    if let Ok(result) = cmd.output() {
        if result.status.success() {
            return Ok(ToolOutput::success_with_path(
                format!("Extracted {} files", files.len()),
                output_dir,
            ));
        }
    }

    // Try 7z
    let mut cmd = Command::new("7z");
    cmd.arg("x")
        .arg("-y")
        .arg(format!("-o{}", output_dir.to_string_lossy()))
        .arg(archive_path);

    for file in files {
        cmd.arg(file);
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Failed to extract files".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Extracted {} files", files.len()),
        output_dir,
    ))
}

/// Check if RAR archive is encrypted.
pub fn rar_is_encrypted<P: AsRef<Path>>(input: P) -> Result<bool> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    // Try to list without password
    let mut cmd = Command::new("unrar");
    cmd.arg("l").arg("-p-").arg(input_path);

    if let Ok(result) = cmd.output() {
        let output = String::from_utf8_lossy(&result.stdout);
        let stderr = String::from_utf8_lossy(&result.stderr);

        // Check for encryption indicators
        if output.contains('*') || stderr.contains("password") || stderr.contains("encrypted") {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rar_methods() {
        // Just verify the module compiles correctly
        // This test exists to ensure the module structure is valid
    }
}
