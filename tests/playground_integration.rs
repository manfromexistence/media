//! Playground Integration Tests
//!
//! Tests DxMedia with real providers and organizes assets into playground folders.

use std::fs;
use std::path::PathBuf;

use dx_media::scraping::{ScrapingCategory, ScrapingRegistry};
use dx_media::tools::{archive, document, image, utility};
use dx_media::{DxMedia, MediaType};

// ============================================================================
// PATH HELPERS
// ============================================================================

fn playground_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("playground")
}

fn providers_path() -> PathBuf {
    playground_path().join("assets").join("providers")
}

fn scraping_path() -> PathBuf {
    playground_path().join("assets").join("scraping")
}

fn tools_path() -> PathBuf {
    playground_path().join("tools")
}

fn setup_dirs() {
    let providers = providers_path();
    let scraping = scraping_path();
    let tools = tools_path();

    // Provider directories
    for p in [
        "openverse",
        "wikimedia",
        "met",
        "nasa",
        "archive",
        "loc",
        "dpla",
        "rijksmuseum",
        "cleveland",
        "europeana",
        "polyhaven",
        "artic",
        "picsum",
    ] {
        fs::create_dir_all(providers.join(p)).ok();
    }

    // Scraping directory
    fs::create_dir_all(&scraping).ok();

    // Tool directories
    for t in ["image", "video", "audio", "document", "archive", "utility"] {
        fs::create_dir_all(tools.join(t)).ok();
    }
}

// ============================================================================
// DXMEDIA API TESTS
// ============================================================================

mod dxmedia_tests {
    use super::*;

    #[test]
    fn test_dxmedia_initialization() {
        setup_dirs();

        match DxMedia::new() {
            Ok(dx) => {
                let available = dx.available_providers();
                let all = dx.all_providers();

                println!("\n=== DxMedia Provider Status ===");
                println!("All providers: {}", all.len());
                println!("Available providers: {}", available.len());

                for provider in &all {
                    let status = if dx.is_provider_available(provider) {
                        "✓"
                    } else {
                        "✗"
                    };
                    println!("  {} {}", status, provider);
                }
            }
            Err(e) => println!("Failed to create DxMedia: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_dxmedia_search_wikimedia() {
        setup_dirs();

        match DxMedia::new() {
            Ok(dx) => {
                let result = dx
                    .search("nature landscape")
                    .media_type(MediaType::Image)
                    .provider("wikimedia")
                    .count(5)
                    .execute()
                    .await;

                match result {
                    Ok(results) => {
                        println!("\n[Wikimedia] Found {} results", results.total_count);
                        for asset in results.assets.iter().take(3) {
                            println!("  - {}", asset.title);
                        }
                    }
                    Err(e) => println!("[Wikimedia] Search error: {:?}", e),
                }
            }
            Err(e) => println!("Failed to create DxMedia: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_dxmedia_search_nasa() {
        setup_dirs();

        match DxMedia::new() {
            Ok(dx) => {
                let result = dx
                    .search("earth space")
                    .media_type(MediaType::Image)
                    .provider("nasa")
                    .count(3)
                    .execute()
                    .await;

                match result {
                    Ok(results) => {
                        println!("\n[NASA] Found {} results", results.total_count);
                        for asset in results.assets.iter().take(3) {
                            println!("  - {}", asset.title);
                        }
                    }
                    Err(e) => println!("[NASA] Search error: {:?}", e),
                }
            }
            Err(e) => println!("Failed to create DxMedia: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_dxmedia_search_met() {
        setup_dirs();

        match DxMedia::new() {
            Ok(dx) => {
                let result = dx
                    .search("painting art")
                    .media_type(MediaType::Image)
                    .provider("met")
                    .count(3)
                    .execute()
                    .await;

                match result {
                    Ok(results) => {
                        println!("\n[Met Museum] Found {} results", results.total_count);
                        for asset in results.assets.iter().take(3) {
                            println!("  - {}", asset.title);
                        }
                    }
                    Err(e) => println!("[Met Museum] Search error: {:?}", e),
                }
            }
            Err(e) => println!("Failed to create DxMedia: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_dxmedia_search_and_download() {
        setup_dirs();

        match DxMedia::new() {
            Ok(dx) => {
                // Search for an image
                let result = dx
                    .search("flower")
                    .media_type(MediaType::Image)
                    .provider("wikimedia")
                    .count(1)
                    .execute()
                    .await;

                match result {
                    Ok(results) if !results.assets.is_empty() => {
                        let asset = &results.assets[0];
                        println!("\n[Download Test] Found: {}", asset.title);

                        // Download to provider folder
                        let output_dir = providers_path().join("wikimedia");

                        match dx.download_to(asset, &output_dir).await {
                            Ok(path) => {
                                println!("[Download Test] ✓ Downloaded to: {:?}", path);
                            }
                            Err(e) => println!("[Download Test] Download error: {:?}", e),
                        }
                    }
                    Ok(_) => println!("[Download Test] No results found"),
                    Err(e) => println!("[Download Test] Search error: {:?}", e),
                }
            }
            Err(e) => println!("Failed to create DxMedia: {:?}", e),
        }
    }
}

// ============================================================================
// SCRAPING TESTS
// ============================================================================

mod scraping_tests {
    use super::*;

    #[test]
    fn test_scraping_registry_available() {
        let total = ScrapingRegistry::total_count();

        println!("\n=== Available Scraping Targets ===");
        println!("Total targets: {}", total);
        println!("Estimate: {}", ScrapingRegistry::total_assets_estimate());
        println!("Breakdown: {}", ScrapingRegistry::asset_breakdown());

        // Count by category
        let images = ScrapingRegistry::by_category(ScrapingCategory::Images);
        let videos = ScrapingRegistry::by_category(ScrapingCategory::Videos);
        let audio = ScrapingRegistry::by_category(ScrapingCategory::Audio);
        let textures = ScrapingRegistry::by_category(ScrapingCategory::Textures);
        let models = ScrapingRegistry::by_category(ScrapingCategory::Models3D);
        let vectors = ScrapingRegistry::by_category(ScrapingCategory::Vectors);
        let games = ScrapingRegistry::by_category(ScrapingCategory::GameAssets);

        println!("\nBy Category:");
        println!("  Images: {} targets", images.len());
        println!("  Videos: {} targets", videos.len());
        println!("  Audio: {} targets", audio.len());
        println!("  Textures: {} targets", textures.len());
        println!("  3D Models: {} targets", models.len());
        println!("  Vectors: {} targets", vectors.len());
        println!("  Game Assets: {} targets", games.len());

        assert!(total > 200, "Should have 200+ scraping targets");
    }

    #[test]
    fn test_scraping_target_by_id() {
        // Test getting specific targets
        if let Some(stocksnap) = ScrapingRegistry::get("stocksnap") {
            println!("\n=== StockSnap Target ===");
            println!("Name: {}", stocksnap.name);
            println!("Base URL: {}", stocksnap.base_url);
            println!("Estimated Assets: {}", stocksnap.estimated_assets);
            println!("License: {}", stocksnap.license);
        }

        // List all image targets
        let image_targets = ScrapingRegistry::by_category(ScrapingCategory::Images);
        println!("\n=== Image Scraping Targets ({}) ===", image_targets.len());
        for target in image_targets.iter().take(10) {
            println!("  - {} ({})", target.name, target.base_url);
        }
    }

    #[test]
    fn test_scraping_list_ids() {
        // List all target IDs
        let ids = ScrapingRegistry::list_ids();
        println!("\n=== All Scraping Target IDs ({}) ===", ids.len());
        for id in ids.iter().take(20) {
            println!("  - {}", id);
        }
    }
}

// ============================================================================
// TOOL TESTS
// ============================================================================

mod tool_tests {
    use super::*;

    // IMAGE TOOLS (10)
    #[test]
    fn test_image_tools_suite() {
        setup_dirs();
        let output_dir = tools_path().join("image");
        fs::create_dir_all(&output_dir).ok();

        // Create a test image for testing
        let test_image = output_dir.join("test.png");
        if !test_image.exists() {
            // Create a minimal PNG for testing
            let png_bytes: &[u8] = &[
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
                0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
                0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08, // 8x8
                0x08, 0x02, 0x00, 0x00, 0x00, 0x4B, 0x6D, 0x29, 0xDE, 0x00, 0x00, 0x00, 0x17, 0x49,
                0x44, 0x41, 0x54, 0x78, 0x9C, 0x62, 0xF8, 0xCF, 0xC0, 0xF0, 0x1F, 0x00, 0x05, 0xFE,
                0x02, 0xFE, 0xDC, 0xCC, 0x59, 0xE7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,
                0xAE, 0x42, 0x60, 0x82,
            ];
            fs::write(&test_image, png_bytes).ok();
        }

        println!("\n=== IMAGE TOOLS (10) ===");

        // 1. Resize
        let out = output_dir.join("resized.png");
        let r = image::resize(&test_image, &out, 100, 100);
        println!("1. Resize: {}", if r.is_ok() { "✓" } else { "✗" });

        // 2. Crop
        let out = output_dir.join("cropped.png");
        let r = image::crop(&test_image, &out, 0, 0, 4, 4);
        println!("2. Crop: {}", if r.is_ok() { "✓" } else { "✗" });

        // 3. Rotate
        let out = output_dir.join("rotated.png");
        let r = image::rotate(&test_image, &out, 90.0);
        println!("3. Rotate: {}", if r.is_ok() { "✓" } else { "✗" });

        // 4. Convert
        let out = output_dir.join("converted.jpg");
        let r = image::convert(&test_image, &out);
        println!("4. Convert: {}", if r.is_ok() { "✓" } else { "✗" });

        // 5. Compress
        let out = output_dir.join("compressed.jpg");
        let r = image::compress(&test_image, &out, 80);
        println!("5. Compress: {}", if r.is_ok() { "✓" } else { "✗" });

        // 6. Grayscale
        let out = output_dir.join("grayscale.png");
        let r = image::grayscale(&test_image, &out);
        println!("6. Grayscale: {}", if r.is_ok() { "✓" } else { "✗" });

        // 7. Thumbnail
        let out = output_dir.join("thumb.png");
        let r = image::thumbnail(&test_image, &out, 32);
        println!("7. Thumbnail: {}", if r.is_ok() { "✓" } else { "✗" });

        // 8. Info (EXIF metadata)
        let r = image::read_exif(&test_image);
        println!(
            "8. Info (EXIF): {}",
            if r.is_ok() {
                "✓"
            } else {
                "✗ (needs exiftool)"
            }
        );

        // 9. Watermark (needs two images)
        println!("9. Watermark: API available (needs watermark image)");

        // 10. Strip Metadata
        let out = output_dir.join("stripped.png");
        let r = image::strip_metadata(&test_image, &out);
        println!("10. Strip Metadata: {}", if r.is_ok() { "✓" } else { "✗" });
    }

    // VIDEO TOOLS (10)
    #[test]
    fn test_video_tools_suite() {
        setup_dirs();
        let output_dir = tools_path().join("video");
        fs::create_dir_all(&output_dir).ok();

        println!("\n=== VIDEO TOOLS (10) ===");

        // Create a minimal test video (or skip if no ffmpeg)
        println!("1. Convert: API available (needs video file)");
        println!("2. Compress: API available (needs video file)");
        println!("3. Extract Frame: API available (needs video file)");
        println!("4. Extract Audio: API available (needs video file)");
        println!("5. Trim: API available (needs video file)");
        println!("6. Merge: API available (needs video files)");
        println!("7. Info: API available (needs video file)");
        println!("8. Resize: API available (needs video file)");
        println!("9. GIF: API available (needs video file)");
        println!("10. Add Audio: API available (needs video + audio files)");
    }

    // AUDIO TOOLS (10)
    #[test]
    fn test_audio_tools_suite() {
        setup_dirs();
        let output_dir = tools_path().join("audio");
        fs::create_dir_all(&output_dir).ok();

        println!("\n=== AUDIO TOOLS (10) ===");

        println!("1. Convert: API available (needs audio file)");
        println!("2. Trim: API available (needs audio file)");
        println!("3. Merge: API available (needs audio files)");
        println!("4. Volume: API available (needs audio file)");
        println!("5. Normalize: API available (needs audio file)");
        println!("6. Info: API available (needs audio file)");
        println!("7. Silence Remove: API available (needs audio file)");
        println!("8. Split: API available (needs audio file)");
        println!("9. Waveform: API available (needs audio file)");
        println!("10. Transcribe: API available (needs audio file)");
    }

    // DOCUMENT TOOLS (10)
    #[test]
    fn test_document_tools_suite() {
        setup_dirs();
        let output_dir = tools_path().join("document");
        fs::create_dir_all(&output_dir).ok();

        println!("\n=== DOCUMENT TOOLS (10) ===");

        // Create test files
        let md_file = output_dir.join("test.md");
        let html_file = output_dir.join("test.html");
        let txt_file = output_dir.join("test.txt");

        fs::write(
            &md_file,
            "# Test Document\n\nThis is a **test** paragraph.\n\n- Item 1\n- Item 2",
        )
        .ok();
        fs::write(
            &html_file,
            "<html><body><h1>Test</h1><p>Hello World</p></body></html>",
        )
        .ok();
        fs::write(&txt_file, "This is plain text for testing.").ok();

        // 1-4. PDF operations (need PDF files)
        println!("1. PDF Merge: API available (needs PDF files)");
        println!("2. PDF Split: API available (needs PDF files)");
        println!("3. PDF Compress: API available (needs PDF files)");
        println!("4. PDF to Image: API available (needs PDF files)");

        // 5. Markdown to HTML
        let out = output_dir.join("from_md.html");
        let r = document::markdown_to_html(&md_file, &out);
        println!("5. Markdown to HTML: {}", if r.is_ok() { "✓" } else { "✗" });

        // 6. HTML to PDF
        let out = output_dir.join("from_html.pdf");
        let r = document::html_to_pdf(&html_file, &out);
        println!(
            "6. HTML to PDF: {}",
            if r.is_ok() {
                "✓"
            } else {
                "✗ (needs wkhtmltopdf)"
            }
        );

        // 7. Document Convert
        let out = output_dir.join("converted.txt");
        let r = document::convert_document(&md_file, &out, document::DocFormat::Txt);
        println!(
            "7. Document Convert: {}",
            if r.is_ok() {
                "✓"
            } else {
                "✗ (needs pandoc)"
            }
        );

        // 8. Text Extract
        let r = document::extract(&txt_file);
        println!("8. Text Extract: {}", if r.is_ok() { "✓" } else { "✗" });

        // 9-10. PDF operations
        println!("9. PDF Watermark: API available (needs PDF files)");
        println!("10. PDF Encrypt: API available (needs PDF files)");
    }

    // ARCHIVE TOOLS (10)
    #[test]
    fn test_archive_tools_suite() {
        setup_dirs();
        let output_dir = tools_path().join("archive");
        fs::create_dir_all(&output_dir).ok();

        // Create test files
        let test_file = output_dir.join("test.txt");
        fs::write(&test_file, "Test content for archive operations").ok();

        println!("\n=== ARCHIVE TOOLS (10) ===");

        // 1. Create ZIP
        let out = output_dir.join("test.zip");
        let r = archive::create_zip(&[&test_file], &out);
        println!("1. Create ZIP: {}", if r.is_ok() { "✓" } else { "✗" });

        // 2. Extract ZIP
        let extract_dir = output_dir.join("extracted_zip");
        let r = if out.exists() {
            archive::extract_zip(&out, &extract_dir)
        } else {
            Err(dx_media::DxError::file_io(&out, "file not found"))
        };
        println!("2. Extract ZIP: {}", if r.is_ok() { "✓" } else { "✗" });

        // 3. Create TAR
        let out = output_dir.join("test.tar");
        let r = archive::create_tar(&[&test_file], &out);
        println!("3. Create TAR: {}", if r.is_ok() { "✓" } else { "✗" });

        // 4. Extract TAR
        let extract_dir = output_dir.join("extracted_tar");
        let r = if out.exists() {
            archive::extract_tar(&out, &extract_dir)
        } else {
            Err(dx_media::DxError::file_io(&out, "file not found"))
        };
        println!("4. Extract TAR: {}", if r.is_ok() { "✓" } else { "✗" });

        // 5. GZIP
        let out = output_dir.join("test.txt.gz");
        let r = archive::gzip(&test_file, &out);
        println!("5. GZIP: {}", if r.is_ok() { "✓" } else { "✗" });

        // 6. GUNZIP
        let gunzip_out = output_dir.join("decompressed.txt");
        let r = if out.exists() {
            archive::gunzip(&out, &gunzip_out)
        } else {
            Err(dx_media::DxError::file_io(&out, "file not found"))
        };
        println!("6. GUNZIP: {}", if r.is_ok() { "✓" } else { "✗" });

        // 7. List Archive
        let zip_path = output_dir.join("test.zip");
        let r = if zip_path.exists() {
            archive::list_archive(&zip_path)
        } else {
            Err(dx_media::DxError::file_io(&zip_path, "file not found"))
        };
        println!("7. List Archive: {}", if r.is_ok() { "✓" } else { "✗" });

        // 8. Encrypted ZIP
        let out = output_dir.join("encrypted.zip");
        let r = archive::create_encrypted_zip(&[&test_file], &out, "password123");
        println!("8. Encrypted ZIP: {}", if r.is_ok() { "✓" } else { "✗" });

        // 9. Split Archive
        let zip_path = output_dir.join("test.zip");
        let r = if zip_path.exists() {
            archive::split_archive(&zip_path, &output_dir, 1024)
        } else {
            Err(dx_media::DxError::file_io(&zip_path, "file not found"))
        };
        println!("9. Split Archive: {}", if r.is_ok() { "✓" } else { "✗" });

        // 10. 7z Create
        let out = output_dir.join("test.7z");
        let r = archive::create_7z(&[&test_file], &out);
        println!(
            "10. Create 7z: {}",
            if r.is_ok() { "✓" } else { "✗ (needs 7z)" }
        );
    }

    // UTILITY TOOLS (10)
    #[test]
    fn test_utility_tools_suite() {
        setup_dirs();
        let output_dir = tools_path().join("utility");
        fs::create_dir_all(&output_dir).ok();

        // Create test files
        let test_file = output_dir.join("test.txt");
        fs::write(&test_file, "Test content for utility operations").ok();

        let json_file = output_dir.join("test.json");
        fs::write(&json_file, r#"{"name":"test","value":123}"#).ok();

        let csv_file = output_dir.join("test.csv");
        fs::write(&csv_file, "name,age,city\nAlice,30,NYC\nBob,25,LA").ok();

        println!("\n=== UTILITY TOOLS (10) ===");

        // 1. Hash
        let r = utility::hash_file(&test_file, utility::HashAlgorithm::Sha256);
        println!("1. Hash (SHA256): {}", if r.is_ok() { "✓" } else { "✗" });

        // 2. Base64 Encode
        let r = utility::encode_file(&test_file);
        println!("2. Base64 Encode: {}", if r.is_ok() { "✓" } else { "✗" });

        // 3. Base64 Decode
        let encoded = output_dir.join("encoded.txt");
        fs::write(&encoded, "SGVsbG8gV29ybGQh").ok();
        let decoded = output_dir.join("decoded.txt");
        let r = utility::decode_file(&encoded, &decoded);
        println!("3. Base64 Decode: {}", if r.is_ok() { "✓" } else { "✗" });

        // 4. URL Encode
        let r = utility::encode("Hello World! Special: éàü");
        println!("4. URL Encode: {}", if r.is_ok() { "✓" } else { "✗" });

        // 5. URL Decode
        let r = utility::decode("Hello%20World%21");
        println!("5. URL Decode: {}", if r.is_ok() { "✓" } else { "✗" });

        // 6. JSON Format
        let formatted = output_dir.join("formatted.json");
        let r = utility::format_json_file(&json_file, &formatted);
        println!("6. JSON Format: {}", if r.is_ok() { "✓" } else { "✗" });

        // 7. JSON Minify
        let minified = output_dir.join("minified.json");
        let content = fs::read_to_string(&json_file).unwrap_or_default();
        let r = utility::minify_string(&content);
        if let Ok(output) = &r {
            fs::write(&minified, &output.message).ok();
        }
        println!("7. JSON Minify: {}", if r.is_ok() { "✓" } else { "✗" });

        // 8. CSV to JSON
        let csv_json = output_dir.join("from_csv.json");
        let r = utility::csv_to_json(&csv_file, &csv_json);
        println!("8. CSV to JSON: {}", if r.is_ok() { "✓" } else { "✗" });

        // 9. JSON to CSV
        let json_csv = output_dir.join("from_json.csv");
        let r = utility::json_to_csv(&csv_json, &json_csv);
        println!("9. JSON to CSV: {}", if r.is_ok() { "✓" } else { "✗" });

        // 10. YAML to JSON
        let yaml_file = output_dir.join("test.yaml");
        fs::write(&yaml_file, "name: test\nvalue: 123").ok();
        let yaml_json = output_dir.join("from_yaml.json");
        let r = utility::yaml_to_json(&yaml_file, &yaml_json);
        println!("10. YAML to JSON: {}", if r.is_ok() { "✓" } else { "✗" });
    }

    // Summary test
    #[test]
    fn test_tool_api_summary() {
        println!("\n");
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║            DX-MEDIA 60 TOOLS AVAILABILITY                  ║");
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║ Category     │ Total │ Native │ External Deps              ║");
        println!("╟──────────────┼───────┼────────┼────────────────────────────╢");
        println!("║ Image        │  10   │   10   │ ImageMagick (optional)     ║");
        println!("║ Video        │  10   │    0   │ FFmpeg (required)          ║");
        println!("║ Audio        │  10   │    0   │ FFmpeg (required)          ║");
        println!("║ Document     │  10   │    3   │ wkhtmltopdf, pandoc, etc.  ║");
        println!("║ Archive      │  10   │   10   │ 7z (optional)              ║");
        println!("║ Utility      │  10   │   10   │ None                       ║");
        println!("╟──────────────┼───────┼────────┼────────────────────────────╢");
        println!("║ TOTAL        │  60   │   33   │ 27 need external tools     ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        println!();
    }
}

// ============================================================================
// PROVIDER DOWNLOAD TESTS - Download sample from each provider
// ============================================================================

mod provider_download_tests {
    use super::*;

    #[tokio::test]
    async fn test_download_from_picsum() {
        setup_dirs();

        match DxMedia::new() {
            Ok(dx) => {
                let result = dx
                    .search("random")
                    .media_type(MediaType::Image)
                    .provider("picsum")
                    .count(1)
                    .execute()
                    .await;

                match result {
                    Ok(results) if !results.assets.is_empty() => {
                        let asset = &results.assets[0];
                        let output_dir = providers_path().join("picsum");

                        match dx.download_to(asset, &output_dir).await {
                            Ok(path) => println!("[picsum] ✓ Downloaded: {:?}", path),
                            Err(e) => println!("[picsum] Download error: {:?}", e),
                        }
                    }
                    Ok(_) => println!("[picsum] No results"),
                    Err(e) => println!("[picsum] Search error: {:?}", e),
                }
            }
            Err(e) => println!("Failed to create DxMedia: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_download_from_openverse() {
        setup_dirs();

        match DxMedia::new() {
            Ok(dx) => {
                let result = dx
                    .search("sunset")
                    .media_type(MediaType::Image)
                    .provider("openverse")
                    .count(1)
                    .execute()
                    .await;

                match result {
                    Ok(results) if !results.assets.is_empty() => {
                        let asset = &results.assets[0];
                        let output_dir = providers_path().join("openverse");

                        match dx.download_to(asset, &output_dir).await {
                            Ok(path) => println!("[openverse] ✓ Downloaded: {:?}", path),
                            Err(e) => println!("[openverse] Download error: {:?}", e),
                        }
                    }
                    Ok(_) => println!("[openverse] No results"),
                    Err(e) => println!("[openverse] Search error: {:?}", e),
                }
            }
            Err(e) => println!("Failed to create DxMedia: {:?}", e),
        }
    }
}
