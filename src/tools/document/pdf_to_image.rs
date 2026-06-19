//! PDF to image converter.
//!
//! Convert PDF pages to image formats (PNG, JPEG, etc.)

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Image output format.
#[derive(Debug, Clone, Copy, Default)]
pub enum ImageFormat {
    /// PNG format - lossless compression, supports transparency.
    #[default]
    Png,
    /// JPEG format - lossy compression, smaller file sizes.
    Jpeg,
    /// TIFF format - high quality, supports multiple pages.
    Tiff,
    /// BMP format - uncompressed bitmap.
    Bmp,
}

impl ImageFormat {
    /// Get file extension.
    fn extension(&self) -> &str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Bmp => "bmp",
        }
    }

    /// Get Ghostscript device name.
    fn gs_device(&self) -> &str {
        match self {
            ImageFormat::Png => "png16m",
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::Tiff => "tiff24nc",
            ImageFormat::Bmp => "bmp16m",
        }
    }
}

/// PDF to image conversion options.
#[derive(Debug, Clone)]
pub struct PdfToImageOptions {
    /// Output format.
    pub format: ImageFormat,
    /// Resolution in DPI.
    pub dpi: u32,
    /// JPEG quality (1-100, only for JPEG).
    pub quality: u8,
    /// Page range (None = all pages).
    pub pages: Option<(u32, u32)>,
}

impl Default for PdfToImageOptions {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            dpi: 150,
            quality: 90,
            pages: None,
        }
    }
}

impl PdfToImageOptions {
    /// High quality PNG output.
    pub fn high_quality_png() -> Self {
        Self {
            format: ImageFormat::Png,
            dpi: 300,
            ..Default::default()
        }
    }

    /// Web-optimized JPEG output.
    pub fn web_jpeg() -> Self {
        Self {
            format: ImageFormat::Jpeg,
            dpi: 150,
            quality: 85,
            pages: None,
        }
    }
}

/// Convert PDF to images.
///
/// # Arguments
/// * `input` - Path to input PDF
/// * `output_dir` - Directory for output images
///
/// # Example
/// ```no_run
/// use dx_media::tools::document::pdf_to_images;
///
/// pdf_to_images("document.pdf", "./images").unwrap();
/// // Creates: images/page_001.png, images/page_002.png, etc.
/// ```
pub fn pdf_to_images<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    pdf_to_images_with_options(input, output_dir, PdfToImageOptions::default())
}

/// Convert PDF to images with options.
pub fn pdf_to_images_with_options<P: AsRef<Path>>(
    input: P,
    output_dir: P,
    options: PdfToImageOptions,
) -> Result<ToolOutput> {
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

    let pattern = output_dir.join(format!("page_%03d.{}", options.format.extension()));

    let gs_cmd = if cfg!(windows) { "gswin64c" } else { "gs" };

    let mut cmd = Command::new(gs_cmd);
    cmd.arg(format!("-sDEVICE={}", options.format.gs_device()))
        .arg(format!("-r{}", options.dpi))
        .arg("-dNOPAUSE")
        .arg("-dBATCH")
        .arg("-dSAFER");

    if matches!(options.format, ImageFormat::Jpeg) {
        cmd.arg(format!("-dJPEGQ={}", options.quality));
    }

    if let Some((start, end)) = options.pages {
        cmd.arg(format!("-dFirstPage={}", start))
            .arg(format!("-dLastPage={}", end));
    }

    cmd.arg(format!("-sOutputFile={}", pattern.to_string_lossy()))
        .arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run Ghostscript: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "PDF to image conversion failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    // Count output files
    let images: Vec<_> = std::fs::read_dir(output_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == options.format.extension())
                        .unwrap_or(false)
                })
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default();

    Ok(ToolOutput::success(format!(
        "Converted PDF to {} {} images at {} DPI",
        images.len(),
        options.format.extension().to_uppercase(),
        options.dpi
    ))
    .with_paths(images))
}

/// Convert single PDF page to image.
pub fn pdf_page_to_image<P: AsRef<Path>>(
    input: P,
    output: P,
    page: u32,
    options: PdfToImageOptions,
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

    let gs_cmd = if cfg!(windows) { "gswin64c" } else { "gs" };

    let mut cmd = Command::new(gs_cmd);
    cmd.arg(format!("-sDEVICE={}", options.format.gs_device()))
        .arg(format!("-r{}", options.dpi))
        .arg("-dNOPAUSE")
        .arg("-dBATCH")
        .arg("-dSAFER")
        .arg(format!("-dFirstPage={}", page))
        .arg(format!("-dLastPage={}", page));

    if matches!(options.format, ImageFormat::Jpeg) {
        cmd.arg(format!("-dJPEGQ={}", options.quality));
    }

    cmd.arg(format!("-sOutputFile={}", output_path.to_string_lossy()))
        .arg(input_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run Ghostscript: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "Page to image conversion failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Converted page {} to image", page),
        output_path,
    ))
}

/// Extract PDF thumbnail (first page as small image).
pub fn pdf_thumbnail<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let options = PdfToImageOptions {
        format: ImageFormat::Jpeg,
        dpi: 72,
        quality: 80,
        pages: Some((1, 1)),
    };
    pdf_page_to_image(input, output, 1, options)
}

/// Create PDF preview (first few pages as images).
pub fn pdf_preview<P: AsRef<Path>>(input: P, output_dir: P, max_pages: u32) -> Result<ToolOutput> {
    let options = PdfToImageOptions {
        format: ImageFormat::Jpeg,
        dpi: 150,
        quality: 85,
        pages: Some((1, max_pages)),
    };
    pdf_to_images_with_options(input, output_dir, options)
}

/// Batch convert multiple PDFs to images.
pub fn batch_pdf_to_images<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    options: PdfToImageOptions,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut total_images = 0;

    for input in inputs {
        let input_path = input.as_ref();
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("pdf");

        let pdf_output_dir = output_dir.join(file_stem);

        if let Ok(result) = pdf_to_images_with_options(input_path, &pdf_output_dir, options.clone())
        {
            total_images += result
                .metadata
                .get("count")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
        }
    }

    Ok(ToolOutput::success(format!(
        "Converted {} PDFs to approximately {} images",
        inputs.len(),
        total_images
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.gs_device(), "jpeg");
    }

    #[test]
    fn test_options() {
        let high = PdfToImageOptions::high_quality_png();
        assert_eq!(high.dpi, 300);

        let web = PdfToImageOptions::web_jpeg();
        assert!(matches!(web.format, ImageFormat::Jpeg));
    }
}
