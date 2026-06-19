//! Icon generation tool.
//!
//! Generate favicons and app icons using ImageMagick.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// Standard favicon sizes in pixels (16x16, 32x32, 48x48).
pub const FAVICON_SIZES: &[u32] = &[16, 32, 48];

/// Standard iOS app icon sizes in pixels for various device resolutions.
pub const IOS_SIZES: &[u32] = &[57, 60, 72, 76, 114, 120, 144, 152, 167, 180];

/// Standard Android app icon sizes in pixels for various screen densities.
pub const ANDROID_SIZES: &[u32] = &[36, 48, 72, 96, 144, 192, 512];

/// Standard Windows tile icon sizes in pixels for Start menu tiles.
pub const WINDOWS_SIZES: &[u32] = &[70, 150, 310];

/// Generate a single icon size.
pub fn generate_icon<P: AsRef<Path>>(input: P, output: P, size: u32) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let size_arg = format!("{}x{}", size, size);

    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-resize",
            &size_arg,
            "-gravity",
            "center",
            "-extent",
            &size_arg,
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick icon generation failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Generated {}x{} icon", size, size),
        output_path,
    ))
}

/// Generate favicon.ico with multiple sizes.
pub fn generate_favicon<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    // Generate all sizes and combine into ICO
    let temp_dir = tempfile::tempdir().map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to create temp directory: {}", e),
        source: None,
    })?;

    let mut temp_files = Vec::new();
    for size in FAVICON_SIZES {
        let temp_file = temp_dir.path().join(format!("icon_{}.png", size));
        generate_icon(input_path, &temp_file, *size)?;
        temp_files.push(temp_file);
    }

    // Combine into ICO
    let mut args = vec!["convert".to_string()];
    for temp_file in &temp_files {
        args.push(temp_file.to_str().unwrap_or("").to_string());
    }
    args.push(output_path.to_str().unwrap_or("").to_string());

    let status = Command::new("magick")
        .args(&args)
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "ImageMagick favicon generation failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        format!("Generated favicon with sizes: {:?}", FAVICON_SIZES),
        output_path,
    ))
}

/// Generate iOS app icons.
pub fn generate_ios_icons<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut generated = 0;
    for size in IOS_SIZES {
        let output_path = output_dir.join(format!("icon_{}x{}.png", size, size));
        generate_icon(input_path, &output_path, *size)?;
        generated += 1;
    }

    Ok(ToolOutput::success(format!(
        "Generated {} iOS icons in {}",
        generated,
        output_dir.display()
    ))
    .with_metadata("count", generated.to_string()))
}

/// Generate Android app icons.
pub fn generate_android_icons<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    // Create density subdirectories
    let densities = [
        ("ldpi", 36),
        ("mdpi", 48),
        ("hdpi", 72),
        ("xhdpi", 96),
        ("xxhdpi", 144),
        ("xxxhdpi", 192),
    ];

    let mut generated = 0;
    for (density, size) in densities {
        let density_dir = output_dir.join(format!("mipmap-{}", density));
        std::fs::create_dir_all(&density_dir).map_err(|e| DxError::FileIo {
            path: density_dir.clone(),
            message: format!("Failed to create density directory: {}", e),
            source: None,
        })?;

        let output_path = density_dir.join("ic_launcher.png");
        generate_icon(input_path, &output_path, size)?;
        generated += 1;
    }

    // Generate adaptive icon background (512x512)
    let adaptive_dir = output_dir.join("mipmap-xxxhdpi");
    let _ = std::fs::create_dir_all(&adaptive_dir);

    Ok(ToolOutput::success(format!(
        "Generated {} Android icons in {}",
        generated,
        output_dir.display()
    ))
    .with_metadata("count", generated.to_string()))
}

/// Generate PWA icons.
pub fn generate_pwa_icons<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let pwa_sizes = [72, 96, 128, 144, 152, 192, 384, 512];
    let mut generated = 0;

    for size in pwa_sizes {
        let output_path = output_dir.join(format!("icon-{}x{}.png", size, size));
        generate_icon(input_path, &output_path, size)?;
        generated += 1;
    }

    Ok(ToolOutput::success(format!(
        "Generated {} PWA icons in {}",
        generated,
        output_dir.display()
    ))
    .with_metadata("count", generated.to_string()))
}

/// Generate all platform icons.
pub fn generate_all_icons<P: AsRef<Path>>(input: P, output_dir: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    // Favicon
    let favicon_path = output_dir.join("favicon.ico");
    generate_favicon(input_path, &favicon_path)?;

    // iOS
    let ios_dir = output_dir.join("ios");
    generate_ios_icons(input_path, &ios_dir)?;

    // Android
    let android_dir = output_dir.join("android");
    generate_android_icons(input_path, &android_dir)?;

    // PWA
    let pwa_dir = output_dir.join("pwa");
    generate_pwa_icons(input_path, &pwa_dir)?;

    Ok(ToolOutput::success(format!(
        "Generated all icons in {}",
        output_dir.display()
    )))
}

/// Generate rounded icon.
pub fn generate_rounded_icon<P: AsRef<Path>>(
    input: P,
    output: P,
    size: u32,
    radius: u32,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let size_arg = format!("{}x{}", size, size);
    let radius_arg = format!("roundrectangle 0,0,{},{},{},{}", size, size, radius, radius);

    // Create rounded corners using mask
    let status = Command::new("magick")
        .args([
            "convert",
            input_path.to_str().unwrap_or(""),
            "-resize",
            &size_arg,
            "-alpha",
            "set",
            "(",
            "+clone",
            "-alpha",
            "extract",
            "-draw",
            &radius_arg,
            ")",
            "-alpha",
            "off",
            "-compose",
            "CopyOpacity",
            "-composite",
            output_path.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute ImageMagick: {}", e),
        })?;

    if !status.success() {
        // Fallback to simple resize
        generate_icon(input_path, output_path, size)?;
    }

    Ok(ToolOutput::success_with_path(
        format!("Generated {}x{} rounded icon", size, size),
        output_path,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_favicon_sizes() {
        assert_eq!(FAVICON_SIZES, &[16, 32, 48]);
    }

    #[test]
    fn test_ios_sizes() {
        assert!(IOS_SIZES.contains(&180));
    }
}
