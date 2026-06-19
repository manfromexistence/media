//! Image processing tools.
//!
//! This module provides 10 image manipulation tools using ImageMagick:
//! 1. Format Converter - Convert between image formats
//! 2. Smart Resizer - Resize with aspect ratio options
//! 3. Image Compressor - Reduce file size with quality control
//! 4. Watermarker - Add text/logo overlays
//! 5. EXIF Wiper - Remove metadata for privacy
//! 6. QR Code Generator/Reader - Create and decode QR codes
//! 7. Color Palette Extractor - Extract dominant colors
//! 8. Grayscale/Filter Applier - Apply visual effects
//! 9. OCR (Text Extractor) - Extract text from images
//! 10. Icon Generator - Generate favicon and app icons
//!
//! ## Native Processing
//!
//! When compiled with the `image-core` feature, native Rust implementations
//! are available that don't require external tools like ImageMagick.
//! See the `native` submodule for these functions.

pub mod compressor;
pub mod converter;
pub mod exif;
pub mod filters;
pub mod icons;
pub mod native;
pub mod ocr;
pub mod palette;
pub mod qrcode;
pub mod resizer;
pub mod svg;
pub mod watermark;

pub use compressor::*;
pub use converter::*;
pub use exif::*;
pub use filters::*;
pub use icons::*;
pub use native::*;
pub use ocr::*;
pub use palette::*;
pub use qrcode::*;
pub use resizer::*;
pub use watermark::*;

// SVG-specific exports (avoid name conflicts with icons module)
#[cfg(feature = "image-svg")]
pub use svg::{generate_icons_from_svg, generate_web_icons, svg_to_png, svg_to_png_width};

use crate::error::Result;
use std::path::Path;

/// Image tools collection.
pub struct ImageTools;

impl ImageTools {
    /// Create a new ImageTools instance.
    pub fn new() -> Self {
        Self
    }

    /// Convert image format.
    pub fn convert<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        converter::convert(input, output)
    }

    /// Resize image.
    pub fn resize<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        width: u32,
        height: u32,
    ) -> Result<super::ToolOutput> {
        resizer::resize(input, output, width, height)
    }

    /// Compress image.
    pub fn compress<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        quality: u8,
    ) -> Result<super::ToolOutput> {
        compressor::compress(input, output, quality)
    }

    /// Add text watermark to image.
    pub fn add_watermark<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        text: &str,
        position: WatermarkPosition,
    ) -> Result<super::ToolOutput> {
        watermark::add_text_watermark(input, output, text, position)
    }

    /// Remove EXIF data from image.
    pub fn strip_metadata<P: AsRef<Path>>(&self, input: P, output: P) -> Result<super::ToolOutput> {
        exif::strip_metadata(input, output)
    }

    /// Generate QR code.
    pub fn generate_qr<P: AsRef<Path>>(
        &self,
        data: &str,
        output: P,
        size: u32,
    ) -> Result<super::ToolOutput> {
        qrcode::generate_qr(data, output, size)
    }

    /// Decode QR code from image.
    pub fn decode_qr<P: AsRef<Path>>(&self, input: P) -> Result<super::ToolOutput> {
        qrcode::decode_qr(input)
    }

    /// Extract color palette from image.
    pub fn extract_palette<P: AsRef<Path>>(
        &self,
        input: P,
        num_colors: u32,
    ) -> Result<super::ToolOutput> {
        palette::extract_palette(input, num_colors)
    }

    /// Apply filter to image.
    pub fn apply_filter<P: AsRef<Path>>(
        &self,
        input: P,
        output: P,
        filter: Filter,
    ) -> Result<super::ToolOutput> {
        filters::apply_filter(input, output, filter)
    }

    /// Extract text from image (OCR).
    pub fn extract_text<P: AsRef<Path>>(&self, input: P) -> Result<super::ToolOutput> {
        ocr::extract_text(input, OcrOptions::default())
    }

    /// Generate icons from image.
    pub fn generate_icons<P: AsRef<Path>>(
        &self,
        input: P,
        output_dir: P,
    ) -> Result<super::ToolOutput> {
        icons::generate_all_icons(input, output_dir)
    }
}

impl Default for ImageTools {
    fn default() -> Self {
        Self::new()
    }
}
