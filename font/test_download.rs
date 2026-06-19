use dx_font::prelude::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> FontResult<()> {
    println!("=== Testing Font Download ===\n");

    // Test 1: Search for a font
    println!("1. Searching for 'roboto'...");
    let search = FontSearch::new()?;
    let results = search.search("roboto").await?;

    println!("   ✓ Found {} fonts", results.total);
    if let Some(font) = results.fonts.first() {
        println!("   First result: {} ({})", font.name, font.provider.name());
    }

    // Test 2: Download a font
    println!("\n2. Downloading Roboto font...");
    let downloader = FontDownloader::new()?;
    let output_dir = PathBuf::from("./test_fonts");

    match downloader
        .download_google_font("roboto", &output_dir, &["woff2"], &["latin"])
        .await
    {
        Ok(path) => {
            println!("   ✓ Downloaded to: {}", path.display());

            // Check if file exists
            if path.exists() {
                match std::fs::metadata(&path) {
                    Ok(metadata) => println!("   ✓ File size: {} bytes", metadata.len()),
                    Err(e) => println!("   ⚠ Could not read file metadata: {}", e),
                }
            }
        }
        Err(e) => {
            println!("   ✗ Download failed: {}", e);
        }
    }

    println!("\n=== Test Complete ===");
    Ok(())
}
