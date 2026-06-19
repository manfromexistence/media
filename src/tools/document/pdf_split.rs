//! PDF splitter.
//!
//! Split PDF files into individual pages or ranges.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Split a PDF into individual pages.
///
/// # Arguments
/// * `input` - Path to input PDF
/// * `output_dir` - Directory for output pages
///
/// # Example
/// ```no_run
/// use dx_media::tools::document::split_pdf;
///
/// split_pdf("document.pdf", "./pages").unwrap();
/// // Creates: pages/page_001.pdf, pages/page_002.pdf, etc.
/// ```
pub fn split_pdf<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    // Try pdftk first
    if let Ok(result) = split_with_pdftk(input_path, output_dir) {
        return Ok(result);
    }

    // Fall back to Ghostscript
    if let Ok(result) = split_with_ghostscript(input_path, output_dir) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "PDF split failed. Install pdftk or Ghostscript.".to_string(),
        source: None,
    })
}

/// Split using pdftk.
fn split_with_pdftk(input: &Path, output_dir: &Path) -> Result<ToolOutput> {
    let pattern = output_dir.join("page_%03d.pdf");

    let mut cmd = Command::new("pdftk");
    cmd.arg(input).arg("burst").arg("output").arg(&pattern);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pdftk: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "pdftk split failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    // Remove doc_data.txt created by pdftk
    let _ = std::fs::remove_file(output_dir.join("doc_data.txt"));

    // Count output files
    let count = std::fs::read_dir(output_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
                })
                .count()
        })
        .unwrap_or(0);

    Ok(ToolOutput::success(format!("Split into {} pages", count)))
}

/// Split using Ghostscript.
fn split_with_ghostscript(input: &Path, output_dir: &Path) -> Result<ToolOutput> {
    let gs_cmd = if cfg!(windows) { "gswin64c" } else { "gs" };
    let pattern = output_dir.join("page_%03d.pdf");

    let mut cmd = Command::new(gs_cmd);
    cmd.arg("-dBATCH")
        .arg("-dNOPAUSE")
        .arg("-q")
        .arg("-sDEVICE=pdfwrite")
        .arg(format!("-sOutputFile={}", pattern.to_string_lossy()))
        .arg(input);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run Ghostscript: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Ghostscript split failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success("Split PDF into pages"))
}

/// Extract specific page range from PDF.
///
/// # Arguments
/// * `input` - Path to input PDF
/// * `output` - Path for output PDF
/// * `start` - Start page (1-indexed)
/// * `end` - End page (inclusive)
pub fn extract_pages<P: AsRef<Path>>(
    input: P,
    output: P,
    start: u32,
    end: u32,
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

    if start == 0 || end < start {
        return Err(DxError::Config {
            message: "Invalid page range".to_string(),
            source: None,
        });
    }

    // Try pdftk
    let mut cmd = Command::new("pdftk");
    cmd.arg(input_path)
        .arg("cat")
        .arg(format!("{}-{}", start, end))
        .arg("output")
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pdftk: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Page extraction failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Extracted pages {}-{}", start, end),
        output_path,
    ))
}

/// Extract single page from PDF.
pub fn extract_page<P: AsRef<Path>>(input: P, output: P, page: u32) -> Result<ToolOutput> {
    extract_pages(input, output, page, page)
}

/// Extract every Nth page.
pub fn extract_nth_pages<P: AsRef<Path>>(
    input: P,
    output: P,
    n: u32,
    offset: u32,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Get page count first
    let page_count = get_page_count(input_path)?;

    // Build page list
    let pages: Vec<String> = (offset..=page_count)
        .step_by(n as usize)
        .map(|p| p.to_string())
        .collect();

    if pages.is_empty() {
        return Err(DxError::Config {
            message: "No pages match criteria".to_string(),
            source: None,
        });
    }

    let _page_spec = pages.join(" ");

    let mut cmd = Command::new("pdftk");
    cmd.arg(input_path).arg("cat");

    for page in &pages {
        cmd.arg(page);
    }

    cmd.arg("output").arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pdftk: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Page extraction failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Extracted every {}th page ({} pages)", n, pages.len()),
        output_path,
    ))
}

/// Extract odd pages only.
pub fn extract_odd_pages<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    extract_nth_pages(input, output, 2, 1)
}

/// Extract even pages only.
pub fn extract_even_pages<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    extract_nth_pages(input, output, 2, 2)
}

/// Get the number of pages in a PDF.
pub fn get_page_count<P: AsRef<Path>>(input: P) -> Result<u32> {
    let input_path = input.as_ref();

    let mut cmd = Command::new("pdftk");
    cmd.arg(input_path).arg("dump_data");

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pdftk: {}", e),
        source: None,
    })?;

    let output = String::from_utf8_lossy(&result.stdout);

    for line in output.lines() {
        if line.starts_with("NumberOfPages:") {
            if let Some(count_str) = line.split(':').nth(1) {
                if let Ok(count) = count_str.trim().parse() {
                    return Ok(count);
                }
            }
        }
    }

    Err(DxError::Config {
        message: "Could not determine page count".to_string(),
        source: None,
    })
}

/// Remove specific pages from PDF.
pub fn remove_pages<P: AsRef<Path>>(
    input: P,
    output: P,
    pages_to_remove: &[u32],
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let total_pages = get_page_count(input_path)?;

    // Build list of pages to keep
    let pages_to_keep: Vec<String> = (1..=total_pages)
        .filter(|p| !pages_to_remove.contains(p))
        .map(|p| p.to_string())
        .collect();

    if pages_to_keep.is_empty() {
        return Err(DxError::Config {
            message: "Cannot remove all pages".to_string(),
            source: None,
        });
    }

    let mut cmd = Command::new("pdftk");
    cmd.arg(input_path).arg("cat");

    for page in &pages_to_keep {
        cmd.arg(page);
    }

    cmd.arg("output").arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pdftk: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Page removal failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Removed {} pages", pages_to_remove.len()),
        output_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_range() {
        let result = extract_pages("input.pdf", "output.pdf", 5, 2);
        assert!(result.is_err());
    }
}
