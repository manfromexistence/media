//! Advanced Usage Patterns for dx-font
//!
//! Demonstrates real-world usage patterns including:
//! - Custom configuration
//! - Progress tracking
//! - Batch operations
//! - CDN URL generation

use dx_font::prelude::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> FontResult<()> {
    println!("=== Advanced dx-font Usage ===\n");

    // Example 1: Custom configuration
    custom_configuration().await?;

    // Example 2: Batch font downloads
    batch_downloads().await?;

    // Example 3: CDN URL generation
    cdn_url_generation().await?;

    // Example 4: Font filtering and selection
    font_filtering().await?;

    Ok(())
}

async fn custom_configuration() -> FontResult<()> {
    println!("1. Custom Configuration");

    let config = Config::builder()
        .output_dir(PathBuf::from("./custom_fonts"))
        .timeout_seconds(30)
        .max_retries(5)
        .cache_ttl_seconds(3600) // 1 hour
        .rate_limit_per_second(10.0)
        .max_concurrent_downloads(5)
        .build()?;

    println!("   ✓ Custom config created");
    println!("   - Output: {}", config.output_dir.display());
    println!("   - Timeout: {}s", config.timeout_seconds);
    println!("   - Max retries: {}", config.max_retries);
    println!("   - Cache TTL: {}s\n", config.cache_ttl_seconds);

    Ok(())
}

async fn batch_downloads() -> FontResult<()> {
    println!("2. Batch Font Downloads");

    let fonts_to_download = vec![
        ("roboto", vec!["woff2"]),
        ("open-sans", vec!["woff2", "ttf"]),
        ("lato", vec!["woff2"]),
    ];

    let downloader = FontDownloader::new()?;
    let output_dir = PathBuf::from("./fonts");

    println!("   Downloading {} fonts...", fonts_to_download.len());

    for (font_name, formats) in fonts_to_download {
        println!("   - {}", font_name);

        match downloader
            .download_google_font(
                font_name,
                &output_dir,
                &formats.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
                &["latin"],
            )
            .await
        {
            Ok(path) => println!("     ✓ Downloaded to {}", path.display()),
            Err(e) => println!("     ✗ Failed: {}", e),
        }
    }

    println!();
    Ok(())
}

async fn cdn_url_generation() -> FontResult<()> {
    println!("3. CDN URL Generation");

    let search = FontSearch::new()?;
    let results = search.search("roboto").await?;

    if let Some(_font) = results.fonts.first() {
        println!("   Font: Roboto");

        // Use static method instead of instance
        let urls = CdnUrlGenerator::for_google_font("roboto", "Roboto");

        println!("   CDN URLs:");
        if let Some(css) = urls.css_url {
            println!("   - CSS: {}", css);
        }
        if let Some(woff2) = urls.woff2_url {
            println!("   - WOFF2: {}", woff2);
        }
    }

    println!();
    Ok(())
}

async fn font_filtering() -> FontResult<()> {
    println!("4. Font Filtering and Selection");

    let search = FontSearch::new()?;
    let results = search.search("sans").await?;

    println!("   Total results: {}", results.total);

    // Filter by provider
    let google_fonts: Vec<_> = results
        .fonts
        .iter()
        .filter(|f| f.provider == FontProvider::GoogleFonts)
        .collect();

    println!("   Google Fonts: {}", google_fonts.len());

    // Filter by category (if available)
    let sans_serif: Vec<_> = results
        .fonts
        .iter()
        .filter(|f| {
            f.category
                .as_ref()
                .is_some_and(|c| matches!(c, FontCategory::SansSerif))
        })
        .collect();

    println!("   Sans-serif: {}", sans_serif.len());

    // Show top 5
    println!("   Top 5 results:");
    for (i, font) in results.fonts.iter().take(5).enumerate() {
        println!("   {}. {} ({})", i + 1, font.name, font.provider.name());
    }

    println!();
    Ok(())
}
