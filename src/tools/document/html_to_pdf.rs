//! HTML to PDF converter.
//!
//! Convert HTML files or web pages to PDF.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// HTML to PDF options.
#[derive(Debug, Clone)]
pub struct HtmlToPdfOptions {
    /// Page size (e.g., "A4", "Letter").
    pub page_size: String,
    /// Page orientation.
    pub orientation: PageOrientation,
    /// Margin in mm.
    pub margin: u32,
    /// Enable background graphics.
    pub background: bool,
    /// Enable JavaScript.
    pub javascript: bool,
    /// Wait time for JavaScript in ms.
    pub js_delay: u32,
}

/// Page orientation.
#[derive(Debug, Clone, Copy, Default)]
pub enum PageOrientation {
    /// Portrait orientation - taller than wide.
    #[default]
    Portrait,
    /// Landscape orientation - wider than tall.
    Landscape,
}

impl Default for HtmlToPdfOptions {
    fn default() -> Self {
        Self {
            page_size: "A4".to_string(),
            orientation: PageOrientation::Portrait,
            margin: 10,
            background: true,
            javascript: true,
            js_delay: 200,
        }
    }
}

impl HtmlToPdfOptions {
    /// A4 portrait with minimal margins.
    pub fn a4() -> Self {
        Self::default()
    }

    /// Letter size for US documents.
    pub fn letter() -> Self {
        Self {
            page_size: "Letter".to_string(),
            ..Default::default()
        }
    }

    /// Landscape orientation.
    pub fn landscape() -> Self {
        Self {
            orientation: PageOrientation::Landscape,
            ..Default::default()
        }
    }
}

/// Convert HTML file to PDF.
///
/// # Arguments
/// * `input` - Path to HTML file
/// * `output` - Path for PDF output
///
/// # Example
/// ```no_run
/// use dx_media::tools::document::html_to_pdf;
///
/// html_to_pdf("page.html", "document.pdf").unwrap();
/// ```
pub fn html_to_pdf<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    html_to_pdf_with_options(input, output, HtmlToPdfOptions::default())
}

/// Convert HTML to PDF with options.
pub fn html_to_pdf_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: HtmlToPdfOptions,
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

    // Try wkhtmltopdf first (most common)
    if let Ok(result) = convert_with_wkhtmltopdf(input_path, output_path, &options) {
        return Ok(result);
    }

    // Try Chrome/Chromium headless
    if let Ok(result) = convert_with_chrome(input_path, output_path, &options) {
        return Ok(result);
    }

    // Try weasyprint
    if let Ok(result) = convert_with_weasyprint(input_path, output_path) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "HTML to PDF conversion failed. Install wkhtmltopdf, Chrome, or weasyprint."
            .to_string(),
        source: None,
    })
}

/// Convert using wkhtmltopdf.
fn convert_with_wkhtmltopdf(
    input: &Path,
    output: &Path,
    options: &HtmlToPdfOptions,
) -> Result<ToolOutput> {
    let mut cmd = Command::new("wkhtmltopdf");

    cmd.arg("--page-size")
        .arg(&options.page_size)
        .arg("--margin-top")
        .arg(format!("{}mm", options.margin))
        .arg("--margin-bottom")
        .arg(format!("{}mm", options.margin))
        .arg("--margin-left")
        .arg(format!("{}mm", options.margin))
        .arg("--margin-right")
        .arg(format!("{}mm", options.margin));

    if matches!(options.orientation, PageOrientation::Landscape) {
        cmd.arg("--orientation").arg("Landscape");
    }

    if options.background {
        cmd.arg("--background");
    } else {
        cmd.arg("--no-background");
    }

    if options.javascript {
        cmd.arg("--enable-javascript")
            .arg("--javascript-delay")
            .arg(options.js_delay.to_string());
    } else {
        cmd.arg("--disable-javascript");
    }

    cmd.arg(input).arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run wkhtmltopdf: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "wkhtmltopdf failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Converted HTML to PDF using wkhtmltopdf",
        output,
    ))
}

/// Convert using Chrome headless.
fn convert_with_chrome(
    input: &Path,
    output: &Path,
    _options: &HtmlToPdfOptions,
) -> Result<ToolOutput> {
    // Try common Chrome executable names
    let chrome_names = if cfg!(windows) {
        vec!["chrome", "chromium", "google-chrome"]
    } else if cfg!(target_os = "macos") {
        vec![
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "chromium",
        ]
    } else {
        vec!["google-chrome", "chromium", "chromium-browser"]
    };

    let input_url = format!(
        "file://{}",
        input
            .canonicalize()
            .unwrap_or_else(|_| input.to_path_buf())
            .to_string_lossy()
    );

    for chrome in chrome_names {
        let mut cmd = Command::new(chrome);
        cmd.arg("--headless")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg(format!("--print-to-pdf={}", output.to_string_lossy()))
            .arg("--print-to-pdf-no-header")
            .arg(&input_url);

        if let Ok(result) = cmd.output() {
            if result.status.success() {
                return Ok(ToolOutput::success_with_path(
                    "Converted HTML to PDF using Chrome",
                    output,
                ));
            }
        }
    }

    Err(DxError::Config {
        message: "Chrome conversion failed".to_string(),
        source: None,
    })
}

/// Convert using weasyprint.
fn convert_with_weasyprint(input: &Path, output: &Path) -> Result<ToolOutput> {
    let mut cmd = Command::new("weasyprint");
    cmd.arg(input).arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run weasyprint: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "weasyprint failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "Converted HTML to PDF using weasyprint",
        output,
    ))
}

/// Convert URL to PDF.
pub fn url_to_pdf<P: AsRef<Path>>(url: &str, output: P) -> Result<ToolOutput> {
    let output_path = output.as_ref();

    // Try wkhtmltopdf first
    let mut cmd = Command::new("wkhtmltopdf");
    cmd.arg("--page-size").arg("A4").arg(url).arg(output_path);

    let result = cmd.output();

    if let Ok(result) = result {
        if result.status.success() {
            return Ok(ToolOutput::success_with_path(
                format!("Converted {} to PDF", url),
                output_path,
            ));
        }
    }

    // Try Chrome
    let chrome_names = if cfg!(windows) {
        vec!["chrome", "chromium"]
    } else {
        vec!["google-chrome", "chromium", "chromium-browser"]
    };

    for chrome in chrome_names {
        let mut cmd = Command::new(chrome);
        cmd.arg("--headless")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg(format!("--print-to-pdf={}", output_path.to_string_lossy()))
            .arg(url);

        if let Ok(result) = cmd.output() {
            if result.status.success() {
                return Ok(ToolOutput::success_with_path(
                    format!("Converted {} to PDF", url),
                    output_path,
                ));
            }
        }
    }

    Err(DxError::Config {
        message: "URL to PDF conversion failed".to_string(),
        source: None,
    })
}

/// Convert HTML string to PDF.
pub fn html_string_to_pdf<P: AsRef<Path>>(html: &str, output: P) -> Result<ToolOutput> {
    let output_path = output.as_ref();

    // Write HTML to temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("html_to_pdf_{}.html", std::process::id()));

    std::fs::write(&temp_file, html).map_err(|e| DxError::FileIo {
        path: temp_file.clone(),
        message: format!("Failed to write temp file: {}", e),
        source: None,
    })?;

    let result = html_to_pdf(&temp_file, &output_path.to_path_buf());

    // Clean up
    let _ = std::fs::remove_file(&temp_file);

    result
}

/// Batch convert multiple HTML files.
pub fn batch_html_to_pdf<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    options: HtmlToPdfOptions,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut converted = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
        let output_path = output_dir.join(format!("{}.pdf", file_stem));

        if html_to_pdf_with_options(input_path, &output_path, options.clone()).is_ok() {
            converted.push(output_path);
        }
    }

    Ok(
        ToolOutput::success(format!("Converted {} HTML files to PDF", converted.len()))
            .with_paths(converted),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options() {
        let a4 = HtmlToPdfOptions::a4();
        assert_eq!(a4.page_size, "A4");

        let letter = HtmlToPdfOptions::letter();
        assert_eq!(letter.page_size, "Letter");
    }
}
