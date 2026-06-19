//! Document processing tools.
//!
//! This module provides 10 document manipulation tools:
//! 1. PDF Merger - Combine multiple PDFs
//! 2. PDF Splitter - Split PDF into pages
//! 3. PDF Compressor - Reduce PDF file size
//! 4. PDF to Image - Convert PDF pages to images
//! 5. Markdown Converter - Convert markdown to HTML/PDF
//! 6. HTML to PDF - Convert web pages to PDF
//! 7. Document Converter - Convert between formats
//! 8. Text Extractor - Extract text from documents
//! 9. PDF Watermark - Add watermarks to PDFs
//! 10. PDF Encryption - Password protect PDFs
//!
//! ## Native Processing
//!
//! Enable the `document-core` feature for native Rust document processing
//! using `lopdf` for PDFs and `pulldown-cmark` for Markdown.

pub mod doc_convert;
pub mod html_to_pdf;
pub mod markdown;
pub mod native;
pub mod pdf_compress;
pub mod pdf_encrypt;
pub mod pdf_merge;
pub mod pdf_split;
pub mod pdf_to_image;
pub mod pdf_watermark;
pub mod text_extract;

pub use doc_convert::*;
pub use html_to_pdf::*;
pub use markdown::*;
pub use native::*;
pub use pdf_compress::*;
pub use pdf_encrypt::*;
pub use pdf_merge::*;
pub use pdf_split::*;
pub use pdf_to_image::*;
pub use pdf_watermark::*;
pub use text_extract::*;

use crate::error::{DxError, Result};
use std::path::Path;

fn native_document_error(path: &Path, action: &str, error: std::io::Error) -> DxError {
    DxError::FileIo {
        path: path.to_path_buf(),
        message: format!("{action}: {error}"),
        source: Some(error),
    }
}

/// Document tools collection.
pub struct DocumentTools;

impl DocumentTools {
    /// Create a new DocumentTools instance.
    pub fn new() -> Self {
        Self
    }

    /// Merge multiple PDFs.
    pub fn merge_pdf<P: AsRef<Path>>(&self, inputs: &[P], output: P) -> Result<super::ToolOutput> {
        pdf_merge::merge_pdfs(inputs, output)
    }

    /// Split PDF into pages.
    pub fn split_pdf<P: AsRef<Path>>(&self, input: P, output_dir: P) -> Result<super::ToolOutput> {
        pdf_split::split_pdf(input, output_dir)
    }

    /// Compress PDF.
    pub fn compress_pdf<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        quality: CompressionQuality,
    ) -> Result<super::ToolOutput> {
        pdf_compress::compress_pdf(input, output, quality)
    }

    /// Convert PDF to images.
    pub fn pdf_to_images<P: AsRef<Path>>(
        &self,
        input: P,
        output_dir: P,
    ) -> Result<super::ToolOutput> {
        pdf_to_image::pdf_to_images(input, output_dir)
    }

    /// Convert markdown to HTML.
    pub fn markdown_to_html<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
    ) -> Result<super::ToolOutput> {
        let input = input.as_ref();
        let output = output.as_ref();
        native::markdown_to_html_native(input, output, true).map_err(|error| {
            native_document_error(output, "Failed to convert Markdown to native HTML", error)
        })
    }

    /// Convert HTML to PDF.
    pub fn html_to_pdf<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        html_to_pdf::html_to_pdf(input, output)
    }

    /// Convert document to another format.
    pub fn convert_document<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        format: doc_convert::DocFormat,
    ) -> Result<super::ToolOutput> {
        doc_convert::convert_document(input, output, format)
    }

    /// Extract text from document.
    pub fn extract_text<P: AsRef<Path>>(&self, input: P) -> Result<super::ToolOutput> {
        text_extract::extract(input)
    }

    /// Add watermark to PDF.
    pub fn watermark_pdf<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        text: &str,
    ) -> Result<super::ToolOutput> {
        pdf_watermark::text_watermark(input, output, text)
    }

    /// Encrypt PDF with password.
    pub fn encrypt_pdf<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        password: &str,
    ) -> Result<super::ToolOutput> {
        pdf_encrypt::encrypt(input, output, password)
    }

    /// Decrypt PDF with password.
    pub fn decrypt_pdf<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        password: &str,
    ) -> Result<super::ToolOutput> {
        pdf_encrypt::decrypt(input, output, password)
    }
}

impl Default for DocumentTools {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if Ghostscript is available (used for PDF operations).
pub fn check_ghostscript() -> bool {
    std::process::Command::new("gs")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if pdftk is available.
pub fn check_pdftk() -> bool {
    std::process::Command::new("pdftk")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
