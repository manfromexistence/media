//! Error Recovery Patterns for dx-font
//!
//! This example demonstrates how to handle various error scenarios
//! and implement robust error recovery strategies.

use dx_font::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> FontResult<()> {
    println!("=== Error Recovery Patterns ===\n");

    // Pattern 1: Graceful degradation with partial results
    graceful_search_degradation().await?;

    // Pattern 2: Retry with exponential backoff
    retry_with_backoff().await?;

    // Pattern 3: Fallback to alternative providers
    provider_fallback().await?;

    // Pattern 4: Timeout handling
    timeout_handling().await?;

    // Pattern 5: Cache-first strategy
    cache_first_strategy().await?;

    Ok(())
}

/// Pattern 1: Accept partial results when some providers fail
async fn graceful_search_degradation() -> FontResult<()> {
    println!("1. Graceful Degradation with Partial Results");
    println!("   Accepting results even if some providers fail\n");

    let search = FontSearch::new()?;

    match search.search("roboto").await {
        Ok(results) => {
            println!("   ✓ Found {} fonts", results.total);

            if !results.provider_errors.is_empty() {
                println!("   ⚠ Some providers had issues:");
                for error in &results.provider_errors {
                    println!("     - {}: {}", error.provider, error.message);
                }
            }

            println!("   → Strategy: Use available results, log errors\n");
        }
        Err(FontError::AllProvidersFailed { errors }) => {
            println!("   ✗ All providers failed:");
            for (provider, error) in errors {
                println!("     - {}: {}", provider, error);
            }
            println!("   → Strategy: Show cached results or error message\n");
        }
        Err(e) => {
            println!("   ✗ Unexpected error: {}", e);
            println!("   → Strategy: Log and show user-friendly message\n");
        }
    }

    Ok(())
}

/// Pattern 2: Implement custom retry logic with exponential backoff
async fn retry_with_backoff() -> FontResult<()> {
    println!("2. Retry with Exponential Backoff");
    println!("   Retrying failed operations with increasing delays\n");

    let max_retries = 3;
    let mut attempt = 0;

    loop {
        attempt += 1;
        println!("   Attempt {}/{}", attempt, max_retries);

        let search = FontSearch::new()?;
        match search.search("test").await {
            Ok(results) => {
                println!("   ✓ Success: {} fonts found\n", results.total);
                break;
            }
            Err(e) if e.is_retryable() && attempt < max_retries => {
                let delay = Duration::from_secs(2u64.pow(attempt - 1));
                println!("   ⚠ Retryable error: {}", e);
                println!("   → Waiting {:?} before retry\n", delay);
                tokio::time::sleep(delay).await;
            }
            Err(e) => {
                println!("   ✗ Non-retryable or max retries: {}\n", e);
                break;
            }
        }
    }

    Ok(())
}

/// Pattern 3: Fallback to alternative providers
async fn provider_fallback() -> FontResult<()> {
    println!("3. Provider Fallback Strategy");
    println!("   Trying alternative providers if primary fails\n");

    let search = FontSearch::new()?;
    let results = search.search("roboto").await?;

    // Group results by provider
    let mut by_provider: std::collections::HashMap<FontProvider, Vec<&Font>> =
        std::collections::HashMap::new();

    for font in &results.fonts {
        by_provider
            .entry(font.provider.clone())
            .or_default()
            .push(font);
    }

    // Preferred provider order
    let preferred = vec![
        FontProvider::GoogleFonts,
        FontProvider::BunnyFonts,
        FontProvider::Fontsource,
    ];

    for provider in preferred {
        if let Some(fonts) = by_provider.get(&provider) {
            println!("   ✓ Using {} ({} fonts)", provider.name(), fonts.len());
            println!("   → Strategy: Use first available from preferred list\n");
            break;
        }
    }

    Ok(())
}

/// Pattern 4: Handle timeouts gracefully
async fn timeout_handling() -> FontResult<()> {
    println!("4. Timeout Handling");
    println!("   Setting reasonable timeouts and handling them\n");

    let _config = Config::builder().timeout_seconds(5).build()?;

    println!("   Configured timeout: 5 seconds");

    let search = FontSearch::new()?;

    match tokio::time::timeout(Duration::from_secs(10), search.search("test")).await {
        Ok(Ok(results)) => {
            println!("   ✓ Completed within timeout: {} fonts\n", results.total);
        }
        Ok(Err(e)) => {
            println!("   ✗ Search error: {}", e);
            println!("   → Strategy: Show cached results or partial data\n");
        }
        Err(_) => {
            println!("   ✗ Operation timed out");
            println!("   → Strategy: Cancel and show cached results\n");
        }
    }

    Ok(())
}

/// Pattern 5: Cache-first strategy for reliability
async fn cache_first_strategy() -> FontResult<()> {
    println!("5. Cache-First Strategy");
    println!("   Prioritizing cached data for reliability\n");

    let search = FontSearch::new()?;

    // First request - may be slow
    println!("   First request (may hit network)...");
    let start = std::time::Instant::now();
    let _results1 = search.search("roboto").await?;
    let duration1 = start.elapsed();
    println!("   ✓ Completed in {:?}", duration1);

    // Second request - should be fast from cache
    println!("   Second request (should hit cache)...");
    let start = std::time::Instant::now();
    let _results2 = search.search("roboto").await?;
    let duration2 = start.elapsed();
    println!("   ✓ Completed in {:?}", duration2);

    if duration2 < duration1 / 2 {
        println!(
            "   → Cache is working! {}x faster\n",
            duration1.as_millis() / duration2.as_millis().max(1)
        );
    }

    Ok(())
}
