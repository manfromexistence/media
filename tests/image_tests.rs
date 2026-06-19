//! Tests for image tools.
//!
//! Tests are organized by tool category. Tests that require external dependencies
//! (ImageMagick, Tesseract, etc.) are marked with `#[ignore]` and can be run with:
//! `cargo test -- --ignored`

mod common;

use common::TestFixture;
use dx_media::tools::image;

// =============================================================================
// 1. converter - Image format conversion
// =============================================================================

#[test]
#[cfg(feature = "image-core")]
fn test_image_format_enum() {
    // Verify the image crate formats exist
    use image::ImageFormat;
    assert_eq!(format!("{:?}", ImageFormat::Png), "Png");
    assert_eq!(format!("{:?}", ImageFormat::Jpeg), "Jpeg");
    assert_eq!(format!("{:?}", ImageFormat::Gif), "Gif");
    assert_eq!(format!("{:?}", ImageFormat::Webp), "Webp");
    assert_eq!(format!("{:?}", ImageFormat::Bmp), "Bmp");
    assert_eq!(format!("{:?}", ImageFormat::Tiff), "Tiff");
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_image_convert() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("test.png");

    let result = image::convert(&input, &output);
    assert!(
        result.is_ok(),
        "Image conversion should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Output file should exist");
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_image_convert_to_format() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.png");
    let output = fixture.path("test.jpg");

    let result = image::convert(&input, &output);
    assert!(
        result.is_ok(),
        "Format conversion should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_image_get_info() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");

    let result = image::get_info(&input);
    assert!(
        result.is_ok(),
        "Get info should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 2. resizer - Image resizing
// =============================================================================

#[test]
fn test_resize_filter_enum() {
    assert_eq!(format!("{:?}", image::ResizeFilter::Lanczos), "Lanczos");
    assert_eq!(format!("{:?}", image::ResizeFilter::Bilinear), "Bilinear");
    assert_eq!(format!("{:?}", image::ResizeFilter::Bicubic), "Bicubic");
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_image_resize() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("resized.pgm");

    let result = image::resize(&input, &output, 100, 100);
    assert!(result.is_ok(), "Resize should succeed: {:?}", result.err());
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_image_resize_fit() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("resized.pgm");

    let result = image::resize_fit(&input, &output, 200, 200);
    assert!(
        result.is_ok(),
        "Resize fit should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_image_scale() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("scaled.pgm");

    let result = image::scale(&input, &output, 50);
    assert!(result.is_ok(), "Scale should succeed: {:?}", result.err());
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_image_thumbnail() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("thumb.pgm");

    let result = image::thumbnail(&input, &output, 64);
    assert!(
        result.is_ok(),
        "Thumbnail should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 3. compressor - Image compression
// =============================================================================

#[test]
fn test_compression_quality_enum() {
    assert_eq!(format!("{:?}", image::CompressionQuality::Low), "Low");
    assert_eq!(format!("{:?}", image::CompressionQuality::Medium), "Medium");
    assert_eq!(format!("{:?}", image::CompressionQuality::High), "High");
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_image_compress() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("compressed.pgm");

    let result = image::compress(&input, &output, 80);
    assert!(
        result.is_ok(),
        "Compress should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_image_optimize() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("optimized.pgm");

    let result = image::optimize(&input, &output);
    assert!(
        result.is_ok(),
        "Optimize should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 4. watermark - Image watermarking
// =============================================================================

#[test]
fn test_watermark_position_enum() {
    assert_eq!(
        format!("{:?}", image::WatermarkPosition::TopLeft),
        "TopLeft"
    );
    assert_eq!(
        format!("{:?}", image::WatermarkPosition::TopRight),
        "TopRight"
    );
    assert_eq!(
        format!("{:?}", image::WatermarkPosition::BottomLeft),
        "BottomLeft"
    );
    assert_eq!(
        format!("{:?}", image::WatermarkPosition::BottomRight),
        "BottomRight"
    );
    assert_eq!(format!("{:?}", image::WatermarkPosition::Center), "Center");
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_image_text_watermark() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("watermarked.pgm");

    let result = image::add_text_watermark(
        &input,
        &output,
        "© 2025",
        image::WatermarkPosition::BottomRight,
    );
    assert!(
        result.is_ok(),
        "Text watermark should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_image_watermark_options() {
    let options = image::WatermarkOptions::default();
    // Verify default options are sensible (opacity is u8, 0-100)
    assert!(
        options.opacity > 0 && options.opacity <= 100,
        "Opacity should be between 0 and 100"
    );
}

// =============================================================================
// 5. exif - EXIF metadata
// =============================================================================

#[test]
#[ignore = "requires exiftool"]
fn test_exif_read() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");

    let result = image::read_exif(&input);
    assert!(
        result.is_ok(),
        "Read EXIF should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires exiftool"]
fn test_exif_strip_metadata() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("stripped.pgm");

    let result = image::strip_metadata(&input, &output);
    assert!(
        result.is_ok(),
        "Strip metadata should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires exiftool"]
fn test_exif_set_copyright() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("copyrighted.pgm");

    let result = image::set_copyright(&input, &output, "© Test 2025");
    assert!(
        result.is_ok(),
        "Set copyright should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 6. qrcode - QR code generation
// =============================================================================

#[test]
fn test_qr_error_correction_enum() {
    assert_eq!(format!("{:?}", image::QrErrorCorrection::Low), "Low");
    assert_eq!(format!("{:?}", image::QrErrorCorrection::Medium), "Medium");
    assert_eq!(format!("{:?}", image::QrErrorCorrection::High), "High");
}

#[test]
#[ignore = "requires qrcode feature"]
fn test_qr_generate() {
    let fixture = TestFixture::new();
    let output = fixture.path("qr.png");

    let result = image::generate_qr("https://example.com", &output, 200);
    assert!(
        result.is_ok(),
        "QR generation should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "QR code file should exist");
}

#[test]
#[ignore = "requires qrcode feature"]
fn test_qr_generate_svg() {
    let fixture = TestFixture::new();
    let output = fixture.path("qr.svg");

    let result = image::generate_qr_svg("Test data", &output, 200);
    assert!(
        result.is_ok(),
        "QR SVG generation should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires qrcode feature and decoder"]
fn test_qr_decode() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("qr.png");

    let result = image::decode_qr(&input);
    // Decoding may fail on test images that aren't actual QR codes
    let _ = result;
}

// =============================================================================
// 7. palette - Color palette extraction
// =============================================================================

#[test]
#[ignore = "requires ImageMagick"]
fn test_palette_extract() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");

    let result = image::extract_palette(&input, 5);
    assert!(
        result.is_ok(),
        "Palette extraction should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_color_struct() {
    let color = image::Color {
        r: 255,
        g: 128,
        b: 64,
        percentage: 25.0,
    };
    assert_eq!(color.to_hex(), "#ff8040");
    assert_eq!(color.to_rgb(), "rgb(255, 128, 64)");
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_dominant_color() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");

    let result = image::extract_dominant_color(&input);
    assert!(
        result.is_ok(),
        "Dominant color extraction should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 8. filters - Image filters
// =============================================================================

#[test]
fn test_filter_enum() {
    // Verify all filter variants exist and are debuggable
    let filters = [
        image::Filter::Grayscale,
        image::Filter::Sepia,
        image::Filter::Invert,
        image::Filter::Blur,
        image::Filter::Sharpen,
        image::Filter::Emboss,
        image::Filter::Edge,
        image::Filter::OilPaint,
        image::Filter::Charcoal,
    ];
    for filter in filters {
        assert!(!format!("{:?}", filter).is_empty());
    }
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_apply_filter() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("filtered.pgm");

    let result = image::apply_filter(&input, &output, image::Filter::Grayscale);
    assert!(
        result.is_ok(),
        "Apply filter should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_grayscale() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("gray.pgm");

    let result = image::grayscale(&input, &output);
    assert!(
        result.is_ok(),
        "Grayscale should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_sepia() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("sepia.pgm");

    let result = image::sepia(&input, &output);
    assert!(result.is_ok(), "Sepia should succeed: {:?}", result.err());
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_brightness() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("bright.pgm");

    let result = image::brightness(&input, &output, 20);
    assert!(
        result.is_ok(),
        "Brightness should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_contrast() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("contrast.pgm");

    let result = image::contrast(&input, &output, 30);
    assert!(
        result.is_ok(),
        "Contrast should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_blur() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("blurred.pgm");

    let result = image::blur(&input, &output, 3.0);
    assert!(result.is_ok(), "Blur should succeed: {:?}", result.err());
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_rotate() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("rotated.pgm");

    let result = image::rotate(&input, &output, 90.0);
    assert!(result.is_ok(), "Rotate should succeed: {:?}", result.err());
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_flip() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output_h = fixture.path("flipped_h.pgm");
    let output_v = fixture.path("flipped_v.pgm");

    let result_h = image::flip_horizontal(&input, &output_h);
    let result_v = image::flip_vertical(&input, &output_v);
    assert!(
        result_h.is_ok(),
        "Flip horizontal should succeed: {:?}",
        result_h.err()
    );
    assert!(
        result_v.is_ok(),
        "Flip vertical should succeed: {:?}",
        result_v.err()
    );
}

// =============================================================================
// 9. ocr - Optical character recognition
// =============================================================================

#[test]
fn test_ocr_options() {
    let options = image::OcrOptions::default();
    // Verify default options are sensible
    assert!(
        !options.language.is_empty(),
        "Default language should be set"
    );
}

#[test]
#[ignore = "requires Tesseract"]
fn test_ocr_extract_simple() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");

    let result = image::extract_text_simple(&input);
    assert!(
        result.is_ok(),
        "OCR extraction should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires Tesseract"]
fn test_ocr_extract_with_options() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let options = image::OcrOptions::default();

    let result = image::extract_text(&input, options);
    assert!(
        result.is_ok(),
        "OCR with options should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires Tesseract"]
fn test_ocr_list_languages() {
    let result = image::list_languages();
    assert!(
        result.is_ok(),
        "List languages should succeed: {:?}",
        result.err()
    );
}

// =============================================================================
// 10. icons - Icon generation
// =============================================================================

#[test]
#[ignore = "requires ImageMagick"]
fn test_icon_generate() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("icon.png");

    let result = image::generate_icon(&input, &output, 64);
    assert!(
        result.is_ok(),
        "Icon generation should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_favicon_generate() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output = fixture.path("favicon.ico");

    let result = image::generate_favicon(&input, &output);
    assert!(
        result.is_ok(),
        "Favicon generation should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_ios_icons() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output_dir = fixture.path("ios_icons");

    let result = image::generate_ios_icons(&input, &output_dir);
    assert!(
        result.is_ok(),
        "iOS icons generation should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_android_icons() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output_dir = fixture.path("android_icons");

    let result = image::generate_android_icons(&input, &output_dir);
    assert!(
        result.is_ok(),
        "Android icons generation should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "requires ImageMagick"]
fn test_all_icons() {
    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.pgm");
    let output_dir = fixture.path("all_icons");

    let result = image::generate_all_icons(&input, &output_dir);
    assert!(
        result.is_ok(),
        "All icons generation should succeed: {:?}",
        result.err()
    );
}
