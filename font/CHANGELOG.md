
# Changelog

All notable changes to font will be documented in this file. The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

## [0.2.0] - 2024-12-28

### Added

- Custom error types with `FontError` enum providing detailed context for all failure modes
- `CacheManager` for caching provider responses with configurable TTL
- `RateLimiter` with token bucket algorithm and per-provider rate limits
- `RetryClient` with exponential backoff and jitter for HTTP requests
- `FileVerifier` for validating downloaded font files (magic bytes, checksums)
- `ConfigBuilder` for ergonomic configuration construction
- Tracing spans for async operations (search, download, provider queries)
- INFO/WARN level logging for significant events and failures
- Comprehensive rustdoc documentation with examples
- `prelude` module exporting commonly used types
- Property-based tests for core components (cache, rate limiter, retry policy)
- Real API integration for DaFont and FontSpace providers (web scraping)
- Provider error reporting in `SearchResults`
- Checksum verification support for downloads
- `DownloadResult` with verification status, bytes downloaded, and duration
- Integration tests for Google Fonts and provider health checks
- Benchmarks for parallel search performance
- Error recovery examples and documentation
- Advanced usage examples (batch downloads, CDN URLs, filtering)
- SECURITY.md with vulnerability reporting guidelines
- CONTRIBUTING.md with development workflow and guidelines

### Changed

- All provider methods now return `FontResult<T>` instead of `Result<T>`
- Improved error messages with full context (URLs, provider names, attempt counts)
- Search results now include `from_cache` and `provider_errors` fields
- Configuration validation rejects invalid values (timeout=0, rate_limit<=0)
- `RetryClient::with_defaults()` now returns `Result` instead of panicking
- README examples use proper error handling with `?` instead of `unwrap()`

### Fixed

- All clippy warnings resolved
- All `cargo doc` warnings resolved
- Removed dead code and unused variables
- Production code no longer uses `expect()` or `unwrap()`
- Property-based tests use `is_some_and()` instead of deprecated `map_or()`

## [0.1.0] - 2024-01-01

### Added

- Initial implementation with basic font search and download
- Support for Google Fonts, Bunny Fonts, Fontsource, FontShare providers
- CLI interface with search, download, list, info, stats commands
