# Error Recovery Guide

This guide explains how to handle errors gracefully in dx-font and implement robust error recovery strategies.

## Error Types

dx-font uses a comprehensive error hierarchy via the `FontError` enum:

```rust
pub enum FontError {
    Network { url: String, source: reqwest::Error },
    Provider { provider: String, message: String, source: Option<Box<dyn Error>> },
    Parse { provider: String, message: String },
    Download { font_id: String, message: String },
    Cache { message: String, source: Option<std::io::Error> },
    RateLimit { provider: String, retry_after_secs: u64 },
    Validation { message: String },
    AllProvidersFailed { errors: Vec<(String, FontError)> },
    Timeout { timeout_secs: u64 },
    Verification { message: String },
}
```

## Recovery Patterns

### 1. Graceful Degradation

Accept partial results when some providers fail:

```rust
use dx_font::prelude::*;

let search = FontSearch::new()?;

match search.search("roboto").await {
    Ok(results) => {
        println!("Found {} fonts", results.total);
        
        // Check for provider errors
        if !results.provider_errors.is_empty() {
            eprintln!("Some providers had issues:");
            for error in &results.provider_errors {
                eprintln!("  - {}: {}", error.provider, error.message);
            }
        }
        
        // Use available results
        for font in results.fonts {
            println!("  - {}", font.name);
        }
    }
    Err(FontError::AllProvidersFailed { errors }) => {
        eprintln!("All providers failed:");
        for (provider, error) in errors {
            eprintln!("  - {}: {}", provider, error);
        }
        // Fallback: show cached results or error page
    }
    Err(e) => {
        eprintln!("Search error: {}", e);
    }
}
```

### 2. Retry with Exponential Backoff

Implement custom retry logic for transient failures:

```rust
use dx_font::prelude::*;
use std::time::Duration;

async fn search_with_retry(query: &str, max_retries: u32) -> FontResult<SearchResults> {
    let mut attempt = 0;
    
    loop {
        attempt += 1;
        
        let search = FontSearch::new()?;
        match search.search(query).await {
            Ok(results) => return Ok(results),
            Err(e) if e.is_retryable() && attempt < max_retries => {
                let delay = Duration::from_secs(2u64.pow(attempt - 1));
                eprintln!("Attempt {} failed: {}. Retrying in {:?}", attempt, e, delay);
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 3. Provider Fallback

Try alternative providers if the primary fails:

```rust
use dx_font::prelude::*;

let search = FontSearch::new()?;
let results = search.search("roboto").await?;

// Preferred provider order
let preferred = vec![
    FontProvider::GoogleFonts,
    FontProvider::BunnyFonts,
    FontProvider::Fontsource,
];

// Group results by provider
let mut by_provider: std::collections::HashMap<FontProvider, Vec<&Font>> = 
    std::collections::HashMap::new();

for font in &results.fonts {
    by_provider.entry(font.provider.clone()).or_default().push(font);
}

// Use first available from preferred list
for provider in preferred {
    if let Some(fonts) = by_provider.get(&provider) {
        println!("Using {} ({} fonts)", provider.name(), fonts.len());
        // Use these fonts
        break;
    }
}
```

### 4. Timeout Handling

Set reasonable timeouts and handle them gracefully:

```rust
use dx_font::prelude::*;
use std::time::Duration;

let search = FontSearch::new()?;

match tokio::time::timeout(
    Duration::from_secs(10),
    search.search("roboto")
).await {
    Ok(Ok(results)) => {
        println!("Found {} fonts", results.total);
    }
    Ok(Err(e)) => {
        eprintln!("Search error: {}", e);
        // Show cached results or error message
    }
    Err(_) => {
        eprintln!("Search timed out");
        // Cancel operation and show cached results
    }
}
```

### 5. Cache-First Strategy

Prioritize cached data for reliability:

```rust
use dx_font::prelude::*;

let search = FontSearch::new()?;

// Try to get results (may use cache)
match search.search("roboto").await {
    Ok(results) => {
        if results.from_cache {
            println!("Using cached results (may be stale)");
        }
        // Use results
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        // Try to load from persistent cache or show error
    }
}
```

## Error Context

All errors include rich context for debugging:

```rust
use dx_font::prelude::*;

match search.search("test").await {
    Err(FontError::Network { url, source }) => {
        eprintln!("Network error for {}: {}", url, source);
    }
    Err(FontError::Provider { provider, message, .. }) => {
        eprintln!("Provider {} failed: {}", provider, message);
    }
    Err(FontError::RateLimit { provider, retry_after_secs }) => {
        eprintln!("Rate limited by {}, retry after {}s", provider, retry_after_secs);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        // Print full error chain
        eprintln!("Chain: {}", e.error_chain());
    }
    Ok(_) => {}
}
```

## Retryable Errors

Use `is_retryable()` to determine if an error should be retried:

```rust
use dx_font::prelude::*;

let error = FontError::timeout(30);
if error.is_retryable() {
    // Retry the operation
}

// Retryable errors:
// - Network errors
// - Rate limit errors
// - Timeout errors

// Non-retryable errors:
// - Validation errors
// - Parse errors
// - Verification errors
```

## Best Practices

1. **Always check `provider_errors`** in search results to detect partial failures
2. **Use `is_retryable()`** to implement smart retry logic
3. **Set reasonable timeouts** to prevent hanging operations
4. **Log errors with full context** using `error_chain()` for debugging
5. **Implement fallback strategies** for critical operations
6. **Cache results** to improve reliability and performance
7. **Handle `AllProvidersFailed`** separately from other errors
8. **Use provider fallback** when specific providers are unreliable

## Example: Production-Ready Error Handling

```rust
use dx_font::prelude::*;
use std::time::Duration;

pub async fn robust_font_search(query: &str) -> FontResult<Vec<Font>> {
    let max_retries = 3;
    let timeout = Duration::from_secs(10);
    
    for attempt in 1..=max_retries {
        let search = FontSearch::new()?;
        
        let result = tokio::time::timeout(
            timeout,
            search.search(query)
        ).await;
        
        match result {
            Ok(Ok(results)) => {
                // Log any provider errors
                for error in &results.provider_errors {
                    tracing::warn!(
                        provider = %error.provider,
                        error = %error.message,
                        "Provider failed"
                    );
                }
                
                if results.total > 0 {
                    return Ok(results.fonts);
                }
                
                // No results but no error - might be legitimate
                return Ok(vec![]);
            }
            Ok(Err(e)) if e.is_retryable() && attempt < max_retries => {
                let delay = Duration::from_secs(2u64.pow(attempt - 1));
                tracing::warn!(
                    attempt,
                    error = %e,
                    retry_in = ?delay,
                    "Retrying after error"
                );
                tokio::time::sleep(delay).await;
                continue;
            }
            Ok(Err(e)) => {
                tracing::error!(error = %e, "Search failed");
                return Err(e);
            }
            Err(_) => {
                tracing::error!(attempt, "Search timed out");
                if attempt < max_retries {
                    continue;
                }
                return Err(FontError::timeout(timeout.as_secs()));
            }
        }
    }
    
    Err(FontError::validation("Max retries exceeded"))
}
```

## See Also

- [examples/error_recovery.rs](../examples/error_recovery.rs) - Complete error recovery examples
- [examples/advanced_usage.rs](../examples/advanced_usage.rs) - Advanced usage patterns
- [API Documentation](https://docs.rs/dx-font) - Full API reference
