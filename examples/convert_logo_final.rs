//! Convert logo.svg to all icon sizes using native SVG support.

#[cfg(feature = "image-svg")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dx_media::tools::image::svg::generate_all_icons;

    let logo_svg = "apps/www/public/logo.svg";
    let output_dir = "apps/www/public/icons";

    println!("Converting logo.svg to all icon sizes...");

    let result = generate_all_icons(logo_svg, output_dir)?;

    println!("\n{}", result.message);
    println!("Generated {} icons", result.output_paths.len());

    Ok(())
}

#[cfg(not(feature = "image-svg"))]
fn main() {
    eprintln!("Error: This example requires the 'image-svg' feature");
    eprintln!(
        "Run with: cargo run --example convert_logo_final --features image-svg --all-features"
    );
}
