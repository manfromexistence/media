//! Archive encryption utilities.
//!
//! Create encrypted archives.

use crate::error::{DxError, Result};
use crate::tools::{ToolOutput, ToolReceipt};
use std::path::Path;
use std::process::Command;

/// Encryption method.
#[derive(Debug, Clone, Copy, Default)]
pub enum EncryptionMethod {
    /// AES-256 encryption.
    #[default]
    Aes256,
    /// ZipCrypto (weak, max compatibility).
    ZipCrypto,
}

/// Create encrypted ZIP archive.
///
/// # Arguments
/// * `inputs` - Files to include
/// * `output` - Output archive path
/// * `password` - Encryption password
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::encrypt;
///
/// encrypt::create_encrypted_zip(&["secret.txt"], "encrypted.zip", "mypassword").unwrap();
/// ```
pub fn create_encrypted_zip<P: AsRef<Path>>(
    inputs: &[P],
    output: P,
    password: &str,
) -> Result<ToolOutput> {
    create_encrypted_zip_with_method(inputs, output, password, EncryptionMethod::Aes256)
}

/// Create encrypted ZIP with specific method.
pub fn create_encrypted_zip_with_method<P: AsRef<Path>>(
    inputs: &[P],
    output: P,
    password: &str,
    method: EncryptionMethod,
) -> Result<ToolOutput> {
    let output_path = output.as_ref();
    validate_zip_output(output_path)?;

    // Try 7z first (better encryption)
    if let Ok(result) = create_with_7z(inputs, output_path, password, method) {
        return Ok(result);
    }

    // Try zip
    if let Ok(result) = create_with_zip(inputs, output_path, password) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "Encrypted archive creation failed. Install 7z or zip.".to_string(),
        source: None,
    })
}

/// Create using 7z with AES encryption.
fn create_with_7z<P: AsRef<Path>>(
    inputs: &[P],
    output: &Path,
    password: &str,
    method: EncryptionMethod,
) -> Result<ToolOutput> {
    let mut cmd = Command::new("7z");
    cmd.arg("a").arg("-tzip").arg(format!("-p{}", password));

    // Set encryption method
    match method {
        EncryptionMethod::Aes256 => {
            cmd.arg("-mem=AES256");
        }
        EncryptionMethod::ZipCrypto => {
            cmd.arg("-mem=ZipCrypto");
        }
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
            message: format!("7z failed: {}", String::from_utf8_lossy(&result.stderr)),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Created encrypted archive with {:?}", method),
        output,
    )
    .with_receipt(
        ToolReceipt::local("archive.encrypt")
            .with_dependency("7z")
            .with_source(archive_input_sources(inputs)),
    )
    .with_archive_type_validation(output, "zip")
    .with_metadata("encryption", format!("{:?}", method)))
}

/// Create using zip command.
fn create_with_zip<P: AsRef<Path>>(
    inputs: &[P],
    output: &Path,
    password: &str,
) -> Result<ToolOutput> {
    let mut cmd = Command::new("zip");
    cmd.arg("-r")
        .arg("-e") // Encrypt
        .arg("-P")
        .arg(password)
        .arg(output);

    for input in inputs {
        cmd.arg(input.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run zip: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "zip encryption failed".to_string(),
            source: None,
        });
    }

    Ok(
        ToolOutput::success_with_path("Created encrypted ZIP archive", output)
            .with_receipt(
                ToolReceipt::local("archive.encrypt")
                    .with_dependency("zip")
                    .with_source(archive_input_sources(inputs)),
            )
            .with_archive_type_validation(output, "zip"),
    )
}

fn archive_input_sources<T: AsRef<Path>>(inputs: &[T]) -> String {
    inputs
        .iter()
        .map(|input| input.as_ref().display().to_string())
        .collect::<Vec<_>>()
        .join(";")
}

fn archive_output_extension(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| "unknown".to_string())
}

fn validate_zip_output(path: &Path) -> Result<()> {
    let actual_extension = archive_output_extension(path);
    if actual_extension == "zip" {
        return Ok(());
    }

    Err(DxError::Config {
        message: format!(
            "archive.encrypt failed type validation: archive-extension-mismatch (expected .zip, got .{actual_extension})"
        ),
        source: None,
    })
}

trait ArchiveTypeValidation {
    fn with_archive_type_validation(self, path: &Path, expected_extension: &'static str) -> Self;
}

impl ArchiveTypeValidation for ToolOutput {
    fn with_archive_type_validation(self, path: &Path, expected_extension: &'static str) -> Self {
        let actual_extension = archive_output_extension(path);
        let valid = actual_extension == expected_extension;

        let mut output = self
            .with_metadata("tool.expected_media_type", "archive")
            .with_metadata("tool.expected_output_extension", expected_extension)
            .with_metadata("tool.output_extension", actual_extension)
            .with_metadata("tool.type_validation", if valid { "pass" } else { "fail" });

        if !valid {
            output =
                output.with_metadata("tool.type_validation_reason", "archive-extension-mismatch");
        }

        output
    }
}

/// Create encrypted 7z archive.
pub fn create_encrypted_7z<P: AsRef<Path>>(
    inputs: &[P],
    output: P,
    password: &str,
) -> Result<ToolOutput> {
    let output_path = output.as_ref();

    let mut cmd = Command::new("7z");
    cmd.arg("a")
        .arg("-t7z")
        .arg(format!("-p{}", password))
        .arg("-mhe=on") // Encrypt headers
        .arg(output_path);

    for input in inputs {
        cmd.arg(input.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run 7z: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "7z encryption failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Created encrypted 7z archive",
        output_path,
    ))
}

/// Extract encrypted archive.
pub fn extract_encrypted<P: AsRef<Path>>(
    input: P,
    output_dir: P,
    password: &str,
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

    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Try 7z first
    if let Ok(result) = extract_with_7z(input_path, output_dir, password) {
        return Ok(result);
    }

    // Try unzip for .zip files
    if ext == "zip" {
        if let Ok(result) = extract_with_unzip(input_path, output_dir, password) {
            return Ok(result);
        }
    }

    Err(DxError::Config {
        message: "Extraction failed. Wrong password?".to_string(),
        source: None,
    })
}

/// Extract using 7z.
fn extract_with_7z(input: &Path, output_dir: &Path, password: &str) -> Result<ToolOutput> {
    let mut cmd = Command::new("7z");
    cmd.arg("x")
        .arg("-y")
        .arg(format!("-p{}", password))
        .arg(format!("-o{}", output_dir.to_string_lossy()))
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
        "Extracted encrypted archive",
        output_dir,
    ))
}

/// Extract using unzip.
fn extract_with_unzip(input: &Path, output_dir: &Path, password: &str) -> Result<ToolOutput> {
    let mut cmd = Command::new("unzip");
    cmd.arg("-o")
        .arg("-P")
        .arg(password)
        .arg(input)
        .arg("-d")
        .arg(output_dir);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run unzip: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "unzip extraction failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Extracted encrypted ZIP",
        output_dir,
    ))
}

/// Check if archive is encrypted.
pub fn is_encrypted<P: AsRef<Path>>(input: P) -> Result<bool> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "File not found".to_string(),
            source: None,
        });
    }

    // Try to list with 7z
    let mut cmd = Command::new("7z");
    cmd.arg("l").arg("-slt").arg(input_path);

    if let Ok(result) = cmd.output() {
        let output = String::from_utf8_lossy(&result.stdout);
        if output.contains("Encrypted = +") {
            return Ok(true);
        }
    }

    // Try unzip test
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if ext == "zip" {
        let mut cmd = Command::new("unzip");
        cmd.arg("-t").arg(input_path);

        if let Ok(result) = cmd.output() {
            let output = String::from_utf8_lossy(&result.stderr);
            if output.contains("incorrect password") || output.contains("need password") {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Change archive password.
pub fn change_password<P: AsRef<Path>>(
    input: P,
    output: P,
    old_password: &str,
    new_password: &str,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Extract to temp
    let temp_dir = std::env::temp_dir().join(format!("archive_pw_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).map_err(|e| DxError::FileIo {
        path: temp_dir.clone(),
        message: format!("Failed to create temp dir: {}", e),
        source: None,
    })?;

    // Extract with old password
    extract_encrypted(input_path, &temp_dir, old_password)?;

    // Get all files
    let files: Vec<_> = walkdir::WalkDir::new(&temp_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    // Create new archive with new password
    let file_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
    create_encrypted_zip(&file_refs, output_path, new_password)?;

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(ToolOutput::success_with_path(
        "Changed archive password",
        output_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_method() {
        let method = EncryptionMethod::Aes256;
        assert!(matches!(method, EncryptionMethod::Aes256));
    }
}
