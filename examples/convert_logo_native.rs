//! Convert SVG logo to all icon sizes using native Rust.

#[cfg(feature = "image-core")]
use image::{ImageBuffer, Rgba};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "image-core"))]
    {
        eprintln!("Error: image-core feature required");
        eprintln!("Run with: cargo run --example convert_logo_native --features image-core");
    }

    #[cfg(feature = "image-core")]
    {
        let logo_svg = std::fs::read_to_string("apps/www/public/logo.svg")?;
        let output_dir = "apps/www/public/icons";

        // Create output directory
        std::fs::create_dir_all(output_dir)?;

        // Parse SVG and render to PNG at base size
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_str(&logo_svg, &opt)?;

        // Icon sizes
        let sizes = vec![16, 32, 48, 64, 96, 128, 192, 256, 384, 512, 1024];

        println!("Converting logo.svg to {} different sizes...", sizes.len());

        for size in &sizes {
            // Render SVG at target size
            let mut pixmap = tiny_skia::Pixmap::new(*size, *size).unwrap();

            resvg::render(
                &tree,
                tiny_skia::Transform::identity(),
                &mut pixmap.as_mut(),
            );

            // Convert to image crate format
            let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
                ImageBuffer::from_raw(*size, *size, pixmap.data().to_vec()).unwrap();

            let output = format!("{}/icon-{}x{}.png", output_dir, size, size);
            img.save(&output)?;

            println!("✓ Created {}x{} icon", size, size);
        }

        // Copy 32x32 as favicon
        std::fs::copy(
            format!("{}/icon-32x32.png", output_dir),
            format!("{}/favicon.png", output_dir),
        )?;

        println!("\n✓ Icon conversion complete!");
        println!("Icons saved to: {}", output_dir);
    }

    Ok(())
}
