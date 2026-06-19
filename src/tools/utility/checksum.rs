//! Checksum calculation and verification.
//!
//! Supports multiple hashing algorithms for file integrity verification.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use rayon::prelude::*;

use crate::tools::{ToolOutput, ToolReceipt};

/// Supported checksum algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChecksumAlgorithm {
    /// MD5 (legacy, not recommended for security).
    Md5,
    /// SHA-1 (legacy, not recommended for security).
    Sha1,
    /// SHA-256 (recommended).
    Sha256,
    /// SHA-512.
    Sha512,
    /// Blake3 (modern, very fast).
    Blake3,
    /// CRC32 (fast, for integrity only).
    Crc32,
    /// XXHash64 (very fast, non-cryptographic).
    XxHash64,
}

impl ChecksumAlgorithm {
    /// Get the algorithm name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA-1",
            Self::Sha256 => "SHA-256",
            Self::Sha512 => "SHA-512",
            Self::Blake3 => "BLAKE3",
            Self::Crc32 => "CRC32",
            Self::XxHash64 => "XXH64",
        }
    }

    /// Get the expected hash length in characters (hex).
    pub fn hex_length(&self) -> usize {
        match self {
            Self::Md5 => 32,
            Self::Sha1 => 40,
            Self::Sha256 => 64,
            Self::Sha512 => 128,
            Self::Blake3 => 64,
            Self::Crc32 => 8,
            Self::XxHash64 => 16,
        }
    }

    /// Parse algorithm from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "md5" => Some(Self::Md5),
            "sha1" | "sha-1" => Some(Self::Sha1),
            "sha256" | "sha-256" => Some(Self::Sha256),
            "sha512" | "sha-512" => Some(Self::Sha512),
            "blake3" => Some(Self::Blake3),
            "crc32" => Some(Self::Crc32),
            "xxhash64" | "xxh64" => Some(Self::XxHash64),
            _ => None,
        }
    }
}

/// Checksum result for a file.
#[derive(Debug, Clone)]
pub struct ChecksumResult {
    /// Path to the file.
    pub path: PathBuf,
    /// Algorithm used.
    pub algorithm: ChecksumAlgorithm,
    /// Calculated hash (hex string).
    pub hash: String,
    /// File size in bytes.
    pub size: u64,
}

/// Calculate checksum for a single file.
pub fn calculate_checksum(
    path: impl AsRef<Path>,
    algorithm: ChecksumAlgorithm,
) -> std::io::Result<ChecksumResult> {
    let path = path.as_ref();
    let size = path.metadata()?.len();
    let hash = calculate_checksum_hex(path, algorithm)?;

    Ok(ChecksumResult {
        path: path.to_path_buf(),
        algorithm,
        hash,
        size,
    })
}

/// Calculate checksums for multiple files in parallel.
pub fn calculate_checksums_parallel<P: AsRef<Path> + Sync>(
    paths: &[P],
    algorithm: ChecksumAlgorithm,
) -> Vec<Result<ChecksumResult, (PathBuf, std::io::Error)>> {
    paths
        .par_iter()
        .map(|path| {
            let path = path.as_ref();
            calculate_checksum(path, algorithm).map_err(|e| (path.to_path_buf(), e))
        })
        .collect()
}

/// Verify a file against an expected checksum.
pub fn verify_checksum(
    path: impl AsRef<Path>,
    expected: &str,
    algorithm: ChecksumAlgorithm,
) -> std::io::Result<bool> {
    let result = calculate_checksum(path, algorithm)?;
    Ok(result.hash.eq_ignore_ascii_case(expected))
}

/// Generate a checksum file (like sha256sum format).
pub fn generate_checksum_file<P: AsRef<Path> + Sync>(
    files: &[P],
    output: impl AsRef<Path>,
    algorithm: ChecksumAlgorithm,
) -> std::io::Result<()> {
    let results = calculate_checksums_parallel(files, algorithm);
    let mut file = File::create(output)?;

    for result in results {
        match result {
            Ok(checksum) => {
                writeln!(file, "{}  {}", checksum.hash, checksum.path.display())?;
            }
            Err((path, e)) => {
                writeln!(file, "# ERROR: {} - {}", path.display(), e)?;
            }
        }
    }

    Ok(())
}

/// Parse and verify a checksum file.
pub fn verify_checksum_file(
    checksum_file: impl AsRef<Path>,
    algorithm: ChecksumAlgorithm,
) -> std::io::Result<VerificationReport> {
    let content = std::fs::read_to_string(checksum_file)?;
    let mut report = VerificationReport::default();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse "hash  filename" or "hash *filename"
        let parts: Vec<&str> = if line.contains("  ") {
            line.splitn(2, "  ").collect()
        } else if line.contains(" *") {
            line.splitn(2, " *").collect()
        } else {
            continue;
        };

        if parts.len() != 2 {
            report.errors.push(format!("Invalid line: {}", line));
            continue;
        }

        let expected_hash = parts[0].trim();
        let file_path = parts[1].trim();
        let path = PathBuf::from(file_path);

        match verify_checksum(&path, expected_hash, algorithm) {
            Ok(true) => {
                report.passed.push(path);
            }
            Ok(false) => {
                report.failed.push(path);
            }
            Err(e) => {
                report.errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    Ok(report)
}

/// Verification report.
#[derive(Debug, Default)]
pub struct VerificationReport {
    /// Files that passed verification.
    pub passed: Vec<PathBuf>,
    /// Files that failed verification.
    pub failed: Vec<PathBuf>,
    /// Errors during verification.
    pub errors: Vec<String>,
}

impl VerificationReport {
    /// Check if all files passed.
    pub fn all_passed(&self) -> bool {
        self.failed.is_empty() && self.errors.is_empty()
    }

    /// Get total number of files checked.
    pub fn total(&self) -> usize {
        self.passed.len() + self.failed.len()
    }
}

/// Calculate checksum and return as ToolOutput.
pub fn checksum_tool(path: impl AsRef<Path>, algorithm: ChecksumAlgorithm) -> ToolOutput {
    match calculate_checksum(&path, algorithm) {
        Ok(result) => {
            let mut metadata = HashMap::new();
            metadata.insert("algorithm".to_string(), result.algorithm.name().to_string());
            metadata.insert("hash".to_string(), result.hash.clone());
            metadata.insert("size".to_string(), result.size.to_string());

            ToolOutput::success(format!(
                "{}: {} ({})",
                algorithm.name(),
                result.hash,
                format_size(result.size)
            ))
            .with_paths(vec![result.path])
            .with_receipt(ToolReceipt::local("utility.hash"))
            .with_metadata_entries(metadata)
        }
        Err(e) => ToolOutput::failure(format!("Checksum calculation failed: {}", e))
            .with_receipt(ToolReceipt::local("utility.hash")),
    }
}

/// Verify checksum and return as ToolOutput.
pub fn verify_tool(
    path: impl AsRef<Path>,
    expected: &str,
    algorithm: ChecksumAlgorithm,
) -> ToolOutput {
    match verify_checksum(&path, expected, algorithm) {
        Ok(true) => ToolOutput::success(format!("{}: PASSED", algorithm.name()))
            .with_paths(vec![path.as_ref().to_path_buf()])
            .with_receipt(ToolReceipt::local("utility.verify-checksum")),
        Ok(false) => {
            ToolOutput::failure(format!("{}: FAILED - checksum mismatch", algorithm.name()))
                .with_paths(vec![path.as_ref().to_path_buf()])
                .with_receipt(ToolReceipt::local("utility.verify-checksum"))
        }
        Err(e) => ToolOutput::failure(format!("Verification failed: {}", e))
            .with_receipt(ToolReceipt::local("utility.verify-checksum")),
    }
}

/// Format size for display.
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

fn calculate_checksum_hex(path: &Path, algorithm: ChecksumAlgorithm) -> io::Result<String> {
    #[cfg(windows)]
    if let Some(powershell_algorithm) = powershell_algorithm(algorithm) {
        if let Ok(hash) = hash_with_powershell(path, powershell_algorithm) {
            return Ok(hash);
        }
    }

    if let Some((command, args)) = checksum_command(algorithm) {
        if let Ok(hash) = hash_with_command(path, command, args) {
            return Ok(hash);
        }
    }

    if let Some(openssl_algorithm) = openssl_algorithm(algorithm) {
        if let Ok(hash) = hash_with_openssl(path, openssl_algorithm) {
            return Ok(hash);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "{} checksum requires an installed checksum tool",
            algorithm.name()
        ),
    ))
}

#[cfg(windows)]
fn powershell_algorithm(algorithm: ChecksumAlgorithm) -> Option<&'static str> {
    match algorithm {
        ChecksumAlgorithm::Md5 => Some("MD5"),
        ChecksumAlgorithm::Sha1 => Some("SHA1"),
        ChecksumAlgorithm::Sha256 => Some("SHA256"),
        ChecksumAlgorithm::Sha512 => Some("SHA512"),
        ChecksumAlgorithm::Blake3 | ChecksumAlgorithm::Crc32 | ChecksumAlgorithm::XxHash64 => None,
    }
}

#[cfg(windows)]
fn hash_with_powershell(path: &Path, algorithm: &str) -> io::Result<String> {
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(
            "& { param([string] $Path, [string] $Algorithm) \
             (Get-FileHash -LiteralPath $Path -Algorithm $Algorithm).Hash.ToLowerInvariant() }",
        )
        .arg(path)
        .arg(algorithm)
        .output()?;

    parse_hash_output(output, "PowerShell")
}

fn checksum_command(
    algorithm: ChecksumAlgorithm,
) -> Option<(&'static str, &'static [&'static str])> {
    match algorithm {
        ChecksumAlgorithm::Md5 => Some(("md5sum", &[])),
        ChecksumAlgorithm::Sha1 => Some(("sha1sum", &[])),
        ChecksumAlgorithm::Sha256 => Some(("sha256sum", &[])),
        ChecksumAlgorithm::Sha512 => Some(("sha512sum", &[])),
        ChecksumAlgorithm::Blake3 => Some(("b3sum", &[])),
        ChecksumAlgorithm::Crc32 => Some(("crc32", &[])),
        ChecksumAlgorithm::XxHash64 => Some(("xxhsum", &["-H64"])),
    }
}

fn hash_with_command(path: &Path, command: &str, args: &[&str]) -> io::Result<String> {
    let output = Command::new(command).args(args).arg(path).output()?;
    parse_hash_output(output, command)
}

fn openssl_algorithm(algorithm: ChecksumAlgorithm) -> Option<&'static str> {
    match algorithm {
        ChecksumAlgorithm::Md5 => Some("md5"),
        ChecksumAlgorithm::Sha1 => Some("sha1"),
        ChecksumAlgorithm::Sha256 => Some("sha256"),
        ChecksumAlgorithm::Sha512 => Some("sha512"),
        ChecksumAlgorithm::Blake3 | ChecksumAlgorithm::Crc32 | ChecksumAlgorithm::XxHash64 => None,
    }
}

fn hash_with_openssl(path: &Path, algorithm: &str) -> io::Result<String> {
    let output = Command::new("openssl")
        .arg("dgst")
        .arg(format!("-{}", algorithm))
        .arg(path)
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("openssl {} failed", algorithm),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split('=')
        .last()
        .map(str::trim)
        .filter(|hash| !hash.is_empty())
        .map(|hash| hash.to_ascii_lowercase())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing openssl hash output"))
}

fn parse_hash_output(output: std::process::Output, command: &str) -> io::Result<String> {
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("{} checksum command failed", command),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .next()
        .filter(|hash| !hash.is_empty())
        .map(|hash| hash.to_ascii_lowercase())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing checksum output"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_checksum_algorithm() {
        assert_eq!(ChecksumAlgorithm::Sha256.name(), "SHA-256");
        assert_eq!(ChecksumAlgorithm::Sha256.hex_length(), 64);
        assert_eq!(
            ChecksumAlgorithm::from_str("sha256"),
            Some(ChecksumAlgorithm::Sha256)
        );
    }

    #[test]
    fn test_calculate_checksum() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, b"test content").unwrap();

        let result = calculate_checksum(&file, ChecksumAlgorithm::Sha256).unwrap();

        assert_eq!(result.algorithm, ChecksumAlgorithm::Sha256);
        assert_eq!(
            result.hash,
            "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72"
        );
        assert_eq!(result.size, 12);
    }

    #[test]
    fn test_verify_checksum() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, b"test content").unwrap();

        let result = calculate_checksum(&file, ChecksumAlgorithm::Sha256).unwrap();

        assert!(verify_checksum(&file, &result.hash, ChecksumAlgorithm::Sha256).unwrap());
        assert!(!verify_checksum(&file, "wrong_hash", ChecksumAlgorithm::Sha256).unwrap());
    }
}
