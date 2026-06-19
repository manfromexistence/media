//! Document format converter.
//!
//! Convert between document formats (DOC, DOCX, ODT, RTF, etc.).

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Supported document formats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocFormat {
    /// Microsoft Word 97-2003
    Doc,
    /// Microsoft Word 2007+
    Docx,
    /// OpenDocument Text
    Odt,
    /// Rich Text Format
    Rtf,
    /// PDF
    Pdf,
    /// Plain Text
    Txt,
    /// HTML
    Html,
    /// EPUB
    Epub,
}

impl DocFormat {
    /// Get file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            DocFormat::Doc => "doc",
            DocFormat::Docx => "docx",
            DocFormat::Odt => "odt",
            DocFormat::Rtf => "rtf",
            DocFormat::Pdf => "pdf",
            DocFormat::Txt => "txt",
            DocFormat::Html => "html",
            DocFormat::Epub => "epub",
        }
    }

    /// LibreOffice filter name.
    pub fn libreoffice_filter(&self) -> Option<&'static str> {
        match self {
            DocFormat::Pdf => Some("writer_pdf_Export"),
            DocFormat::Docx => Some("MS Word 2007 XML"),
            DocFormat::Doc => Some("MS Word 97"),
            DocFormat::Odt => Some("writer8"),
            DocFormat::Rtf => Some("Rich Text Format"),
            DocFormat::Txt => Some("Text"),
            DocFormat::Html => Some("HTML (StarWriter)"),
            DocFormat::Epub => Some("EPUB"),
        }
    }

    /// Detect format from extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "doc" => Some(DocFormat::Doc),
            "docx" => Some(DocFormat::Docx),
            "odt" => Some(DocFormat::Odt),
            "rtf" => Some(DocFormat::Rtf),
            "pdf" => Some(DocFormat::Pdf),
            "txt" | "text" => Some(DocFormat::Txt),
            "html" | "htm" => Some(DocFormat::Html),
            "epub" => Some(DocFormat::Epub),
            _ => None,
        }
    }
}

/// Convert document to another format.
///
/// # Arguments
/// * `input` - Input document path
/// * `output` - Output path
/// * `format` - Target format
///
/// # Example
/// ```no_run
/// use dx_media::tools::document::doc_convert::{convert_document, DocFormat};
///
/// convert_document("document.docx", "output.pdf", DocFormat::Pdf).unwrap();
/// ```
pub fn convert_document<P: AsRef<Path>>(
    input: P,
    output: P,
    format: DocFormat,
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

    // Try LibreOffice first
    if let Ok(result) = convert_with_libreoffice(input_path, output_path, format) {
        return Ok(result);
    }

    // Try Pandoc
    if let Ok(result) = convert_with_pandoc(input_path, output_path, format) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "Document conversion failed. Install LibreOffice or Pandoc.".to_string(),
        source: None,
    })
}

/// Convert using LibreOffice.
fn convert_with_libreoffice(input: &Path, output: &Path, format: DocFormat) -> Result<ToolOutput> {
    let output_dir = output.parent().unwrap_or(Path::new("."));
    let output_ext = format.extension();

    // LibreOffice command line
    let lo_names = if cfg!(windows) {
        vec!["soffice", "libreoffice"]
    } else if cfg!(target_os = "macos") {
        vec![
            "/Applications/LibreOffice.app/Contents/MacOS/soffice",
            "libreoffice",
            "soffice",
        ]
    } else {
        vec!["libreoffice", "soffice"]
    };

    for lo in lo_names {
        let mut cmd = Command::new(lo);
        cmd.arg("--headless")
            .arg("--convert-to")
            .arg(output_ext)
            .arg("--outdir")
            .arg(output_dir)
            .arg(input);

        if let Ok(result) = cmd.output() {
            if result.status.success() {
                // LibreOffice outputs with same basename
                let temp_output = output_dir.join(format!(
                    "{}.{}",
                    input.file_stem().unwrap_or_default().to_string_lossy(),
                    output_ext
                ));

                // Rename if needed
                if temp_output != output {
                    if let Err(e) = std::fs::rename(&temp_output, output) {
                        // Try copy if rename fails (cross-device)
                        std::fs::copy(&temp_output, output).map_err(|_| DxError::FileIo {
                            path: output.to_path_buf(),
                            message: format!("Failed to move output: {}", e),
                            source: None,
                        })?;
                        let _ = std::fs::remove_file(&temp_output);
                    }
                }

                return Ok(ToolOutput::success_with_path(
                    format!("Converted to {} using LibreOffice", output_ext),
                    output,
                ));
            }
        }
    }

    Err(DxError::Config {
        message: "LibreOffice conversion failed".to_string(),
        source: None,
    })
}

/// Convert using Pandoc.
fn convert_with_pandoc(input: &Path, output: &Path, format: DocFormat) -> Result<ToolOutput> {
    let mut cmd = Command::new("pandoc");
    cmd.arg("-o").arg(output).arg(input);

    // Add format-specific options
    match format {
        DocFormat::Pdf => {
            cmd.arg("--pdf-engine=xelatex");
        }
        DocFormat::Epub => {
            cmd.arg("-t").arg("epub3");
        }
        _ => {}
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pandoc: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!("Pandoc failed: {}", String::from_utf8_lossy(&result.stderr)),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Converted to {} using Pandoc", format.extension()),
        output,
    ))
}

/// Convert document to PDF.
pub fn to_pdf<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    convert_document(input, output, DocFormat::Pdf)
}

/// Convert document to DOCX.
pub fn to_docx<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    convert_document(input, output, DocFormat::Docx)
}

/// Convert document to ODT.
pub fn to_odt<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    convert_document(input, output, DocFormat::Odt)
}

/// Convert document to plain text.
pub fn to_text<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    convert_document(input, output, DocFormat::Txt)
}

/// Convert document to HTML.
pub fn to_html<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    convert_document(input, output, DocFormat::Html)
}

/// Batch convert documents.
pub fn batch_convert<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    format: DocFormat,
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
        let output_path = output_dir.join(format!("{}.{}", file_stem, format.extension()));

        if convert_document(input_path, &output_path, format).is_ok() {
            converted.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!(
        "Converted {} documents to {}",
        converted.len(),
        format.extension()
    ))
    .with_paths(converted))
}

/// Get document info.
pub fn get_doc_info<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "File not found".to_string(),
            source: None,
        });
    }

    let mut output = ToolOutput::success("Document info retrieved");

    // File info
    if let Ok(metadata) = std::fs::metadata(input_path) {
        output = output.with_metadata("size", metadata.len().to_string());
    }

    // Format
    if let Some(ext) = input_path.extension().and_then(|e| e.to_str()) {
        if let Some(format) = DocFormat::from_extension(ext) {
            output = output.with_metadata("format", format!("{:?}", format));
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_extension() {
        assert_eq!(DocFormat::Pdf.extension(), "pdf");
        assert_eq!(DocFormat::Docx.extension(), "docx");
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(DocFormat::from_extension("pdf"), Some(DocFormat::Pdf));
        assert_eq!(DocFormat::from_extension("DOCX"), Some(DocFormat::Docx));
    }
}
