//! OCR (Optical Character Recognition) tool.
//!
//! Extract text from images using Tesseract OCR.

use crate::deps::check_tool_dependency;
use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// OCR configuration options.
#[derive(Debug, Clone)]
pub struct OcrOptions {
    /// Language for recognition (e.g., "eng", "deu", "fra").
    pub language: String,
    /// Page segmentation mode (0-13).
    pub psm: u8,
    /// OCR Engine Mode (0-3).
    pub oem: u8,
}

impl Default for OcrOptions {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            psm: 3, // Fully automatic page segmentation
            oem: 3, // Default, based on available engines
        }
    }
}

impl OcrOptions {
    /// Create options for a single column of text.
    pub fn single_column() -> Self {
        Self {
            psm: 4,
            ..Default::default()
        }
    }

    /// Create options for a single block of text.
    pub fn single_block() -> Self {
        Self {
            psm: 6,
            ..Default::default()
        }
    }

    /// Create options for a single line.
    pub fn single_line() -> Self {
        Self {
            psm: 7,
            ..Default::default()
        }
    }

    /// Create options for a single word.
    pub fn single_word() -> Self {
        Self {
            psm: 8,
            ..Default::default()
        }
    }

    /// Set language.
    pub fn with_language(mut self, lang: &str) -> Self {
        self.language = lang.to_string();
        self
    }
}

/// Extract text from an image using OCR.
///
/// Requires Tesseract to be installed.
///
/// # Arguments
/// * `input` - Path to the image file
/// * `options` - OCR options
///
/// # Errors
///
/// Returns `DxError::MissingDependency` if Tesseract is not installed.
///
/// # Example
/// ```no_run
/// use dx_media::tools::image::ocr::{extract_text, OcrOptions};
///
/// let result = extract_text("screenshot.png", OcrOptions::default()).unwrap();
/// println!("Text: {}", result.message);
/// ```
pub fn extract_text<P: AsRef<Path>>(input: P, options: OcrOptions) -> Result<ToolOutput> {
    // Check for Tesseract dependency
    check_tool_dependency("image::ocr")?;

    let input_path = input.as_ref();

    let psm_arg = options.psm.to_string();
    let oem_arg = options.oem.to_string();

    let output = Command::new("tesseract")
        .args([
            input_path.to_str().unwrap_or(""),
            "stdout",
            "-l",
            &options.language,
            "--psm",
            &psm_arg,
            "--oem",
            &oem_arg,
        ])
        .output()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute Tesseract: {}", e),
        })?;

    if !output.status.success() {
        return Err(DxError::Internal {
            message: format!(
                "Tesseract failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(ToolOutput::success(text.clone())
        .with_metadata("language", options.language)
        .with_metadata("char_count", text.len().to_string()))
}

/// Extract text with default options.
pub fn extract_text_simple<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    extract_text(input, OcrOptions::default())
}

/// Extract text to file.
pub fn extract_text_to_file<P: AsRef<Path>>(
    input: P,
    output: P,
    options: OcrOptions,
) -> Result<ToolOutput> {
    // Check for Tesseract dependency
    check_tool_dependency("image::ocr")?;

    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Remove .txt extension if present (tesseract adds it)
    let output_base = output_path.with_extension("");

    let psm_arg = options.psm.to_string();
    let oem_arg = options.oem.to_string();

    let status = Command::new("tesseract")
        .args([
            input_path.to_str().unwrap_or(""),
            output_base.to_str().unwrap_or(""),
            "-l",
            &options.language,
            "--psm",
            &psm_arg,
            "--oem",
            &oem_arg,
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute Tesseract: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "Tesseract command failed".to_string(),
        });
    }

    // Tesseract adds .txt extension
    let actual_output = output_base.with_extension("txt");

    Ok(ToolOutput::success_with_path(
        "Text extracted to file",
        &actual_output,
    ))
}

/// Extract text as searchable PDF.
pub fn extract_text_to_pdf<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    // Check for Tesseract dependency
    check_tool_dependency("image::ocr")?;

    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let output_base = output_path.with_extension("");

    let status = Command::new("tesseract")
        .args([
            input_path.to_str().unwrap_or(""),
            output_base.to_str().unwrap_or(""),
            "pdf",
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute Tesseract: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "Tesseract PDF command failed".to_string(),
        });
    }

    let actual_output = output_base.with_extension("pdf");

    Ok(ToolOutput::success_with_path(
        "Created searchable PDF",
        &actual_output,
    ))
}

/// Extract text with HOCR output (includes position data).
pub fn extract_text_hocr<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    // Check for Tesseract dependency
    check_tool_dependency("image::ocr")?;

    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let output_base = output_path.with_extension("");

    let status = Command::new("tesseract")
        .args([
            input_path.to_str().unwrap_or(""),
            output_base.to_str().unwrap_or(""),
            "hocr",
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute Tesseract: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "Tesseract HOCR command failed".to_string(),
        });
    }

    let actual_output = output_base.with_extension("hocr");

    Ok(ToolOutput::success_with_path(
        "Created HOCR output with position data",
        &actual_output,
    ))
}

/// Preprocess image for better OCR (using ImageMagick).
pub fn preprocess_for_ocr<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-colorspace",
            "Gray",
            "-normalize",
            "-sharpen",
            "0x1",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick preprocess command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        "Image preprocessed for OCR",
        output_path,
    ))
}

/// Batch OCR multiple images.
pub fn batch_extract<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    options: OcrOptions,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut processed = 0;
    for input in inputs {
        let input_path = input.as_ref();
        let stem = input_path.file_stem().unwrap_or_default();
        let output_path = output_dir.join(format!("{}.txt", stem.to_string_lossy()));

        extract_text_to_file(input_path, &output_path, options.clone())?;
        processed += 1;
    }

    Ok(
        ToolOutput::success(format!("Extracted text from {} images", processed))
            .with_metadata("count", processed.to_string()),
    )
}

/// List available Tesseract languages.
pub fn list_languages() -> Result<ToolOutput> {
    // Check for Tesseract dependency
    check_tool_dependency("image::ocr")?;

    let output = Command::new("tesseract")
        .args(["--list-langs"])
        .output()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute Tesseract: {}", e),
        })?;

    let langs = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(ToolOutput::success(langs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_default() {
        let opts = OcrOptions::default();
        assert_eq!(opts.language, "eng");
        assert_eq!(opts.psm, 3);
    }

    #[test]
    fn test_options_single_line() {
        let opts = OcrOptions::single_line();
        assert_eq!(opts.psm, 7);
    }
}
