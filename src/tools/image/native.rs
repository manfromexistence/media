//! Native image processing using the `image` crate.
//!
//! This module provides high-performance native Rust image processing
//! as an alternative to external tools like ImageMagick.
//!
//! Enable with the `image-core` feature flag.

#[cfg(feature = "image-core")]
use image::{ImageFormat, ImageReader};

use crate::tools::ToolOutput;
#[cfg(feature = "image-core")]
use crate::tools::ToolReceipt;
#[cfg(feature = "image-core")]
use crate::types::MediaType;
#[cfg(feature = "image-core")]
use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "image-core")]
fn bitmap_media_type_for_path(path: &Path) -> MediaType {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| MediaType::Gif.matches_extension(extension))
    {
        MediaType::Gif
    } else {
        MediaType::Image
    }
}

/// Native image format conversion.
///
/// Converts between image formats using pure Rust.
/// Supports: PNG, JPEG, GIF, WebP, BMP, ICO, TIFF
#[cfg(feature = "image-core")]
pub fn convert_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    quality: Option<u8>,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    // Load image
    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // Detect output format from extension
    let format = output
        .extension()
        .and_then(|e| e.to_str())
        .and_then(|e| match e.to_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "gif" => Some(ImageFormat::Gif),
            "webp" => Some(ImageFormat::WebP),
            "bmp" => Some(ImageFormat::Bmp),
            "ico" => Some(ImageFormat::Ico),
            "tiff" | "tif" => Some(ImageFormat::Tiff),
            _ => None,
        })
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported output format",
            )
        })?;

    // Save with appropriate encoder
    match format {
        ImageFormat::Jpeg => {
            let quality = quality.unwrap_or(85);
            let file = std::fs::File::create(output)?;
            let mut writer = std::io::BufWriter::new(file);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
            img.write_with_encoder(encoder)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        _ => {
            img.save(output)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert("width".to_string(), img.width().to_string());
    metadata.insert("height".to_string(), img.height().to_string());
    metadata.insert("format".to_string(), format!("{:?}", format));

    Ok(ToolOutput::success_with_path(
        format!(
            "Converted {} to {} ({}x{})",
            input.display(),
            output.display(),
            img.width(),
            img.height()
        ),
        output,
    )
    .with_receipt(ToolReceipt::local("image.convert").with_source(input.display().to_string()))
    .with_output_type_validation(output, bitmap_media_type_for_path(output))
    .with_metadata_entries(metadata))
}

/// Native image resizing.
///
/// Resize images using high-quality Lanczos3 filter.
#[cfg(feature = "image-core")]
pub fn resize_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    width: Option<u32>,
    height: Option<u32>,
    keep_aspect: bool,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let (orig_w, orig_h) = (img.width(), img.height());

    let (new_w, new_h) = match (width, height, keep_aspect) {
        (Some(w), Some(h), false) => (w, h),
        (Some(w), Some(h), true) => {
            let ratio_w = w as f32 / orig_w as f32;
            let ratio_h = h as f32 / orig_h as f32;
            let ratio = ratio_w.min(ratio_h);
            (
                (orig_w as f32 * ratio) as u32,
                (orig_h as f32 * ratio) as u32,
            )
        }
        (Some(w), None, _) => {
            let ratio = w as f32 / orig_w as f32;
            (w, (orig_h as f32 * ratio) as u32)
        }
        (None, Some(h), _) => {
            let ratio = h as f32 / orig_h as f32;
            ((orig_w as f32 * ratio) as u32, h)
        }
        (None, None, _) => (orig_w, orig_h),
    };

    let resized = img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);

    resized
        .save(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut metadata = HashMap::new();
    metadata.insert("original_width".to_string(), orig_w.to_string());
    metadata.insert("original_height".to_string(), orig_h.to_string());
    metadata.insert("new_width".to_string(), new_w.to_string());
    metadata.insert("new_height".to_string(), new_h.to_string());

    Ok(ToolOutput::success_with_path(
        format!("Resized {}x{} -> {}x{}", orig_w, orig_h, new_w, new_h),
        output,
    )
    .with_receipt(ToolReceipt::local("image.resize").with_source(input.display().to_string()))
    .with_output_type_validation(output, bitmap_media_type_for_path(output))
    .with_metadata_entries(metadata))
}

/// Native image compression.
///
/// Compress images with quality control.
#[cfg(feature = "image-core")]
pub fn compress_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    quality: u8,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    if !has_jpeg_extension(output) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Native image compression writes JPEG output; use a .jpg or .jpeg output path",
        ));
    }

    let original_size = std::fs::metadata(input)?.len();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // Save as JPEG with specified quality
    let file = std::fs::File::create(output)?;
    let mut writer = std::io::BufWriter::new(file);
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
    img.write_with_encoder(encoder)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    drop(writer);

    let new_size = std::fs::metadata(output)?.len();
    let savings = if original_size > 0 {
        ((original_size as i128 - new_size as i128) as f64 / original_size as f64 * 100.0).round()
            as i32
    } else {
        0
    };

    let mut metadata = HashMap::new();
    metadata.insert("original_size".to_string(), original_size.to_string());
    metadata.insert("compressed_size".to_string(), new_size.to_string());
    metadata.insert("savings_percent".to_string(), savings.to_string());
    metadata.insert("quality".to_string(), quality.to_string());
    metadata.insert("actual_mime_type".to_string(), "image/jpeg".to_string());

    Ok(ToolOutput::success_with_path(
        format!(
            "Compressed {} -> {} ({} bytes -> {} bytes, {}% reduction)",
            input.display(),
            output.display(),
            original_size,
            new_size,
            savings
        ),
        output,
    )
    .with_receipt(ToolReceipt::local("image.compress").with_source(input.display().to_string()))
    .with_output_type_validation(output, bitmap_media_type_for_path(output))
    .with_metadata_entries(metadata))
}

#[cfg(feature = "image-core")]
fn has_jpeg_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "jpg" | "jpeg"))
}

#[cfg(feature = "image-core")]
fn input_extension_validation(path: &Path, expected: MediaType) -> (String, &'static str) {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| "unknown".to_string());
    let valid = expected.extensions().contains(&extension.as_str());

    (extension, if valid { "pass" } else { "fail" })
}

/// Extract color palette from image.
#[cfg(feature = "image-core")]
pub fn extract_palette_native(
    input: impl AsRef<Path>,
    num_colors: usize,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let rgba = img.to_rgba8();
    let mut color_counts: HashMap<(u8, u8, u8), usize> = HashMap::new();

    // Quantize colors (5-bit per channel for grouping)
    for pixel in rgba.pixels() {
        let r = (pixel[0] / 8) * 8;
        let g = (pixel[1] / 8) * 8;
        let b = (pixel[2] / 8) * 8;
        *color_counts.entry((r, g, b)).or_insert(0) += 1;
    }

    let mut sorted: Vec<_> = color_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let total: usize = sorted.iter().map(|(_, c)| c).sum();

    let mut metadata = HashMap::new();
    let mut colors_str = Vec::new();

    for (i, ((r, g, b), count)) in sorted.into_iter().take(num_colors).enumerate() {
        let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
        let percentage = (count as f64 / total as f64 * 100.0) as u32;

        metadata.insert(format!("color_{}", i), hex.clone());
        metadata.insert(format!("color_{}_percent", i), percentage.to_string());

        colors_str.push(format!("{} ({}%)", hex, percentage));
    }

    let expected_input = bitmap_media_type_for_path(input);
    let (input_extension, input_validation) = input_extension_validation(input, expected_input);

    Ok(ToolOutput::success(format!(
        "Extracted {} colors: {}",
        num_colors,
        colors_str.join(", ")
    ))
    .with_receipt(ToolReceipt::local("image.palette").with_source(input.display().to_string()))
    .with_metadata("tool.expected_media_type", "metadata")
    .with_metadata("tool.type_validation", "not-applicable")
    .with_metadata("tool.type_validation_scope", "metadata-only")
    .with_metadata("tool.input_expected_media_type", expected_input.as_str())
    .with_metadata("tool.input_extension", input_extension)
    .with_metadata("tool.input_type_validation", input_validation)
    .with_metadata_entries(metadata))
}

/// Apply grayscale filter.
#[cfg(feature = "image-core")]
pub fn grayscale_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let gray = img.grayscale();

    gray.save(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(ToolOutput::success_with_path(
        format!("Applied grayscale filter to {}", input.display()),
        output,
    )
    .with_receipt(
        ToolReceipt::local("image.filter.grayscale").with_source(input.display().to_string()),
    )
    .with_output_type_validation(output, bitmap_media_type_for_path(output)))
}

/// Apply blur filter.
#[cfg(feature = "image-core")]
pub fn blur_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    sigma: f32,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let blurred = img.blur(sigma);

    blurred
        .save(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut metadata = HashMap::new();
    metadata.insert("sigma".to_string(), sigma.to_string());

    Ok(ToolOutput::success_with_path(
        format!("Applied blur (sigma={}) to {}", sigma, input.display()),
        output,
    )
    .with_receipt(ToolReceipt::local("image.filter.blur").with_source(input.display().to_string()))
    .with_output_type_validation(output, bitmap_media_type_for_path(output))
    .with_metadata_entries(metadata))
}

/// Adjust image brightness.
#[cfg(feature = "image-core")]
pub fn brightness_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    value: i32,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let adjusted = img.brighten(value);

    adjusted
        .save(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut metadata = HashMap::new();
    metadata.insert("brightness".to_string(), value.to_string());

    Ok(ToolOutput::success_with_path(
        format!("Adjusted brightness ({:+}) on {}", value, input.display()),
        output,
    )
    .with_receipt(
        ToolReceipt::local("image.filter.brightness").with_source(input.display().to_string()),
    )
    .with_output_type_validation(output, bitmap_media_type_for_path(output))
    .with_metadata_entries(metadata))
}

/// Adjust image contrast.
#[cfg(feature = "image-core")]
pub fn contrast_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    value: f32,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let adjusted = img.adjust_contrast(value);

    adjusted
        .save(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut metadata = HashMap::new();
    metadata.insert("contrast".to_string(), value.to_string());

    Ok(ToolOutput::success_with_path(
        format!("Adjusted contrast ({:+.1}) on {}", value, input.display()),
        output,
    )
    .with_receipt(
        ToolReceipt::local("image.filter.contrast").with_source(input.display().to_string()),
    )
    .with_output_type_validation(output, bitmap_media_type_for_path(output))
    .with_metadata_entries(metadata))
}

/// Flip image horizontally.
#[cfg(feature = "image-core")]
pub fn flip_horizontal_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let flipped = img.fliph();

    flipped
        .save(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(
        ToolOutput::success_with_path(format!("Flipped {} horizontally", input.display()), output)
            .with_receipt(
                ToolReceipt::local("image.filter.flip-horizontal")
                    .with_source(input.display().to_string()),
            )
            .with_output_type_validation(output, bitmap_media_type_for_path(output)),
    )
}

/// Flip image vertically.
#[cfg(feature = "image-core")]
pub fn flip_vertical_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let flipped = img.flipv();

    flipped
        .save(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(
        ToolOutput::success_with_path(format!("Flipped {} vertically", input.display()), output)
            .with_receipt(
                ToolReceipt::local("image.filter.flip-vertical")
                    .with_source(input.display().to_string()),
            )
            .with_output_type_validation(output, bitmap_media_type_for_path(output)),
    )
}

/// Rotate image by 90, 180, or 270 degrees.
#[cfg(feature = "image-core")]
pub fn rotate_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    degrees: u16,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let rotated = match degrees {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Rotation must be 90, 180, or 270 degrees",
            ));
        }
    };

    rotated
        .save(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut metadata = HashMap::new();
    metadata.insert("rotation".to_string(), degrees.to_string());

    Ok(ToolOutput::success_with_path(
        format!("Rotated {} by {} degrees", input.display(), degrees),
        output,
    )
    .with_receipt(
        ToolReceipt::local("image.filter.rotate").with_source(input.display().to_string()),
    )
    .with_output_type_validation(output, bitmap_media_type_for_path(output))
    .with_metadata_entries(metadata))
}

/// Crop image to specified rectangle.
#[cfg(feature = "image-core")]
pub fn crop_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output = output.as_ref();

    let img = ImageReader::open(input)?
        .with_guessed_format()?
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let cropped = img.crop_imm(x, y, width, height);

    cropped
        .save(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut metadata = HashMap::new();
    metadata.insert("x".to_string(), x.to_string());
    metadata.insert("y".to_string(), y.to_string());
    metadata.insert("width".to_string(), width.to_string());
    metadata.insert("height".to_string(), height.to_string());

    Ok(ToolOutput::success_with_path(
        format!(
            "Cropped {} to {}x{} at ({}, {})",
            input.display(),
            width,
            height,
            x,
            y
        ),
        output,
    )
    .with_receipt(ToolReceipt::local("image.filter.crop").with_source(input.display().to_string()))
    .with_output_type_validation(output, bitmap_media_type_for_path(output))
    .with_metadata_entries(metadata))
}

/// Get image information.
#[cfg(feature = "image-core")]
pub fn info_native(input: impl AsRef<Path>) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();

    let reader = ImageReader::open(input)?.with_guessed_format()?;

    let format = reader.format().map(|f| format!("{:?}", f));

    let img = reader
        .decode()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let (width, height) = (img.width(), img.height());
    let color_type = format!("{:?}", img.color());
    let file_size = std::fs::metadata(input)?.len();

    let mut metadata = HashMap::new();
    metadata.insert("width".to_string(), width.to_string());
    metadata.insert("height".to_string(), height.to_string());
    metadata.insert("color_type".to_string(), color_type);
    metadata.insert("file_size".to_string(), file_size.to_string());
    if let Some(fmt) = format.as_ref() {
        metadata.insert("format".to_string(), fmt.clone());
    }

    Ok(ToolOutput::success(format!(
        "{}: {}x{} {:?} ({} bytes)",
        input.display(),
        width,
        height,
        format.as_deref().unwrap_or("Unknown"),
        file_size
    ))
    .with_paths(vec![input.to_path_buf()])
    .with_receipt(ToolReceipt::local("image.info").with_source(input.display().to_string()))
    .with_output_type_validation(input, bitmap_media_type_for_path(input))
    .with_metadata_entries(metadata))
}

// Fallback implementations when image-core is not enabled

/// Converts an image to a different format.
///
/// Supports common formats like PNG, JPEG, WebP, etc.
/// Requires the `image-core` feature to be enabled.
#[cfg(not(feature = "image-core"))]
pub fn convert_native(
    _input: impl AsRef<Path>,
    _output: impl AsRef<Path>,
    _quality: Option<u8>,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native image processing requires the 'image-core' feature",
    ))
}

/// Resizes an image to specified dimensions.
///
/// Can maintain aspect ratio or stretch to exact dimensions.
/// Requires the `image-core` feature to be enabled.
#[cfg(not(feature = "image-core"))]
pub fn resize_native(
    _input: impl AsRef<Path>,
    _output: impl AsRef<Path>,
    _width: Option<u32>,
    _height: Option<u32>,
    _keep_aspect: bool,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native image processing requires the 'image-core' feature",
    ))
}

/// Compresses an image with specified quality level.
///
/// Quality ranges from 0 (lowest) to 100 (highest).
/// Requires the `image-core` feature to be enabled.
#[cfg(not(feature = "image-core"))]
pub fn compress_native(
    _input: impl AsRef<Path>,
    _output: impl AsRef<Path>,
    _quality: u8,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native image processing requires the 'image-core' feature",
    ))
}
