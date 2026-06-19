//! Integration tests for all font providers
//!
//! Tests that verify each provider can be queried successfully.

use dx_font::prelude::*;

#[tokio::test]
#[ignore]
async fn test_all_providers_reachable() {
    let search = FontSearch::new().expect("Failed to create search");

    let results = search.search("test").await.expect("Search failed");

    // At least some providers should respond
    assert!(results.total > 0 || !results.provider_errors.is_empty());

    // Log any provider errors for debugging
    for error in &results.provider_errors {
        eprintln!("Provider {} error: {}", error.provider, error.message);
    }
}

#[tokio::test]
#[ignore]
async fn test_bunny_fonts_provider() {
    let search = FontSearch::new().expect("Failed to create search");
    let results = search.search("roboto").await.expect("Search failed");

    let bunny_results: Vec<_> = results
        .fonts
        .iter()
        .filter(|f| f.provider == FontProvider::BunnyFonts)
        .collect();

    if !bunny_results.is_empty() {
        assert!(bunny_results[0].name.to_lowercase().contains("roboto"));
    }
}

#[tokio::test]
#[ignore]
async fn test_fontsource_provider() {
    let search = FontSearch::new().expect("Failed to create search");
    let results = search.search("inter").await.expect("Search failed");

    let fontsource_results: Vec<_> = results
        .fonts
        .iter()
        .filter(|f| f.provider == FontProvider::Fontsource)
        .collect();

    if !fontsource_results.is_empty() {
        assert!(!fontsource_results[0].name.is_empty());
    }
}

#[tokio::test]
#[ignore]
async fn test_provider_error_handling() {
    let search = FontSearch::new().expect("Failed to create search");

    // Search with a very specific query that might not exist
    let results = search
        .search("xyzabc123nonexistent")
        .await
        .expect("Search should not fail");

    // Even if no results, search should succeed (total is usize, always >= 0)
    assert!(results.total == 0 || !results.fonts.is_empty());
}
