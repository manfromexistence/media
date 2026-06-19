# dx-font

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

A production-ready font search and download library with access to 5,000+ commercial-free fonts from 10 providers.

## Overview

`dx-font` provides a unified, async interface for searching and downloading fonts from multiple free font providers. Built with Rust for maximum performance and reliability.

## Features

- **10 Font Providers**: Google Fonts, Bunny Fonts, Fontsource, Font Library, Font Squirrel, DaFont, 1001 Fonts, FontSpace, Abstract Fonts, Urban Fonts
- **5,000+ Fonts**: Access thousands of commercial-free fonts
- **Parallel Search**: Concurrent search across all providers (514-1,010ms for 10 providers)
- **Auto-Unzip**: Automatically extracts ZIP archives and cleans up
- **Progress Bars**: Real-time download progress with ETA
- **Multiple Formats**: TTF, OTF, WOFF, WOFF2, ZIP
- **Smart Caching**: Response caching with configurable TTL
- **Rate Limiting**: Token bucket algorithm prevents API abuse
- **Retry Logic**: Exponential backoff with jitter for transient failures
- **File Verification**: Magic byte validation for all downloads
- **Graceful Degradation**: Continues working even if some providers fail
- **Zero Unsafe Code**: 100% safe Rust
- **Production Ready**: 110 unit tests, 9 integration tests, 0 clippy warnings

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dx-font = "0.1"
tokio = { version = "1.35", features = ["full"] }
```

## Quick Start

### Search for Fonts

```rust
use dx_font::prelude::*;

#[tokio::main]
async fn main() -> FontResult<()> {
    let search = FontSearch::new()?;
    
    // Search across all providers
    let results = search.search("roboto").await?;
    
    println!("Found {} fonts from {} providers", 
        results.total, 
        results.providers_searched.len()
    );
    
    for font in results.fonts.iter().take(5) {
        println!("  {} ({}) - {} variants",
            font.name,
            font.provider.name(),
            font.variant_count
        );
    }
    
    Ok(())
}
```

### Download Fonts (with Auto-Unzip)

```rust
use dx_font::prelude::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> FontResult<()> {
    let downloader = FontDownloader::new()?;
    
    // Download Google Font - automatically extracts ZIP
    let path = downloader
        .download_google_font(
            "roboto",
            &PathBuf::from("./fonts"),
            &["woff2", "ttf"],
            &["latin"],
        )
        .await?;
    
    println!("Downloaded and extracted to: {}", path.display());
    // ZIP file automatically removed after extraction
    
    Ok(())
}
```

### Download from Fontsource CDN

```rust
use dx_font::prelude::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> FontResult<()> {
    let downloader = FontDownloader::new()?;
    
    // Download specific weight and style
    let path = downloader
        .download_fontsource_font(
            "inter",
            &PathBuf::from("./fonts"),
            400,  // weight
            "normal",  // style
        )
        .await?;
    
    println!("Downloaded: {}", path.display());
    Ok(())
}
```

### Error Handling with Graceful Degradation

```rust
use dx_font::prelude::*;

#[tokio::main]
async fn main() {
    let search = FontSearch::new().expect("Failed to create search");
    
    match search.search("roboto").await {
        Ok(results) => {
            println!("Found {} fonts", results.total);
            
            // Check if any providers failed (graceful degradation)
            if !results.provider_errors.is_empty() {
                println!("Some providers had issues:");
                for err in &results.provider_errors {
                    println!("  {} - {}", err.provider, err.message);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Custom Configuration

```rust
use dx_font::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

let config = Config::builder()
    .output_dir(PathBuf::from("./fonts"))
    .timeout(Duration::from_secs(60))
    .max_retries(5)
    .cache_ttl(Duration::from_secs(7200)) // 2 hours
    .rate_limit_per_second(5.0)
    .rate_limit_burst(10)
    .concurrent_downloads(3)
    .build()?;
```

## CLI Usage

```bash
# Search for fonts
dx-font search "Roboto"

# Search with category filter
dx-font search "mono" --category monospace

# Download a font (auto-extracts if ZIP)
dx-font download "roboto" --output ./fonts

# Download with specific formats
dx-font download "roboto" --output ./fonts --formats woff2,ttf

# Get font details
dx-font info "Open Sans"

# List all available fonts
dx-font list --limit 100

# Check provider health
dx-font health
```

## Supported Providers

| Provider | Fonts | API | Rate Limit | Status |
|----------|-------|-----|------------|--------|
| Google Fonts | 1,500+ | Yes | 50/hour | ✅ Online |
| Bunny Fonts | 1,500+ | Yes | 100/hour | ✅ Online |
| Fontsource | 1,500+ | Yes | 200/hour | ⚠️ Offline |
| Font Library | 1,000+ | Yes | 60/min | ✅ Online |
| Font Squirrel | 500+ | No | Scraping | ✅ Online |
| DaFont | 500+ | No | Scraping | ✅ Online |
| 1001 Fonts | 500+ | No | Scraping | ✅ Online |
| FontSpace | 500+ | No | Scraping | ✅ Online |
| Abstract Fonts | 300+ | No | Scraping | ✅ Online |
| Urban Fonts | 300+ | No | Scraping | ✅ Online |

**Total**: 5,006 fonts across 10 providers (9/10 currently online)

## Auto-Unzip Feature

All ZIP archives are automatically extracted:

- **Detection**: Checks file extension and Content-Type header
- **Extraction**: Uses `zip` crate v7.4 with deflate, bzip2, zstd support
- **Async-Safe**: Runs in `tokio::task::spawn_blocking`
- **Cleanup**: Automatically removes ZIP after successful extraction
- **Fallback**: Returns ZIP path if extraction fails (graceful degradation)

**Example**: Download 381KB ZIP → Extract 18 font files (428KB) → Remove ZIP

## Performance

- **Search**: 514-1,010ms for 10 providers in parallel
- **Download**: ~500ms for 381KB ZIP + 50ms extraction
- **Memory**: Streaming downloads (constant memory usage)
- **Concurrency**: Configurable parallel downloads

## Error Types

Comprehensive error hierarchy with context:

- `FontError::Network` - HTTP/connection failures
- `FontError::Provider` - Provider-specific failures  
- `FontError::Parse` - Response parsing failures
- `FontError::Download` - Download failures
- `FontError::Cache` - Cache read/write failures
- `FontError::RateLimit` - Rate limit exceeded
- `FontError::Validation` - Config/input validation
- `FontError::Timeout` - Request timeout
- `FontError::Verification` - File verification failed

All errors include rich context (URLs, provider names, font IDs, etc.)

## Testing

```bash
# Run all unit tests (110 tests)
cargo test

# Run integration tests (requires network)
cargo test --test integration_providers -- --ignored
cargo test --test integration_google_fonts -- --ignored

# Run benchmarks
cargo bench

# Check code quality
cargo clippy -- -D warnings
cargo fmt --check
```

## Documentation

- [ACTUAL_TEST_RESULTS.md](ACTUAL_TEST_RESULTS.md) - Real test results with performance metrics
- [STATUS.md](STATUS.md) - Production readiness assessment (10/10)
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guidelines
- [SECURITY.md](SECURITY.md) - Security policy
- [docs/ERROR_RECOVERY.md](docs/ERROR_RECOVERY.md) - Error handling patterns

## Code Quality

- **110 Unit Tests**: 100% passing
- **9 Integration Tests**: Network-dependent
- **9 Doc Tests**: All examples compile
- **0 Clippy Warnings**: Production-ready code
- **0 Unsafe Code**: 100% safe Rust
- **0 Production `unwrap()`**: Proper error handling throughout
- **Property-Based Tests**: 4 modules with proptest coverage

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Security

See [SECURITY.md](SECURITY.md) for vulnerability reporting.
