//! PDF merger.
//!
//! Combine multiple PDF files into one.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Merge multiple PDF files.
///
/// # Arguments
/// * `inputs` - Paths to PDF files to merge
/// * `output` - Path for merged output
///
/// # Example
/// ```no_run
/// use dx_media::tools::document::merge_pdfs;
///
/// merge_pdfs(&["doc1.pdf", "doc2.pdf", "doc3.pdf"], "combined.pdf").unwrap();
/// ```
pub fn merge_pdfs<P: AsRef<Path>>(inputs: &[P], output: P) -> Result<ToolOutput> {
    if inputs.is_empty() {
        return Err(DxError::Config {
            message: "No input files provided".to_string(),
            source: None,
        });
    }

    let output_path = output.as_ref();

    // Validate inputs
    for input in inputs {
        let path = input.as_ref();
        if !path.exists() {
            return Err(DxError::FileIo {
                path: path.to_path_buf(),
                message: "Input file not found".to_string(),
                source: None,
            });
        }
    }

    // Try pdftk first
    if let Ok(result) = merge_with_pdftk(inputs, output_path) {
        return Ok(result);
    }

    // Fall back to Ghostscript
    if let Ok(result) = merge_with_ghostscript(inputs, output_path) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "PDF merge failed. Install pdftk or Ghostscript.".to_string(),
        source: None,
    })
}

/// Merge using pdftk.
fn merge_with_pdftk<P: AsRef<Path>>(inputs: &[P], output: &Path) -> Result<ToolOutput> {
    let mut cmd = Command::new("pdftk");

    for input in inputs {
        cmd.arg(input.as_ref());
    }

    cmd.arg("cat").arg("output").arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pdftk: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "pdftk merge failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Merged {} PDF files", inputs.len()),
        output,
    ))
}

/// Merge using Ghostscript.
fn merge_with_ghostscript<P: AsRef<Path>>(inputs: &[P], output: &Path) -> Result<ToolOutput> {
    let gs_cmd = if cfg!(windows) { "gswin64c" } else { "gs" };

    let mut cmd = Command::new(gs_cmd);
    cmd.arg("-dBATCH")
        .arg("-dNOPAUSE")
        .arg("-q")
        .arg("-sDEVICE=pdfwrite")
        .arg(format!("-sOutputFile={}", output.to_string_lossy()));

    for input in inputs {
        cmd.arg(input.as_ref());
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run Ghostscript: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Ghostscript merge failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Merged {} PDF files", inputs.len()),
        output,
    ))
}

/// Merge PDFs in directory (sorted alphabetically).
pub fn merge_directory<P: AsRef<Path>>(input_dir: P, output: P) -> Result<ToolOutput> {
    let input_dir = input_dir.as_ref();

    if !input_dir.is_dir() {
        return Err(DxError::FileIo {
            path: input_dir.to_path_buf(),
            message: "Input is not a directory".to_string(),
            source: None,
        });
    }

    let mut pdf_files: Vec<_> = std::fs::read_dir(input_dir)
        .map_err(|e| DxError::FileIo {
            path: input_dir.to_path_buf(),
            message: format!("Failed to read directory: {}", e),
            source: None,
        })?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("pdf"))
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();

    pdf_files.sort();

    if pdf_files.is_empty() {
        return Err(DxError::Config {
            message: "No PDF files found in directory".to_string(),
            source: None,
        });
    }

    let file_refs: Vec<&Path> = pdf_files.iter().map(|p| p.as_path()).collect();
    merge_pdfs(&file_refs, output.as_ref())
}

/// Append one PDF to another.
pub fn append_pdf<P: AsRef<Path>>(base: P, append: P, output: P) -> Result<ToolOutput> {
    merge_pdfs(&[base.as_ref(), append.as_ref()], output.as_ref())
}

/// Interleave pages from two PDFs (useful for double-sided scanning).
pub fn interleave_pdfs<P: AsRef<Path>>(odd: P, even: P, output: P) -> Result<ToolOutput> {
    let odd_path = odd.as_ref();
    let even_path = even.as_ref();
    let output_path = output.as_ref();

    // This requires pdftk
    let mut cmd = Command::new("pdftk");
    cmd.arg(format!("A={}", odd_path.to_string_lossy()))
        .arg(format!("B={}", even_path.to_string_lossy()))
        .arg("shuffle")
        .arg("A")
        .arg("B")
        .arg("output")
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pdftk: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "PDF interleave failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Interleaved PDF pages",
        output_path,
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_empty_inputs() {
        let result = super::merge_pdfs::<&str>(&[], "output.pdf");
        assert!(result.is_err());
    }
}
