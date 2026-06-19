//! File hashing utilities.
//!
//! Calculate checksums using various hash algorithms.

use crate::error::{DxError, Result};
use crate::tools::{ToolOutput, ToolReceipt};
use std::path::Path;
use std::process::Command;

/// Hash algorithm.
#[derive(Debug, Clone, Copy, Default)]
pub enum HashAlgorithm {
    /// MD5 (fast, not secure).
    Md5,
    /// SHA-1 (legacy).
    Sha1,
    /// SHA-256 (recommended).
    #[default]
    Sha256,
    /// SHA-384.
    Sha384,
    /// SHA-512.
    Sha512,
    /// CRC32 (checksum).
    Crc32,
}

impl HashAlgorithm {
    /// Get command name for this algorithm.
    fn command(&self) -> &'static str {
        match self {
            HashAlgorithm::Md5 => "md5sum",
            HashAlgorithm::Sha1 => "sha1sum",
            HashAlgorithm::Sha256 => "sha256sum",
            HashAlgorithm::Sha384 => "sha384sum",
            HashAlgorithm::Sha512 => "sha512sum",
            HashAlgorithm::Crc32 => "crc32",
        }
    }

    /// Get algorithm name.
    pub fn name(&self) -> &'static str {
        match self {
            HashAlgorithm::Md5 => "MD5",
            HashAlgorithm::Sha1 => "SHA-1",
            HashAlgorithm::Sha256 => "SHA-256",
            HashAlgorithm::Sha384 => "SHA-384",
            HashAlgorithm::Sha512 => "SHA-512",
            HashAlgorithm::Crc32 => "CRC32",
        }
    }

    /// Get hash length in hex characters.
    pub fn hex_length(&self) -> usize {
        match self {
            HashAlgorithm::Md5 => 32,
            HashAlgorithm::Sha1 => 40,
            HashAlgorithm::Sha256 => 64,
            HashAlgorithm::Sha384 => 96,
            HashAlgorithm::Sha512 => 128,
            HashAlgorithm::Crc32 => 8,
        }
    }
}

/// Calculate file hash.
///
/// # Arguments
/// * `input` - File to hash
/// * `algorithm` - Hash algorithm to use
///
/// # Example
/// ```no_run
/// use dx_media::tools::utility::hash::{hash_file, HashAlgorithm};
///
/// let result = hash_file("file.txt", HashAlgorithm::Sha256).unwrap();
/// println!("{}", result.message);
/// ```
pub fn hash_file<P: AsRef<Path>>(input: P, algorithm: HashAlgorithm) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "File not found".to_string(),
            source: None,
        });
    }

    // Try native command
    if let Ok(result) = hash_with_command(input_path, algorithm) {
        return Ok(with_hash_receipt(
            result,
            input_path,
            algorithm,
            "native-command",
        ));
    }

    // Try PowerShell on Windows
    #[cfg(windows)]
    if let Ok(result) = hash_with_powershell(input_path, algorithm) {
        return Ok(with_hash_receipt(
            result,
            input_path,
            algorithm,
            "powershell",
        ));
    }

    // Try openssl
    if let Ok(result) = hash_with_openssl(input_path, algorithm) {
        return Ok(with_hash_receipt(result, input_path, algorithm, "openssl"));
    }

    Err(DxError::Config {
        message: format!("Failed to calculate {} hash", algorithm.name()),
        source: None,
    })
}

fn with_hash_receipt(
    output: ToolOutput,
    input: &Path,
    algorithm: HashAlgorithm,
    backend: &'static str,
) -> ToolOutput {
    output
        .with_receipt(ToolReceipt::local("utility.hash").with_source(input.display().to_string()))
        .with_metadata("tool.hash_algorithm", algorithm.name())
        .with_metadata("tool.hash_backend", backend)
        .with_metadata("tool.expected_media_type", "file")
        .with_metadata("tool.type_validation", "not-applicable")
        .with_metadata("tool.type_validation_scope", "byte-stream")
}

/// Hash using native command.
fn hash_with_command(input: &Path, algorithm: HashAlgorithm) -> Result<ToolOutput> {
    let mut cmd = Command::new(algorithm.command());
    cmd.arg(input);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run {}: {}", algorithm.command(), e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Hash command failed".to_string(),
            source: None,
        });
    }

    let output = String::from_utf8_lossy(&result.stdout);
    let hash = output.split_whitespace().next().unwrap_or("").to_string();

    Ok(ToolOutput::success(hash.clone())
        .with_metadata("algorithm", algorithm.name().to_string())
        .with_metadata("hash", hash))
}

/// Hash using PowerShell.
#[cfg(windows)]
fn hash_with_powershell(input: &Path, algorithm: HashAlgorithm) -> Result<ToolOutput> {
    let algo = match algorithm {
        HashAlgorithm::Md5 => "MD5",
        HashAlgorithm::Sha1 => "SHA1",
        HashAlgorithm::Sha256 => "SHA256",
        HashAlgorithm::Sha384 => "SHA384",
        HashAlgorithm::Sha512 => "SHA512",
        HashAlgorithm::Crc32 => {
            return Err(DxError::Config {
                message: "CRC32 not supported in PowerShell".to_string(),
                source: None,
            });
        }
    };

    let mut cmd = Command::new("powershell");
    cmd.arg("-NoProfile")
        .arg("-Command")
        .arg(
            "& { param([string] $Path, [string] $Algorithm) \
             (Get-FileHash -LiteralPath $Path -Algorithm $Algorithm).Hash }",
        )
        .arg(input)
        .arg(algo);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run PowerShell: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "PowerShell hash failed".to_string(),
            source: None,
        });
    }

    let hash = String::from_utf8_lossy(&result.stdout)
        .trim()
        .to_lowercase();

    Ok(ToolOutput::success(hash.clone())
        .with_metadata("algorithm", algorithm.name().to_string())
        .with_metadata("hash", hash))
}

/// Hash using OpenSSL.
fn hash_with_openssl(input: &Path, algorithm: HashAlgorithm) -> Result<ToolOutput> {
    let algo = match algorithm {
        HashAlgorithm::Md5 => "md5",
        HashAlgorithm::Sha1 => "sha1",
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Sha384 => "sha384",
        HashAlgorithm::Sha512 => "sha512",
        HashAlgorithm::Crc32 => {
            return Err(DxError::Config {
                message: "CRC32 not supported in OpenSSL".to_string(),
                source: None,
            });
        }
    };

    let mut cmd = Command::new("openssl");
    cmd.arg("dgst").arg(format!("-{}", algo)).arg(input);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run openssl: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "OpenSSL hash failed".to_string(),
            source: None,
        });
    }

    let output = String::from_utf8_lossy(&result.stdout);
    // Format: SHA256(file)= hash
    let hash = output.split('=').last().unwrap_or("").trim().to_lowercase();

    Ok(ToolOutput::success(hash.clone())
        .with_metadata("algorithm", algorithm.name().to_string())
        .with_metadata("hash", hash))
}

/// Calculate MD5 hash.
pub fn md5<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    hash_file(input, HashAlgorithm::Md5)
}

/// Calculate SHA-256 hash.
pub fn sha256<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    hash_file(input, HashAlgorithm::Sha256)
}

/// Calculate SHA-512 hash.
pub fn sha512<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    hash_file(input, HashAlgorithm::Sha512)
}

/// Verify file hash.
pub fn verify_hash<P: AsRef<Path>>(
    input: P,
    expected: &str,
    algorithm: HashAlgorithm,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let result = hash_file(input_path, algorithm)?;

    let actual = result.metadata.get("hash").cloned().unwrap_or_default();
    let matches = actual.to_lowercase() == expected.to_lowercase();

    Ok(ToolOutput::success(if matches {
        "Hash matches!".to_string()
    } else {
        format!("Hash mismatch!\nExpected: {}\nActual: {}", expected, actual)
    })
    .with_metadata("valid", matches.to_string())
    .with_metadata("expected", expected.to_string())
    .with_metadata("actual", actual)
    .with_receipt(
        ToolReceipt::local("utility.verify-checksum").with_source(input_path.display().to_string()),
    )
    .with_metadata("tool.hash_algorithm", algorithm.name())
    .with_metadata("tool.expected_media_type", "file")
    .with_metadata("tool.type_validation", "not-applicable")
    .with_metadata("tool.type_validation_scope", "checksum-comparison"))
}

/// Calculate hashes with multiple algorithms.
pub fn multi_hash<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    let mut output = ToolOutput::success("File hashes calculated");

    let algorithms = [
        HashAlgorithm::Md5,
        HashAlgorithm::Sha1,
        HashAlgorithm::Sha256,
        HashAlgorithm::Sha512,
    ];

    for algo in &algorithms {
        if let Ok(result) = hash_file(input_path, *algo) {
            if let Some(hash) = result.metadata.get("hash") {
                output = output.with_metadata(algo.name(), hash.clone());
            }
        }
    }

    Ok(output)
}

/// Batch hash multiple files.
pub fn batch_hash<P: AsRef<Path>>(inputs: &[P], algorithm: HashAlgorithm) -> Result<ToolOutput> {
    let mut results = Vec::new();

    for input in inputs {
        let path = input.as_ref();
        if let Ok(result) = hash_file(path, algorithm) {
            if let Some(hash) = result.metadata.get("hash") {
                results.push(format!("{} {}", hash, path.display()));
            }
        }
    }

    Ok(ToolOutput::success(results.join("\n"))
        .with_metadata("algorithm", algorithm.name().to_string())
        .with_metadata("file_count", results.len().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_name() {
        assert_eq!(HashAlgorithm::Sha256.name(), "SHA-256");
        assert_eq!(HashAlgorithm::Md5.hex_length(), 32);
    }
}
