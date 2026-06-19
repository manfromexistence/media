//! PDF watermarking.
//!
//! Add text or image watermarks to PDF documents.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Watermark position.
#[derive(Debug, Clone, Copy, Default)]
pub enum WatermarkPosition {
    /// Center of the page.
    #[default]
    Center,
    /// Top-left corner of the page.
    TopLeft,
    /// Top-right corner of the page.
    TopRight,
    /// Bottom-left corner of the page.
    BottomLeft,
    /// Bottom-right corner of the page.
    BottomRight,
    /// Diagonal across the page, typically at 45 degrees.
    Diagonal,
}

/// Watermark options.
#[derive(Debug, Clone)]
pub struct WatermarkOptions {
    /// Position of watermark.
    pub position: WatermarkPosition,
    /// Opacity (0.0 - 1.0).
    pub opacity: f32,
    /// Font size for text watermarks.
    pub font_size: u32,
    /// Rotation angle in degrees.
    pub rotation: f32,
    /// Scale for image watermarks.
    pub scale: f32,
    /// Color for text watermarks (hex).
    pub color: String,
}

impl Default for WatermarkOptions {
    fn default() -> Self {
        Self {
            position: WatermarkPosition::Center,
            opacity: 0.3,
            font_size: 48,
            rotation: 0.0,
            scale: 1.0,
            color: "#808080".to_string(),
        }
    }
}

impl WatermarkOptions {
    /// Create diagonal watermark (common for "DRAFT" stamps).
    pub fn diagonal() -> Self {
        Self {
            position: WatermarkPosition::Diagonal,
            rotation: -45.0,
            opacity: 0.3,
            font_size: 72,
            ..Default::default()
        }
    }

    /// Create subtle bottom-right watermark.
    pub fn subtle() -> Self {
        Self {
            position: WatermarkPosition::BottomRight,
            opacity: 0.15,
            font_size: 24,
            ..Default::default()
        }
    }
}

/// Add text watermark to PDF.
///
/// # Arguments
/// * `input` - Input PDF path
/// * `output` - Output PDF path
/// * `text` - Watermark text
///
/// # Example
/// ```no_run
/// use dx_media::tools::document::pdf_watermark;
///
/// pdf_watermark::text_watermark("doc.pdf", "watermarked.pdf", "CONFIDENTIAL").unwrap();
/// ```
pub fn text_watermark<P: AsRef<Path>>(input: P, output: P, text: &str) -> Result<ToolOutput> {
    text_watermark_with_options(input, output, text, WatermarkOptions::default())
}

/// Add text watermark with options.
pub fn text_watermark_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    text: &str,
    options: WatermarkOptions,
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

    // Try pdftk stamp approach
    if let Ok(result) = stamp_with_pdftk(input_path, output_path, text, &options) {
        return Ok(result);
    }

    // Try qpdf
    if let Ok(result) = watermark_with_qpdf(input_path, output_path, text, &options) {
        return Ok(result);
    }

    // Try Ghostscript
    if let Ok(result) = watermark_with_gs(input_path, output_path, text, &options) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "PDF watermarking failed. Install pdftk, qpdf, or Ghostscript.".to_string(),
        source: None,
    })
}

/// Create watermark overlay PDF.
fn create_watermark_pdf(text: &str, options: &WatermarkOptions) -> Result<std::path::PathBuf> {
    let temp_dir = std::env::temp_dir();
    let watermark_pdf = temp_dir.join(format!("watermark_{}.pdf", std::process::id()));

    // Generate PostScript for watermark
    let ps_content = format!(
        r"%!PS-Adobe-3.0
<<
  /PageSize [612 792]
>> setpagedevice

/Helvetica findfont {} scalefont setfont
0.5 0.5 0.5 setrgbcolor
{} setgray

gsave
306 396 translate
{} rotate
({}) dup stringwidth pop 2 div neg 0 moveto show
grestore

showpage
",
        options.font_size, options.opacity, options.rotation, text
    );

    let ps_path = temp_dir.join(format!("watermark_{}.ps", std::process::id()));
    std::fs::write(&ps_path, ps_content).map_err(|e| DxError::FileIo {
        path: ps_path.clone(),
        message: format!("Failed to write PS file: {}", e),
        source: None,
    })?;

    // Convert PS to PDF using Ghostscript
    let gs_names = if cfg!(windows) {
        vec!["gswin64c", "gswin32c", "gs"]
    } else {
        vec!["gs"]
    };

    for gs in gs_names {
        let mut cmd = Command::new(gs);
        cmd.arg("-q")
            .arg("-dBATCH")
            .arg("-dNOPAUSE")
            .arg("-sDEVICE=pdfwrite")
            .arg(format!("-sOutputFile={}", watermark_pdf.to_string_lossy()))
            .arg(&ps_path);

        if let Ok(result) = cmd.output() {
            if result.status.success() {
                let _ = std::fs::remove_file(&ps_path);
                return Ok(watermark_pdf);
            }
        }
    }

    let _ = std::fs::remove_file(&ps_path);
    Err(DxError::Config {
        message: "Failed to create watermark PDF".to_string(),
        source: None,
    })
}

/// Apply watermark using pdftk.
fn stamp_with_pdftk(
    input: &Path,
    output: &Path,
    text: &str,
    options: &WatermarkOptions,
) -> Result<ToolOutput> {
    // Create watermark PDF
    let watermark_pdf = create_watermark_pdf(text, options)?;

    let mut cmd = Command::new("pdftk");
    cmd.arg(input)
        .arg("stamp")
        .arg(&watermark_pdf)
        .arg("output")
        .arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pdftk: {}", e),
        source: None,
    })?;

    let _ = std::fs::remove_file(&watermark_pdf);

    if !result.status.success() {
        return Err(DxError::Config {
            message: "pdftk stamp failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Added watermark '{}' to PDF", text),
        output,
    ))
}

/// Apply watermark using qpdf.
fn watermark_with_qpdf(
    input: &Path,
    output: &Path,
    text: &str,
    options: &WatermarkOptions,
) -> Result<ToolOutput> {
    // Create watermark PDF
    let watermark_pdf = create_watermark_pdf(text, options)?;

    let mut cmd = Command::new("qpdf");
    cmd.arg("--overlay")
        .arg(&watermark_pdf)
        .arg("--")
        .arg(input)
        .arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run qpdf: {}", e),
        source: None,
    })?;

    let _ = std::fs::remove_file(&watermark_pdf);

    if !result.status.success() {
        return Err(DxError::Config {
            message: "qpdf overlay failed".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Added watermark '{}' to PDF", text),
        output,
    ))
}

/// Apply watermark using Ghostscript.
fn watermark_with_gs(
    input: &Path,
    output: &Path,
    text: &str,
    options: &WatermarkOptions,
) -> Result<ToolOutput> {
    let gs_names = if cfg!(windows) {
        vec!["gswin64c", "gswin32c", "gs"]
    } else {
        vec!["gs"]
    };

    // Create PostScript overlay command
    let ps_overlay = format!(
        r"
        << /EndPage {{
            2 eq {{ pop false }}
            {{
                gsave
                0.5 0.5 0.5 setrgbcolor
                {} setgray
                /Helvetica findfont {} scalefont setfont
                306 396 moveto
                {} rotate
                ({}) dup stringwidth pop 2 div neg 0 rmoveto show
                grestore
                true
            }} ifelse
        }} bind >> setpagedevice
        ",
        options.opacity, options.font_size, options.rotation, text
    );

    for gs in gs_names {
        let mut cmd = Command::new(gs);
        cmd.arg("-q")
            .arg("-dBATCH")
            .arg("-dNOPAUSE")
            .arg("-sDEVICE=pdfwrite")
            .arg(format!("-sOutputFile={}", output.to_string_lossy()))
            .arg("-c")
            .arg(&ps_overlay)
            .arg("-f")
            .arg(input);

        if let Ok(result) = cmd.output() {
            if result.status.success() {
                return Ok(ToolOutput::success_with_path(
                    format!("Added watermark '{}' to PDF using Ghostscript", text),
                    output,
                ));
            }
        }
    }

    Err(DxError::Config {
        message: "Ghostscript watermarking failed".to_string(),
        source: None,
    })
}

/// Add image watermark to PDF.
pub fn image_watermark<P: AsRef<Path>>(
    input: P,
    output: P,
    watermark_image: P,
) -> Result<ToolOutput> {
    image_watermark_with_options(input, output, watermark_image, WatermarkOptions::default())
}

/// Add image watermark with options.
pub fn image_watermark_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    watermark_image: P,
    options: WatermarkOptions,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();
    let watermark_path = watermark_image.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input PDF not found".to_string(),
            source: None,
        });
    }

    if !watermark_path.exists() {
        return Err(DxError::FileIo {
            path: watermark_path.to_path_buf(),
            message: "Watermark image not found".to_string(),
            source: None,
        });
    }

    // Convert image to PDF first
    let temp_dir = std::env::temp_dir();
    let watermark_pdf = temp_dir.join(format!("wm_image_{}.pdf", std::process::id()));

    // Use ImageMagick to convert
    let mut cmd = Command::new("convert");
    cmd.arg(watermark_path)
        .arg("-alpha")
        .arg("set")
        .arg("-channel")
        .arg("A")
        .arg("-evaluate")
        .arg("set")
        .arg(format!("{}%", (options.opacity * 100.0) as u32))
        .arg(&watermark_pdf);

    if let Ok(result) = cmd.output() {
        if result.status.success() {
            // Now stamp with pdftk
            let mut cmd = Command::new("pdftk");
            cmd.arg(input_path)
                .arg("stamp")
                .arg(&watermark_pdf)
                .arg("output")
                .arg(output_path);

            if let Ok(result) = cmd.output() {
                let _ = std::fs::remove_file(&watermark_pdf);
                if result.status.success() {
                    return Ok(ToolOutput::success_with_path(
                        "Added image watermark to PDF",
                        output_path,
                    ));
                }
            }
        }
    }

    let _ = std::fs::remove_file(&watermark_pdf);

    Err(DxError::Config {
        message: "Image watermarking failed. Install ImageMagick and pdftk.".to_string(),
        source: None,
    })
}

/// Add "DRAFT" watermark.
pub fn draft_watermark<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    text_watermark_with_options(input, output, "DRAFT", WatermarkOptions::diagonal())
}

/// Add "CONFIDENTIAL" watermark.
pub fn confidential_watermark<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    text_watermark_with_options(input, output, "CONFIDENTIAL", WatermarkOptions::diagonal())
}

/// Add "COPY" watermark.
pub fn copy_watermark<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    text_watermark_with_options(input, output, "COPY", WatermarkOptions::diagonal())
}

/// Batch watermark multiple PDFs.
pub fn batch_watermark<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    text: &str,
    options: WatermarkOptions,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create directory: {}", e),
        source: None,
    })?;

    let mut processed = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");
        let output_path = output_dir.join(file_name);

        if text_watermark_with_options(input_path, &output_path, text, options.clone()).is_ok() {
            processed.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Watermarked {} PDFs", processed.len())).with_paths(processed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options() {
        let diagonal = WatermarkOptions::diagonal();
        assert_eq!(diagonal.rotation, -45.0);

        let subtle = WatermarkOptions::subtle();
        assert_eq!(subtle.opacity, 0.15);
    }
}
