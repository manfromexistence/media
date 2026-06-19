//! Integration tests for Google Fonts provider
//!
//! These tests make real API calls to Google Fonts to verify the provider
//! implementation works correctly with the actual API.

use dx_font::prelude::*;
use std::time::Duration;

#[tokio::test]
#[ignore] // Run with: cargo test --test integration_google_fonts -- --ignored
async fn test_google_fonts_search() {
    let search = FontSearch::new().expect("Failed to create search");

    let results = search.search("roboto").await.expect("Search failed");

    assert!(results.total > 0, "Should find at least one Roboto font");
    assert!(
        results
            .fonts
            .iter()
            .any(|f| f.provider == FontProvider::GoogleFonts),
        "Should have results from Google Fonts"
    );
}

#[tokio::test]
#[ignore]
async fn test_google_fonts_download() {
    let downloader = FontDownloader::new().expect("Failed to create downloader");
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    let result = downloader
        .download_google_font("roboto", temp_dir.path(), &["woff2"], &["latin"])
        .await;

    assert!(result.is_ok(), "Download should succeed");
    let path = result.unwrap();
    assert!(path.exists(), "Downloaded file should exist");
}

#[tokio::test]
#[ignore]
async fn test_parallel_provider_search() {
    let search = FontSearch::new().expect("Failed to create search");

    let start = std::time::Instant::now();
    let results = search.search("open sans").await.expect("Search failed");
    let duration = start.elapsed();

    assert!(results.total > 0, "Should find fonts");
    assert!(
        duration < Duration::from_secs(10),
        "Parallel search should be fast"
    );

    // Should have results from multiple providers
    let providers: std::collections::HashSet<_> =
        results.fonts.iter().map(|f| &f.provider).collect();
    assert!(providers.len() > 1, "Should query multiple providers");
}

#[tokio::test]
#[ignore]
async fn test_rate_limiting() {
    let search = FontSearch::new().expect("Failed to create search");

    // Make multiple rapid requests
    for _ in 0..5 {
        let result = search.search("test").await;
        assert!(result.is_ok(), "Rate limiting should not cause failures");
    }
}

#[tokio::test]
#[ignore]
async fn test_cache_functionality() {
    let search = FontSearch::new().expect("Failed to create search");

    // First search - should hit API
    let start1 = std::time::Instant::now();
    let results1 = search.search("roboto").await.expect("First search failed");
    let duration1 = start1.elapsed();

    // Second search - should hit cache
    let start2 = std::time::Instant::now();
    let results2 = search.search("roboto").await.expect("Second search failed");
    let duration2 = start2.elapsed();

    assert_eq!(
        results1.total, results2.total,
        "Results should be identical"
    );
    assert!(duration2 < duration1, "Cached search should be faster");
}
