//! Generate favicons from logo.svg

#[cfg(feature = "image-svg")]
use dx_media::tools::image::svg::generate_web_icons;

fn main() -> std::io::Result<()> {
    #[cfg(not(feature = "image-svg"))]
    {
        eprintln!("Error: image-svg feature required");
        eprintln!("Run with: cargo run --example generate_favicons --features image-svg");
    }

    #[cfg(feature = "image-svg")]
    {
        let logo = "apps/www/public/logo.svg";
        let output_dir = "apps/www/public";

        println!("Generating favicons from {}...", logo);

        let result = generate_web_icons(logo, output_dir)?;

        println!("{}", result.message);
        println!("Generated {} files", result.output_paths.len());

        for path in &result.output_paths {
            println!("  - {}", path.display());
        }
    }

    Ok(())
}
