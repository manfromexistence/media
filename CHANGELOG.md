
# Changelog

All notable changes to DX Media will be documented in this file. The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## Unreleased - 2026-05-25

### Changed

- Added source-only receipt hardening for media tool outputs: default `ToolOutput` receipts now include a receipt version, source kind, status, inferred tool name, callsite, and default/explicit completeness metadata.
- Added explicit receipt constructors for provider-backed, fixture-backed, and credential-required tool outputs.
- Tightened media provenance helpers so ambiguous provider labels such as provider names or "free" labels are not treated as known licenses.
- Changed media type validation so assets without MIME or extension evidence are not silently treated as valid.
- Added provider provenance metadata and type handling for DPLA, Europeana, Data.gov, and Wikimedia source mappings.
- Replaced the utility duplicate/checksum toy hashing path with real checksum command backends where available.
- Replaced the missing `dx-serializer` path dependency with a small local media CLI config parser for the exact `media.cli.*` settings this crate consumes.
- Gated icon/font CLI path dependencies behind the existing `cli` feature so library-only provenance tests can compile without the unified CLI stack.
- Pinned `dx-font` away from yanked `zip` 7.4.0 to a non-yanked 6.x line.
- Tightened Data.gov provenance so MIME evidence can override loose format labels and the provider no longer claims a blanket default public-domain license.
- Added receipt-returning download APIs that preserve provider, source URL, download URL, license, byte count, MIME evidence, provider metadata, and output type validation.
- Expanded the media tool registry from 60 to 65 descriptors by adding the credential-required speech tools and FFmpeg-backed speech-preparation helpers.
- Tightened URL extension guessing for downloads to use the parsed final URL path segment and declared media type, avoiding substring matches from parent paths or query strings.

### Fixed

- Native PDF merge no longer reports success while only copying the first input PDF; it now returns an unsupported error until real multi-PDF merging is implemented.
- Native image compression now rejects non-JPEG output paths instead of writing JPEG bytes to misleading extensions.
- Native tar.gz extraction now rejects unsafe non-file/non-directory tar entry types such as symlinks.
- Native ZIP extraction rejects parent-directory entries, and the extended archive CLI now routes ZIP create/list/extract through receipt-bearing native helpers.

### Verification

- Source-only `rustfmt --edition 2024 --check --config skip_children=true` passed for the touched Rust files.
- `cargo metadata --locked --offline --no-deps --format-version 1` passes in this checkout.
- `cargo test --locked --offline --no-default-features --features archive-core,utility-core --test tool_receipt_tests -j1 -- --nocapture` passed: 10 passed, 0 failed.
- `cargo test --locked --offline --no-default-features --features archive-core,image-core,utility-core --test native_safety_tests -j1 -- --nocapture` passed: 3 passed, 0 failed.
- `cargo test --locked --offline --no-default-features --features archive-core,utility-core --test provider_parsing_tests -j1 -- --nocapture` passed: 8 passed, 0 failed.
- `cargo test --locked --offline --no-default-features --features archive-core,utility-core --test wiremock_integration_tests -j1 -- --nocapture` passed: 14 passed, 0 failed.
- Broad `cargo fmt --check` still reports pre-existing formatting drift outside the touched files.
- `cargo check --locked --offline --no-default-features --features cli -j1` was attempted twice and timed out before a result; CLI-only compile coverage remains unproven.

## 1.0.0 - 2026-01-13

### Added

- Constants Module (`src/constants.rs`)
- `EARLY_EXIT_MULTIPLIER`
- Documented constant for search early-exit threshold (3x)
- `DEFAULT_FAILURE_THRESHOLD`
- Circuit breaker failure threshold (3)
- `DEFAULT_RESET_TIMEOUT_SECS`
- Circuit breaker reset timeout (60s)
- `DEFAULT_RATE_LIMIT_REQUESTS`
- Default rate limit (100 requests)
- `DEFAULT_RATE_LIMIT_WINDOW_SECS`
- Rate limit window (60s)
- `BASE_BACKOFF_MS`
- HTTP retry base delay (1000ms)
- `MAX_BACKOFF_JITTER_MS`
- Backoff jitter (500ms)
- Builder Methods
- `MediaAssetBuilder::build_or_log()`
- Build with debug-level logging on failure
- Integration Tests
- Wiremock-based integration tests for NASA and Openverse providers
- Test fixtures for provider response parsing
- Rate limiting integration tests
- Property-Based Tests
- Lock poisoning recovery property test
- Provider response parsing correctness property test
- Builder error message specificity property test
- Documentation
- External dependencies section with minimum versions
- Docker deployment examples (full and minimal)
- Troubleshooting guide for common issues
- Dependency matrix showing which tools require which dependencies

### Changed

- Circuit Breaker
- Safe lock handling that recovers from poisoned locks instead of panicking
- User-Agent
- Changed from browser impersonation to honest identification (`dx-media/VERSION`)
- Clippy Configuration
- Reduced blanket suppressions from 50+ to justified item-level suppressions
- HTTP Client
- Uses documented constants instead of magic numbers

### Deprecated

- `MediaAssetBuilder::try_build()`
- Use `build()` for explicit errors or `build_or_log()` for logging

### Removed

- Unused `timeout` field from HTTP client
- Dead code with "future use" comments
- Blanket `#[allow(dead_code)]` annotations

### Fixed

- Circuit breaker no longer panics on lock poisoning
- Builder validation errors now specify which field is missing

### Security

- Honest User-Agent string for responsible API usage
- SSRF prevention in URL validation
- Content-type verification for downloads
- Filename sanitization for downloaded files

## 0.1.0 - 2025-11-30

### Added

- Core Library (`dx_media`)
- `DxMedia` facade for easy library usage with fluent search builder API
- `SearchEngine` for multi-provider parallel searching
- `Downloader` with async file downloads and retry logic
- `FileManager` for organized file storage by provider/type
- `HttpClient` with built-in rate limiting and exponential backoff
- Provider Support
- Unsplash provider (images)
- requires API key
- Pexels provider (images, videos)
- requires API key
- Pixabay provider (images, videos, vectors)
- requires API key
- `ProviderRegistry` for dynamic provider management
- `Provider` trait for implementing custom providers
- CLI (`dx`)
- `dx search <query>`
- Search across all configured providers-`--type` filter (image, video, audio, gif, vector)
- `--provider` filter for specific providers
- `--count` and `--page` for pagination
- `--orientation` filter (landscape, portrait, square)
- `--color` filter for dominant color
- `--download` flag to auto-download first result
- `dx download <provider:id>`
- Download specific asset
- `dx scrape <url>`
- Scrape and download media from any website-`--type` filter (image, video, audio, gif, vector, all)
- `--count` limit for number of assets
- `--depth` for link-following depth
- `--pattern` for file pattern matching
- `--dry-run` to preview without downloading
- `dx providers`
- List available providers and their status
- `dx config`
- Show current configuration
- Multiple output formats: text, json, json-compact, tsv
- Configuration
- Environment variable configuration
- `.env` file support via dotenvy
- Configurable download directory, timeouts, retry attempts
- Per-provider API key configuration
- Types
- `MediaType` enum (Image, Video, Audio, Gif, Vector, Document, Data, Model3D, Code, Text)
- `MediaAsset` with comprehensive metadata
- `SearchQuery` with filters and pagination
- `SearchResult` with aggregated results from multiple providers
- `License` types (CC0, CC-BY, Unsplash, Pexels, Pixabay, etc.)

### Technical Details

- Built with Rust 2024 Edition
- Async runtime: Tokio with full features
- HTTP client: reqwest with rustls-tls, gzip, brotli compression
- CLI framework: clap with derive macros
- Serialization: serde + serde_json
- Error handling: thiserror + anyhow
- Logging: tracing with env-filter
