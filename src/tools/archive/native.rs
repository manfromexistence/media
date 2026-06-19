//! Native archive processing using pure Rust crates.
//!
//! This module provides high-performance native Rust archive handling
//! as an alternative to external tools.
//!
//! Enable with the `archive-core` feature flag.

use std::path::Path;

#[cfg(feature = "archive-core")]
use std::collections::HashMap;
#[cfg(feature = "archive-core")]
use std::io::{Read, Write};
#[cfg(feature = "archive-core")]
use std::path::{Component, PathBuf};

use crate::tools::ToolOutput;
#[cfg(feature = "archive-core")]
use crate::tools::ToolReceipt;

#[cfg(feature = "archive-core")]
fn archive_input_sources<T: AsRef<Path>>(inputs: &[T]) -> String {
    inputs
        .iter()
        .map(|input| input.as_ref().display().to_string())
        .collect::<Vec<_>>()
        .join(";")
}

#[cfg(feature = "archive-core")]
fn validate_archive_inputs<T: AsRef<Path>>(inputs: &[T]) -> std::io::Result<()> {
    if inputs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Archive creation requires at least one input",
        ));
    }

    for input in inputs {
        let input = input.as_ref();
        if !input.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Archive input does not exist: {}", input.display()),
            ));
        }
        if !input.is_file() && !input.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Archive input is not a regular file or directory: {}",
                    input.display()
                ),
            ));
        }
    }

    Ok(())
}

#[cfg(feature = "archive-core")]
fn archive_output_extension(path: &Path) -> String {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if filename.ends_with(".tar.gz") {
        "tar.gz".to_string()
    } else {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_else(|| "unknown".to_string())
    }
}

#[cfg(feature = "archive-core")]
fn validate_archive_output_extension(
    tool_name: &'static str,
    path: &Path,
    expected_extension: &'static str,
) -> std::io::Result<()> {
    let actual_extension = archive_output_extension(path);
    if actual_extension == expected_extension {
        return Ok(());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!(
            "{tool_name} failed type validation: archive-extension-mismatch \
             (expected .{expected_extension}, got .{actual_extension})"
        ),
    ))
}

#[cfg(feature = "archive-core")]
fn validate_archive_input_extension(
    tool_name: &'static str,
    path: &Path,
    expected_extension: &'static str,
) -> std::io::Result<()> {
    let actual_extension = archive_output_extension(path);
    if actual_extension == expected_extension {
        return Ok(());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!(
            "{tool_name} failed type validation: archive-input-extension-mismatch \
             (expected .{expected_extension}, got .{actual_extension})"
        ),
    ))
}

#[cfg(feature = "archive-core")]
fn with_archive_type_validation(
    output: ToolOutput,
    path: &Path,
    expected_extension: &'static str,
) -> ToolOutput {
    let actual_extension = archive_output_extension(path);
    let valid = actual_extension == expected_extension;

    let mut output = output
        .with_metadata("tool.expected_media_type", "archive")
        .with_metadata("tool.expected_output_extension", expected_extension)
        .with_metadata("tool.output_extension", actual_extension)
        .with_metadata("tool.type_validation", if valid { "pass" } else { "fail" });

    if !valid {
        output = output.with_metadata("tool.type_validation_reason", "archive-extension-mismatch");
    }

    output
}

#[cfg(feature = "archive-core")]
fn with_archive_input_type_validation(
    output: ToolOutput,
    path: &Path,
    expected_extension: &'static str,
) -> ToolOutput {
    let actual_extension = archive_output_extension(path);

    output
        .with_metadata("tool.expected_media_type", "archive")
        .with_metadata("tool.expected_input_extension", expected_extension)
        .with_metadata("tool.input_extension", actual_extension)
        .with_metadata("tool.type_validation", "pass")
}

/// Native ZIP extraction.
#[cfg(feature = "archive-core")]
pub fn extract_zip_native(
    input: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    use zip::ZipArchive;

    let input = input.as_ref();
    let output_dir = output_dir.as_ref();
    validate_archive_input_extension("archive.unzip.native", input, "zip")?;

    std::fs::create_dir_all(output_dir)?;

    let file = std::fs::File::open(input)?;
    let mut archive = ZipArchive::new(file)?;

    let mut extracted = Vec::new();
    let total_files = archive.len();

    for i in 0..total_files {
        let mut file = archive.by_index(i)?;
        let outpath = safe_archive_output_path(output_dir, Path::new(file.name()))?;

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
            extracted.push(outpath);
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert("total_files".to_string(), total_files.to_string());
    metadata.insert("extracted_files".to_string(), extracted.len().to_string());

    let output = ToolOutput::success(format!(
        "Extracted {} files from {} to {}",
        extracted.len(),
        input.display(),
        output_dir.display()
    ))
    .with_paths(extracted)
    .with_receipt(
        ToolReceipt::local("archive.unzip.native").with_source(input.display().to_string()),
    )
    .with_metadata_entries(metadata);

    Ok(with_archive_input_type_validation(output, input, "zip"))
}

/// Native ZIP creation.
#[cfg(feature = "archive-core")]
pub fn create_zip_native(
    inputs: &[impl AsRef<Path>],
    output: impl AsRef<Path>,
    compression_level: Option<i64>,
) -> std::io::Result<ToolOutput> {
    use walkdir::WalkDir;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    let output = output.as_ref();
    validate_archive_inputs(inputs)?;
    validate_archive_output_extension("archive.zip.native", output, "zip")?;
    let input_sources = archive_input_sources(inputs);
    let file = std::fs::File::create(output)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(compression_level);

    let mut file_count = 0;
    let mut total_size = 0u64;

    for input in inputs {
        let input = input.as_ref();

        if input.is_dir() {
            for entry in WalkDir::new(input).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                let relative = path
                    .strip_prefix(input.parent().unwrap_or(input))
                    .unwrap_or(path);

                if path.is_file() {
                    zip.start_file(relative.to_string_lossy(), options)?;
                    let mut f = std::fs::File::open(path)?;
                    let mut buffer = Vec::new();
                    f.read_to_end(&mut buffer)?;
                    zip.write_all(&buffer)?;

                    file_count += 1;
                    total_size += buffer.len() as u64;
                } else if path.is_dir() && path != input {
                    zip.add_directory(relative.to_string_lossy(), options)?;
                }
            }
        } else if input.is_file() {
            let name = input.file_name().unwrap_or_default();
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = std::fs::File::open(input)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;

            file_count += 1;
            total_size += buffer.len() as u64;
        }
    }

    zip.finish()?;

    let compressed_size = std::fs::metadata(output)?.len();
    let compression_ratio = if total_size > 0 {
        (1.0 - (compressed_size as f64 / total_size as f64)) * 100.0
    } else {
        0.0
    };

    let mut metadata = HashMap::new();
    metadata.insert("file_count".to_string(), file_count.to_string());
    metadata.insert("original_size".to_string(), total_size.to_string());
    metadata.insert("compressed_size".to_string(), compressed_size.to_string());
    metadata.insert(
        "compression_ratio".to_string(),
        format!("{:.1}%", compression_ratio),
    );

    let tool_output = ToolOutput::success_with_path(
        format!(
            "Created {} with {} files ({:.1}% compression)",
            output.display(),
            file_count,
            compression_ratio
        ),
        output,
    )
    .with_receipt(ToolReceipt::local("archive.zip.native").with_source(input_sources))
    .with_metadata_entries(metadata);

    Ok(with_archive_type_validation(tool_output, output, "zip"))
}

/// Native tar.gz extraction.
#[cfg(feature = "archive-core")]
pub fn extract_tar_gz_native(
    input: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let input = input.as_ref();
    let output_dir = output_dir.as_ref();
    validate_archive_input_extension("archive.untar-gz.native", input, "tar.gz")?;

    std::fs::create_dir_all(output_dir)?;

    let file = std::fs::File::open(input)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    let mut extracted = Vec::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        let outpath = safe_archive_output_path(output_dir, &path)?;
        let entry_type = entry.header().entry_type();

        if entry_type.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else if entry_type.is_file() {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            entry.unpack(&outpath)?;
            extracted.push(outpath);
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Unsupported or unsafe tar entry type for {}",
                    path.display()
                ),
            ));
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert("extracted_files".to_string(), extracted.len().to_string());

    let output = ToolOutput::success(format!(
        "Extracted {} files from {} to {}",
        extracted.len(),
        input.display(),
        output_dir.display()
    ))
    .with_paths(extracted)
    .with_receipt(
        ToolReceipt::local("archive.untar-gz.native").with_source(input.display().to_string()),
    )
    .with_metadata_entries(metadata);

    Ok(with_archive_input_type_validation(output, input, "tar.gz"))
}

#[cfg(feature = "archive-core")]
fn safe_archive_output_path(output_dir: &Path, entry_path: &Path) -> std::io::Result<PathBuf> {
    let mut safe_relative = PathBuf::new();

    for component in entry_path.components() {
        match component {
            Component::Normal(part) => safe_relative.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unsafe archive entry path: {}", entry_path.display()),
                ));
            }
        }
    }

    if safe_relative.as_os_str().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Archive entry path is empty",
        ));
    }

    Ok(output_dir.join(safe_relative))
}

/// Native tar.gz creation.
#[cfg(feature = "archive-core")]
pub fn create_tar_gz_native(
    inputs: &[impl AsRef<Path>],
    output: impl AsRef<Path>,
    compression_level: Option<u32>,
) -> std::io::Result<ToolOutput> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use tar::Builder;

    let output = output.as_ref();
    validate_archive_inputs(inputs)?;
    validate_archive_output_extension("archive.tar-gz.native", output, "tar.gz")?;
    let input_sources = archive_input_sources(inputs);
    let file = std::fs::File::create(output)?;
    let level = compression_level.unwrap_or(6);
    let encoder = GzEncoder::new(file, Compression::new(level));
    let mut tar = Builder::new(encoder);

    let mut file_count = 0;

    for input in inputs {
        let input = input.as_ref();

        if input.is_dir() {
            tar.append_dir_all(input.file_name().unwrap_or_default(), input)?;
            file_count += walkdir::WalkDir::new(input)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .count();
        } else if input.is_file() {
            tar.append_path_with_name(input, input.file_name().unwrap_or_default())?;
            file_count += 1;
        }
    }

    tar.into_inner()?.finish()?;

    let compressed_size = std::fs::metadata(output)?.len();

    let mut metadata = HashMap::new();
    metadata.insert("file_count".to_string(), file_count.to_string());
    metadata.insert("compressed_size".to_string(), compressed_size.to_string());

    let tool_output = ToolOutput::success_with_path(
        format!("Created {} with {} files", output.display(), file_count),
        output,
    )
    .with_receipt(ToolReceipt::local("archive.tar-gz.native").with_source(input_sources))
    .with_metadata_entries(metadata);

    Ok(with_archive_type_validation(tool_output, output, "tar.gz"))
}

/// List contents of a ZIP archive.
#[cfg(feature = "archive-core")]
pub fn list_zip_native(input: impl AsRef<Path>) -> std::io::Result<ToolOutput> {
    use zip::ZipArchive;

    let input = input.as_ref();
    validate_archive_input_extension("archive.list-zip.native", input, "zip")?;

    let file = std::fs::File::open(input)?;
    let mut archive = ZipArchive::new(file)?;

    let mut files = Vec::new();
    let mut total_size = 0u64;
    let mut compressed_size = 0u64;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        files.push(file.name().to_string());
        total_size += file.size();
        compressed_size += file.compressed_size();
    }

    let mut metadata = HashMap::new();
    metadata.insert("file_count".to_string(), files.len().to_string());
    metadata.insert("total_size".to_string(), total_size.to_string());
    metadata.insert("compressed_size".to_string(), compressed_size.to_string());
    metadata.insert("files".to_string(), files.join(";"));

    let output = ToolOutput::success(format!(
        "{}: {} files, {} bytes (compressed: {} bytes)",
        input.display(),
        files.len(),
        total_size,
        compressed_size
    ))
    .with_paths(vec![input.to_path_buf()])
    .with_receipt(
        ToolReceipt::local("archive.list-zip.native").with_source(input.display().to_string()),
    )
    .with_metadata_entries(metadata);

    Ok(with_archive_input_type_validation(output, input, "zip"))
}

/// Extract specific file from ZIP.
#[cfg(feature = "archive-core")]
pub fn extract_file_from_zip_native(
    archive_path: impl AsRef<Path>,
    file_name: &str,
    output: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    use zip::ZipArchive;

    let archive_path = archive_path.as_ref();
    let output = output.as_ref();
    validate_archive_input_extension("archive.extract-file.native", archive_path, "zip")?;

    let file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut zip_file = archive.by_name(file_name)?;

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut outfile = std::fs::File::create(output)?;
    std::io::copy(&mut zip_file, &mut outfile)?;

    let mut metadata = HashMap::new();
    metadata.insert("file_name".to_string(), file_name.to_string());
    metadata.insert("size".to_string(), zip_file.size().to_string());

    let tool_output = ToolOutput::success_with_path(
        format!(
            "Extracted {} from {} to {}",
            file_name,
            archive_path.display(),
            output.display()
        ),
        output,
    )
    .with_receipt(
        ToolReceipt::local("archive.extract-file.native")
            .with_source(archive_path.display().to_string()),
    )
    .with_metadata_entries(metadata);

    Ok(with_archive_input_type_validation(
        tool_output,
        archive_path,
        "zip",
    ))
}

// Fallback implementations when archive-core is not enabled
#[cfg(not(feature = "archive-core"))]
/// Returns an unsupported error when native ZIP extraction is not compiled in.
pub fn extract_zip_native(
    _input: impl AsRef<Path>,
    _output_dir: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native archive processing requires the 'archive-core' feature",
    ))
}

#[cfg(not(feature = "archive-core"))]
/// Returns an unsupported error when native ZIP creation is not compiled in.
pub fn create_zip_native(
    _inputs: &[impl AsRef<Path>],
    _output: impl AsRef<Path>,
    _compression_level: Option<i64>,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native archive processing requires the 'archive-core' feature",
    ))
}

#[cfg(not(feature = "archive-core"))]
/// Returns an unsupported error when native tar.gz extraction is not compiled in.
pub fn extract_tar_gz_native(
    _input: impl AsRef<Path>,
    _output_dir: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native archive processing requires the 'archive-core' feature",
    ))
}

#[cfg(not(feature = "archive-core"))]
/// Returns an unsupported error when native tar.gz creation is not compiled in.
pub fn create_tar_gz_native(
    _inputs: &[impl AsRef<Path>],
    _output: impl AsRef<Path>,
    _compression_level: Option<u32>,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native archive processing requires the 'archive-core' feature",
    ))
}

#[cfg(not(feature = "archive-core"))]
/// Returns an unsupported error when native ZIP listing is not compiled in.
pub fn list_zip_native(_input: impl AsRef<Path>) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native archive processing requires the 'archive-core' feature",
    ))
}

#[cfg(not(feature = "archive-core"))]
/// Returns an unsupported error when native ZIP single-file extraction is not compiled in.
pub fn extract_file_from_zip_native(
    _archive_path: impl AsRef<Path>,
    _file_name: &str,
    _output: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native archive processing requires the 'archive-core' feature",
    ))
}
