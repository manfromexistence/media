//! PDF compressor.
//!
//! Reduce PDF file size by optimizing images and removing unnecessary data.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Compression quality levels.
#[derive(Debug, Clone, Copy, Default)]
pub enum CompressionQuality {
    /// Screen quality - smallest size, 72 dpi.
    Screen,
    /// eBook quality - medium size, 150 dpi.
    #[default]
    Ebook,
    /// Printer quality - good quality, 300 dpi.
    Printer,
    /// Prepress quality - high quality, 300+ dpi.
    Prepress,
    /// Default - minimal compression.
    Default,
}

impl CompressionQuality {
    /// Get Ghostscript setting name.
    fn gs_setting(&self) -> &str {
        match self {
            CompressionQuality::Screen => "/screen",
            CompressionQuality::Ebook => "/ebook",
            CompressionQuality::Printer => "/printer",
            CompressionQuality::Prepress => "/prepress",
            CompressionQuality::Default => "/default",
        }
    }

    /// Get human-readable description.
    pub fn description(&self) -> &str {
        match self {
            CompressionQuality::Screen => "Screen (72 dpi, smallest)",
            CompressionQuality::Ebook => "eBook (150 dpi, medium)",
            CompressionQuality::Printer => "Printer (300 dpi, good)",
            CompressionQuality::Prepress => "Prepress (300+ dpi, high)",
            CompressionQuality::Default => "Default (minimal compression)",
        }
    }
}

/// Compress a PDF file.
///
/// # Arguments
/// * `input` - Path to input PDF
/// * `output` - Path for compressed output
/// * `quality` - Compression quality level
///
/// # Example
/// ```no_run
/// use dx_media::tools::document::{compress_pdf, CompressionQuality};
///
/// // Compress for screen viewing
/// compress_pdf("large.pdf", "small.pdf", CompressionQuality::Screen).unwrap();
///
/// // Compress for printing
/// compress_pdf("document.pdf", "print.pdf", CompressionQuality::Printer).unwrap();
/// ```
pub fn compress_pdf<P: AsRef<Path>>(
    input: P,
    output: P,
    quality: CompressionQuality,
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

    let input_size = std::fs::metadata(input_path).map_or(0, |m| m.len());

    let gs_cmd = if cfg!(windows) { "gswin64c" } else { "gs" };

    let mut cmd = Command::new(gs_cmd);
    cmd.arg("-sDEVICE=pdfwrite")
        .arg("-dCompatibilityLevel=1.4")
        .arg(format!("-dPDFSETTINGS={}", quality.gs_setting()))
        .arg("-dNOPAUSE")
        .arg("-dQUIET")
        .arg("-dBATCH")
        .arg(format!("-sOutputFile={}", output_path.to_string_lossy()))
        .arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run Ghostscript: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "PDF compression failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    let output_size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    let reduction = if input_size > 0 {
        100.0 - (output_size as f64 / input_size as f64 * 100.0)
    } else {
        0.0
    };

    let mut result = ToolOutput::success_with_path(
        format!(
            "Compressed PDF ({} -> {} bytes, {:.1}% reduction)",
            input_size, output_size, reduction
        ),
        output_path,
    );
    result
        .metadata
        .insert("original_size".to_string(), input_size.to_string());
    result
        .metadata
        .insert("compressed_size".to_string(), output_size.to_string());
    result
        .metadata
        .insert("reduction_percent".to_string(), format!("{:.1}", reduction));

    Ok(result)
}

/// Compress PDF with custom DPI setting.
pub fn compress_pdf_custom<P: AsRef<Path>>(input: P, output: P, dpi: u32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    let input_size = std::fs::metadata(input_path).map_or(0, |m| m.len());

    let gs_cmd = if cfg!(windows) { "gswin64c" } else { "gs" };

    let mut cmd = Command::new(gs_cmd);
    cmd.arg("-sDEVICE=pdfwrite")
        .arg("-dCompatibilityLevel=1.4")
        .arg("-dNOPAUSE")
        .arg("-dQUIET")
        .arg("-dBATCH")
        .arg("-dDownsampleColorImages=true")
        .arg("-dDownsampleGrayImages=true")
        .arg("-dDownsampleMonoImages=true")
        .arg(format!("-dColorImageResolution={}", dpi))
        .arg(format!("-dGrayImageResolution={}", dpi))
        .arg(format!("-dMonoImageResolution={}", dpi))
        .arg(format!("-sOutputFile={}", output_path.to_string_lossy()))
        .arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run Ghostscript: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "PDF compression failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    let output_size = std::fs::metadata(output_path).map_or(0, |m| m.len());

    let reduction = if input_size > 0 {
        100.0 - (output_size as f64 / input_size as f64 * 100.0)
    } else {
        0.0
    };

    Ok(ToolOutput::success_with_path(
        format!(
            "Compressed PDF at {} DPI ({:.1}% reduction)",
            dpi, reduction
        ),
        output_path,
    ))
}

/// Linearize PDF for fast web viewing.
pub fn linearize_pdf<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // qpdf is better for linearization
    let mut cmd = Command::new("qpdf");
    cmd.arg("--linearize").arg(input_path).arg(output_path);

    let result = cmd.output();

    if let Ok(result) = result {
        if result.status.success() {
            return Ok(ToolOutput::success_with_path(
                "Linearized PDF for fast web viewing",
                output_path,
            ));
        }
    }

    // Fall back to Ghostscript with fast web view
    let gs_cmd = if cfg!(windows) { "gswin64c" } else { "gs" };

    let mut cmd = Command::new(gs_cmd);
    cmd.arg("-sDEVICE=pdfwrite")
        .arg("-dFastWebView=true")
        .arg("-dNOPAUSE")
        .arg("-dQUIET")
        .arg("-dBATCH")
        .arg(format!("-sOutputFile={}", output_path.to_string_lossy()))
        .arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run Ghostscript: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "PDF linearization failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Linearized PDF for fast web viewing",
        output_path,
    ))
}

/// Remove unnecessary data from PDF (metadata, forms, etc.).
pub fn clean_pdf<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let gs_cmd = if cfg!(windows) { "gswin64c" } else { "gs" };

    let mut cmd = Command::new(gs_cmd);
    cmd.arg("-sDEVICE=pdfwrite")
        .arg("-dNOPAUSE")
        .arg("-dQUIET")
        .arg("-dBATCH")
        .arg("-dDetectDuplicateImages=true")
        .arg("-dCompressFonts=true")
        .arg("-dEmbedAllFonts=true")
        .arg("-dSubsetFonts=true")
        .arg(format!("-sOutputFile={}", output_path.to_string_lossy()))
        .arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run Ghostscript: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "PDF cleaning failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Cleaned PDF (removed duplicates, optimized fonts)",
        output_path,
    ))
}

/// Batch compress multiple PDFs.
pub fn batch_compress<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    quality: CompressionQuality,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut compressed = Vec::new();
    let mut total_saved: u64 = 0;

    for input in inputs {
        let input_path = input.as_ref();
        let file_name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");
        let output_path = output_dir.join(format!("compressed_{}", file_name));

        let input_size = std::fs::metadata(input_path).map_or(0, |m| m.len());

        if compress_pdf(input_path, &output_path, quality).is_ok() {
            let output_size = std::fs::metadata(&output_path).map_or(0, |m| m.len());
            total_saved += input_size.saturating_sub(output_size);
            compressed.push(output_path);
        }
    }

    let mut result = ToolOutput::success(format!(
        "Compressed {} PDFs (saved {} bytes total)",
        compressed.len(),
        total_saved
    ));
    result
        .metadata
        .insert("total_saved".to_string(), total_saved.to_string());

    Ok(result.with_paths(compressed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_quality() {
        assert_eq!(CompressionQuality::Screen.gs_setting(), "/screen");
        assert_eq!(CompressionQuality::Printer.gs_setting(), "/printer");
    }
}
