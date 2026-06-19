//! Tests for native Rust image processing (no external dependencies).
//!
//! These tests use the `image` crate and should work without ImageMagick.

#[cfg(feature = "image-core")]
mod common;

#[cfg(feature = "image-core")]
use common::TestFixture;

#[cfg(feature = "image-core")]
#[test]
fn test_native_image_convert_png_to_jpg() {
    use dx_media::tools::image::native::convert_native;

    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.png");
    let output = fixture.path("test.jpg");

    let result = convert_native(&input, &output, Some(85));
    assert!(
        result.is_ok(),
        "Native conversion should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Output file should exist");

    // Verify it's a valid JPEG
    let metadata = std::fs::metadata(&output).unwrap();
    assert!(metadata.len() > 0, "Output file should not be empty");
}

#[cfg(feature = "image-core")]
#[test]
fn test_native_image_convert_png_to_webp() {
    use dx_media::tools::image::native::convert_native;

    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.png");
    let output = fixture.path("test.webp");

    let result = convert_native(&input, &output, None);
    assert!(
        result.is_ok(),
        "WebP conversion should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Output file should exist");
}

#[cfg(feature = "image-core")]
#[test]
fn native_image_convert_gif_receipt_validates_gif_output_type() {
    use dx_media::tools::image::native::convert_native;

    let fixture = TestFixture::new();
    let input = fixture.create_test_image("source.png");
    let input_source = input.display().to_string();
    let output = fixture.path("animated.gif");

    let result = convert_native(&input, &output, None).expect("GIF conversion should succeed");

    assert!(output.exists(), "Output GIF should exist");
    assert_eq!(
        result
            .metadata
            .get("tool.expected_media_type")
            .map(String::as_str),
        Some("gif")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
    assert_eq!(
        result.metadata.get("tool.source").map(String::as_str),
        Some(input_source.as_str())
    );
}

#[cfg(feature = "image-core")]
#[test]
fn native_image_info_gif_receipt_validates_gif_input_type() {
    use dx_media::tools::image::native::info_native;

    let fixture = TestFixture::new();
    let input = fixture.create_test_image("source.gif");
    let input_source = input.display().to_string();

    let result = info_native(&input).expect("GIF info should succeed");

    assert_eq!(
        result
            .metadata
            .get("tool.expected_media_type")
            .map(String::as_str),
        Some("gif")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.output_extension")
            .map(String::as_str),
        Some("gif")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
    assert_eq!(
        result.metadata.get("tool.source").map(String::as_str),
        Some(input_source.as_str())
    );
}

#[cfg(feature = "image-core")]
#[test]
fn test_native_image_resize() {
    use dx_media::tools::image::native::resize_native;

    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.png");
    let output = fixture.path("resized.png");

    let result = resize_native(&input, &output, Some(50), Some(50), false);
    assert!(result.is_ok(), "Resize should succeed: {:?}", result.err());
    assert!(output.exists(), "Output file should exist");
}

#[cfg(feature = "image-core")]
#[test]
fn test_native_image_resize_keep_aspect() {
    use dx_media::tools::image::native::resize_native;

    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.png");
    let output = fixture.path("resized_aspect.png");

    let result = resize_native(&input, &output, Some(50), None, true);
    assert!(
        result.is_ok(),
        "Aspect-preserving resize should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Output file should exist");
}

#[cfg(not(feature = "image-core"))]
#[test]
fn test_image_core_feature_disabled() {
    // This test ensures the cfg-disabled suite compiles without image-core.
}
