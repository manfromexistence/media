//! Convert SVG logo to all icon sizes.

use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logo_svg = "apps/www/public/logo.svg";
    let output_dir = "apps/www/public/icons";

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Icon sizes for web, iOS, Android
    let sizes = vec![
        // Favicon
        16, 32, 48, // Web icons
        64, 96, 128, 192, 256, 384, 512, // iOS
        57, 60, 72, 76, 114, 120, 144, 152, 167, 180, // Android
        36, 48, 72, 96, 144, 192, // Large
        512, 1024,
    ];

    // Remove duplicates and sort
    let mut sizes = sizes;
    sizes.sort_unstable();
    sizes.dedup();

    println!("Converting logo.svg to {} different sizes...", sizes.len());

    for size in &sizes {
        let output = format!("{}/icon-{}x{}.png", output_dir, size, size);

        // Use FFmpeg to convert SVG to PNG
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                logo_svg,
                "-vf",
                &format!("scale={}:{}", size, size),
                &output,
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("✓ Created {}x{} icon", size, size);
            }
            _ => {
                eprintln!("✗ Failed to create {}x{} icon", size, size);
            }
        }
    }

    // Create favicon.ico (multi-size ICO file)
    println!("\nCreating favicon.ico...");
    let favicon_sizes = vec![16, 32, 48];
    let mut ico_inputs = Vec::new();

    for size in &favicon_sizes {
        ico_inputs.push(format!("{}/icon-{}x{}.png", output_dir, size, size));
    }

    // Note: Creating ICO requires ImageMagick or a specialized tool
    // For now, just copy the 32x32 as favicon
    std::fs::copy(
        format!("{}/icon-32x32.png", output_dir),
        format!("{}/favicon.png", output_dir),
    )?;

    println!("\n✓ Icon conversion complete!");
    println!("Icons saved to: {}", output_dir);

    Ok(())
}
