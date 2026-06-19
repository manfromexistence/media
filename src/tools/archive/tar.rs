//! TAR archive operations.
//!
//! Create and extract TAR archives.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// TAR compression type.
#[derive(Debug, Clone, Copy, Default)]
pub enum TarCompression {
    /// No compression.
    #[default]
    None,
    /// Gzip compression (.tar.gz).
    Gzip,
    /// Bzip2 compression (.tar.bz2).
    Bzip2,
    /// XZ compression (.tar.xz).
    Xz,
    /// Zstd compression (.tar.zst).
    Zstd,
}

impl TarCompression {
    /// Get tar flag for compression.
    pub fn tar_flag(&self) -> Option<&'static str> {
        match self {
            TarCompression::None => None,
            TarCompression::Gzip => Some("-z"),
            TarCompression::Bzip2 => Some("-j"),
            TarCompression::Xz => Some("-J"),
            TarCompression::Zstd => Some("--zstd"),
        }
    }

    /// Get file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            TarCompression::None => "tar",
            TarCompression::Gzip => "tar.gz",
            TarCompression::Bzip2 => "tar.bz2",
            TarCompression::Xz => "tar.xz",
            TarCompression::Zstd => "tar.zst",
        }
    }

    /// Detect from file extension.
    pub fn from_extension(path: &Path) -> Self {
        let name = path.to_string_lossy().to_lowercase();
        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            TarCompression::Gzip
        } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
            TarCompression::Bzip2
        } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
            TarCompression::Xz
        } else if name.ends_with(".tar.zst") {
            TarCompression::Zstd
        } else {
            TarCompression::None
        }
    }
}

/// Create TAR archive.
///
/// # Arguments
/// * `inputs` - Files and directories to add
/// * `output` - Output TAR path
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::tar;
///
/// tar::create_tar(&["dir1/", "file.txt"], "archive.tar").unwrap();
/// ```
pub fn create_tar<P: AsRef<Path>>(inputs: &[P], output: P) -> Result<ToolOutput> {
    create_tar_with_compression(inputs, output, TarCompression::None)
}

/// Create compressed TAR archive.
pub fn create_tar_gz<P: AsRef<Path>>(inputs: &[P], output: P) -> Result<ToolOutput> {
    create_tar_with_compression(inputs, output, TarCompression::Gzip)
}

/// Create TAR with specific compression.
pub fn create_tar_with_compression<P: AsRef<Path>>(
    inputs: &[P],
    output: P,
    compression: TarCompression,
) -> Result<ToolOutput> {
    let output_path = output.as_ref();

    let mut cmd = Command::new("tar");
    cmd.arg("-c");

    if let Some(flag) = compression.tar_flag() {
        cmd.arg(flag);
    }

    cmd.arg("-f").arg(output_path);

    for input in inputs {
        cmd.arg(input.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run tar: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!("tar failed: {}", String::from_utf8_lossy(&result.stderr)),
            source: None,
        });
    }

    let size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    Ok(ToolOutput::success_with_path(
        format!("Created TAR archive ({} bytes)", size),
        output_path,
    ))
}

/// Extract TAR archive.
///
/// # Arguments
/// * `input` - TAR file path
/// * `output_dir` - Directory to extract to
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::tar;
///
/// tar::extract_tar("archive.tar.gz", "extracted/").unwrap();
/// ```
pub fn extract_tar<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
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

    // Auto-detect compression
    let compression = TarCompression::from_extension(input_path);

    let mut cmd = Command::new("tar");
    cmd.arg("-x");

    if let Some(flag) = compression.tar_flag() {
        cmd.arg(flag);
    }

    cmd.arg("-f").arg(input_path).arg("-C").arg(output_dir);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run tar: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "tar extraction failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Extracted TAR archive",
        output_dir,
    ))
}

/// List TAR archive contents.
pub fn list_tar<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    let compression = TarCompression::from_extension(input_path);

    let mut cmd = Command::new("tar");
    cmd.arg("-t");

    if let Some(flag) = compression.tar_flag() {
        cmd.arg(flag);
    }

    cmd.arg("-f").arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run tar: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Failed to list archive".to_string(),
            source: None,
        });
    }

    let listing = String::from_utf8_lossy(&result.stdout);
    let file_count = listing.lines().count();

    Ok(
        ToolOutput::success(listing.to_string())
            .with_metadata("file_count", file_count.to_string()),
    )
}

/// Append to TAR archive (uncompressed only).
pub fn append_to_tar<P: AsRef<Path>>(archive: P, files: &[P]) -> Result<ToolOutput> {
    let archive_path = archive.as_ref();

    if !archive_path.exists() {
        return Err(DxError::FileIo {
            path: archive_path.to_path_buf(),
            message: "Archive not found".to_string(),
            source: None,
        });
    }

    let mut cmd = Command::new("tar");
    cmd.arg("-r").arg("-f").arg(archive_path);

    for file in files {
        cmd.arg(file.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run tar: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Failed to append to archive".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Added {} files to archive", files.len()),
        archive_path,
    ))
}

/// Extract specific files from TAR.
pub fn tar_extract_files<P: AsRef<Path>>(
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

    let compression = TarCompression::from_extension(archive_path);

    let mut cmd = Command::new("tar");
    cmd.arg("-x");

    if let Some(flag) = compression.tar_flag() {
        cmd.arg(flag);
    }

    cmd.arg("-f").arg(archive_path).arg("-C").arg(output_dir);

    for file in files {
        cmd.arg(file);
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run tar: {}", e),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_compression_detection() {
        assert!(matches!(
            TarCompression::from_extension(&PathBuf::from("test.tar.gz")),
            TarCompression::Gzip
        ));
        assert!(matches!(
            TarCompression::from_extension(&PathBuf::from("test.tar.xz")),
            TarCompression::Xz
        ));
    }
}
