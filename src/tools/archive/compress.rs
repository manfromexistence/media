//! File compression utilities.
//!
//! Compress files using gzip, bzip2, xz, and other algorithms.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Compression algorithm.
#[derive(Debug, Clone, Copy, Default)]
pub enum CompressionAlgorithm {
    /// Gzip compression.
    #[default]
    Gzip,
    /// Bzip2 compression.
    Bzip2,
    /// XZ compression.
    Xz,
    /// Zstd compression.
    Zstd,
    /// LZ4 compression.
    Lz4,
}

impl CompressionAlgorithm {
    /// Get command name.
    pub fn command(&self) -> &'static str {
        match self {
            CompressionAlgorithm::Gzip => "gzip",
            CompressionAlgorithm::Bzip2 => "bzip2",
            CompressionAlgorithm::Xz => "xz",
            CompressionAlgorithm::Zstd => "zstd",
            CompressionAlgorithm::Lz4 => "lz4",
        }
    }

    /// Get file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            CompressionAlgorithm::Gzip => "gz",
            CompressionAlgorithm::Bzip2 => "bz2",
            CompressionAlgorithm::Xz => "xz",
            CompressionAlgorithm::Zstd => "zst",
            CompressionAlgorithm::Lz4 => "lz4",
        }
    }
}

/// Compression level.
#[derive(Debug, Clone, Copy, Default)]
pub enum CompressionLevel {
    /// Fast compression.
    Fast,
    /// Normal compression.
    #[default]
    Normal,
    /// Best compression.
    Best,
    /// Custom level (1-9).
    Custom(u32),
}

impl CompressionLevel {
    /// Get numeric level.
    pub fn level(&self) -> u32 {
        match self {
            CompressionLevel::Fast => 1,
            CompressionLevel::Normal => 6,
            CompressionLevel::Best => 9,
            CompressionLevel::Custom(l) => *l,
        }
    }
}

/// Compress file with gzip.
///
/// # Arguments
/// * `input` - Input file path
/// * `output` - Output path (optional, defaults to input.gz)
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::compress;
///
/// compress::gzip("file.txt", "file.txt.gz").unwrap();
/// ```
pub fn gzip<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    compress_file(
        input,
        output,
        CompressionAlgorithm::Gzip,
        CompressionLevel::Normal,
    )
}

/// Compress file with bzip2.
pub fn bzip2<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    compress_file(
        input,
        output,
        CompressionAlgorithm::Bzip2,
        CompressionLevel::Normal,
    )
}

/// Compress file with xz.
pub fn xz<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    compress_file(
        input,
        output,
        CompressionAlgorithm::Xz,
        CompressionLevel::Normal,
    )
}

/// Compress file with zstd.
pub fn zstd<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    compress_file(
        input,
        output,
        CompressionAlgorithm::Zstd,
        CompressionLevel::Normal,
    )
}

/// Compress file with specified algorithm.
pub fn compress_file<P: AsRef<Path>>(
    input: P,
    output: P,
    algorithm: CompressionAlgorithm,
    level: CompressionLevel,
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

    let original_size = std::fs::metadata(input_path).map_or(0, |m| m.len());

    let mut cmd = Command::new(algorithm.command());

    // Add compression level
    cmd.arg(format!("-{}", level.level()));

    // Keep original, output to stdout
    cmd.arg("-k").arg("-c").arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run {}: {}", algorithm.command(), e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "{} failed: {}",
                algorithm.command(),
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    // Write output
    std::fs::write(output_path, &result.stdout).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to write output: {}", e),
        source: None,
    })?;

    let compressed_size = result.stdout.len() as u64;
    let ratio = if original_size > 0 {
        (compressed_size as f64 / original_size as f64) * 100.0
    } else {
        100.0
    };

    Ok(ToolOutput::success_with_path(
        format!(
            "Compressed {} -> {} ({:.1}% of original)",
            original_size, compressed_size, ratio
        ),
        output_path,
    )
    .with_metadata("original_size", original_size.to_string())
    .with_metadata("compressed_size", compressed_size.to_string())
    .with_metadata("ratio", format!("{:.1}%", ratio)))
}

/// Compress in place (replaces original).
pub fn compress_in_place<P: AsRef<Path>>(
    input: P,
    algorithm: CompressionAlgorithm,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    let mut cmd = Command::new(algorithm.command());
    cmd.arg("-f") // Force
        .arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run {}: {}", algorithm.command(), e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!("{} failed", algorithm.command()),
            source: None,
        });
    }

    let output_path = input_path.with_extension(format!(
        "{}.{}",
        input_path.extension().unwrap_or_default().to_string_lossy(),
        algorithm.extension()
    ));

    Ok(ToolOutput::success_with_path(
        format!("Compressed with {}", algorithm.command()),
        &output_path,
    ))
}

/// Batch compress multiple files.
pub fn batch_compress<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    algorithm: CompressionAlgorithm,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create directory: {}", e),
        source: None,
    })?;

    let mut compressed = Vec::new();
    let mut total_original = 0u64;
    let mut total_compressed = 0u64;

    for input in inputs {
        let input_path = input.as_ref();
        let file_name = format!(
            "{}.{}",
            input_path.file_name().unwrap_or_default().to_string_lossy(),
            algorithm.extension()
        );
        let output_path = output_dir.join(file_name);

        if let Ok(result) = compress_file(
            input_path,
            &output_path,
            algorithm,
            CompressionLevel::Normal,
        ) {
            if let Some(orig) = result.metadata.get("original_size") {
                total_original += orig.parse::<u64>().unwrap_or(0);
            }
            if let Some(comp) = result.metadata.get("compressed_size") {
                total_compressed += comp.parse::<u64>().unwrap_or(0);
            }
            compressed.push(output_path);
        }
    }

    let ratio = if total_original > 0 {
        (total_compressed as f64 / total_original as f64) * 100.0
    } else {
        100.0
    };

    Ok(ToolOutput::success(format!(
        "Compressed {} files ({:.1}% of original)",
        compressed.len(),
        ratio
    ))
    .with_paths(compressed)
    .with_metadata("total_original", total_original.to_string())
    .with_metadata("total_compressed", total_compressed.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_extension() {
        assert_eq!(CompressionAlgorithm::Gzip.extension(), "gz");
        assert_eq!(CompressionAlgorithm::Xz.extension(), "xz");
    }

    #[test]
    fn test_compression_level() {
        assert_eq!(CompressionLevel::Fast.level(), 1);
        assert_eq!(CompressionLevel::Best.level(), 9);
    }
}
