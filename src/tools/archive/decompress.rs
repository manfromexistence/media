//! File decompression utilities.
//!
//! Decompress files compressed with gzip, bzip2, xz, and other algorithms.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Decompress gzip file.
///
/// # Arguments
/// * `input` - Compressed file path
/// * `output` - Output path
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::decompress;
///
/// decompress::gunzip("file.txt.gz", "file.txt").unwrap();
/// ```
pub fn gunzip<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    decompress_file(input, output, "gzip")
}

/// Decompress bzip2 file.
pub fn bunzip2<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    decompress_file(input, output, "bzip2")
}

/// Decompress xz file.
pub fn unxz<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    decompress_file(input, output, "xz")
}

/// Decompress zstd file.
pub fn unzstd<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    decompress_file(input, output, "zstd")
}

/// Decompress lz4 file.
pub fn unlz4<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    decompress_file(input, output, "lz4")
}

/// Decompress file with auto-detected algorithm.
///
/// # Arguments
/// * `input` - Compressed file path
/// * `output` - Output path
///
/// # Example
/// ```no_run
/// use dx_media::tools::archive::decompress;
///
/// decompress::auto_decompress("file.txt.gz", "file.txt").unwrap();
/// ```
pub fn auto_decompress<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let command = match ext.as_str() {
        "gz" | "gzip" => "gzip",
        "bz2" | "bzip2" => "bzip2",
        "xz" => "xz",
        "zst" | "zstd" => "zstd",
        "lz4" => "lz4",
        _ => {
            return Err(DxError::Config {
                message: format!("Unknown compression format: {}", ext),
                source: None,
            });
        }
    };

    decompress_file(input, output, command)
}

/// Decompress file using specified command.
fn decompress_file<P: AsRef<Path>>(input: P, output: P, command: &str) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    let compressed_size = std::fs::metadata(input_path).map_or(0, |m| m.len());

    let mut cmd = Command::new(command);

    // Decompress to stdout
    cmd.arg("-d")
        .arg("-k") // Keep original
        .arg("-c") // Output to stdout
        .arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run {}: {}", command, e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "{} failed: {}",
                command,
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    // Write decompressed output
    std::fs::write(output_path, &result.stdout).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to write output: {}", e),
        source: None,
    })?;

    let decompressed_size = result.stdout.len() as u64;

    Ok(ToolOutput::success_with_path(
        format!(
            "Decompressed {} -> {} bytes",
            compressed_size, decompressed_size
        ),
        output_path,
    )
    .with_metadata("compressed_size", compressed_size.to_string())
    .with_metadata("decompressed_size", decompressed_size.to_string()))
}

/// Decompress in place (replaces compressed file).
pub fn decompress_in_place<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let command = match ext.as_str() {
        "gz" | "gzip" => "gzip",
        "bz2" | "bzip2" => "bzip2",
        "xz" => "xz",
        "zst" | "zstd" => "zstd",
        "lz4" => "lz4",
        _ => {
            return Err(DxError::Config {
                message: format!("Unknown compression format: {}", ext),
                source: None,
            });
        }
    };

    let mut cmd = Command::new(command);
    cmd.arg("-d")
        .arg("-f") // Force overwrite
        .arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run {}: {}", command, e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!("{} failed", command),
            source: None,
        });
    }

    // Output file has the extension removed
    let output_path = input_path.with_extension("");

    Ok(ToolOutput::success_with_path(
        "Decompressed in place",
        &output_path,
    ))
}

/// Batch decompress multiple files.
pub fn batch_decompress<P: AsRef<Path>>(inputs: &[P], output_dir: P) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create directory: {}", e),
        source: None,
    })?;

    let mut decompressed = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();

        // Get output filename (remove compression extension)
        let file_name = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = output_dir.join(file_name);

        if auto_decompress(input_path, &output_path).is_ok() {
            decompressed.push(output_path);
        }
    }

    Ok(
        ToolOutput::success(format!("Decompressed {} files", decompressed.len()))
            .with_paths(decompressed),
    )
}

/// Test integrity of compressed file.
pub fn test_integrity<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "File not found".to_string(),
            source: None,
        });
    }

    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let command = match ext.as_str() {
        "gz" | "gzip" => "gzip",
        "bz2" | "bzip2" => "bzip2",
        "xz" => "xz",
        "zst" | "zstd" => "zstd",
        _ => {
            return Err(DxError::Config {
                message: format!("Unknown compression format: {}", ext),
                source: None,
            });
        }
    };

    let mut cmd = Command::new(command);
    cmd.arg("-t") // Test
        .arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run {}: {}", command, e),
        source: None,
    })?;

    if result.status.success() {
        Ok(ToolOutput::success("File integrity OK").with_metadata("valid", "true".to_string()))
    } else {
        Ok(
            ToolOutput::success("File integrity FAILED")
                .with_metadata("valid", "false".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_auto_detect() {
        let path = std::path::PathBuf::from("test.txt.gz");
        let ext = path.extension().unwrap().to_str().unwrap();
        assert_eq!(ext, "gz");
    }
}
