//! Benchmarks for dx-media performance documentation.
//!
//! These benchmarks measure the performance of core operations to provide
//! documented performance characteristics for production use.
//!
//! Run with: `cargo bench -p dx-media`

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dx_media::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};
use dx_media::{sanitize_filename, validate_url, verify_content_type};

// ═══════════════════════════════════════════════════════════════════════════════
// FILENAME SANITIZATION BENCHMARKS
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_sanitize_filename(c: &mut Criterion) {
    let mut group = c.benchmark_group("filename_sanitization");

    // Simple filename
    group.bench_function("simple", |b| {
        b.iter(|| sanitize_filename(black_box("simple_file.jpg")))
    });

    // Filename with path traversal
    group.bench_function("path_traversal", |b| {
        b.iter(|| sanitize_filename(black_box("../../../etc/passwd")))
    });

    // Filename with special characters
    group.bench_function("special_chars", |b| {
        b.iter(|| sanitize_filename(black_box("file<>:\"|?*name.jpg")))
    });

    // Long filename
    let long_name = "a".repeat(500);
    group.bench_function("long_filename", |b| {
        b.iter(|| sanitize_filename(black_box(&long_name)))
    });

    // Unicode filename
    group.bench_function("unicode", |b| {
        b.iter(|| sanitize_filename(black_box("文件名_ファイル_файл.jpg")))
    });

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════
// URL VALIDATION BENCHMARKS
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_validate_url(c: &mut Criterion) {
    let mut group = c.benchmark_group("url_validation");

    // Valid public URL
    group.bench_function("valid_public", |b| {
        b.iter(|| validate_url(black_box("https://example.com/image.jpg")))
    });

    // Invalid localhost
    group.bench_function("localhost_rejection", |b| {
        b.iter(|| validate_url(black_box("http://localhost/secret")))
    });

    // Invalid private IP
    group.bench_function("private_ip_rejection", |b| {
        b.iter(|| validate_url(black_box("http://192.168.1.1/internal")))
    });

    // Invalid scheme
    group.bench_function("invalid_scheme", |b| {
        b.iter(|| validate_url(black_box("file:///etc/passwd")))
    });

    // Long URL
    let long_path = "a".repeat(1000);
    let long_url = format!("https://example.com/{}", long_path);
    group.bench_function("long_url", |b| {
        b.iter(|| validate_url(black_box(&long_url)))
    });

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTENT-TYPE VERIFICATION BENCHMARKS
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_verify_content_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_type_verification");

    // Simple match
    group.bench_function("simple_match", |b| {
        b.iter(|| verify_content_type(black_box("image/jpeg"), black_box(MediaType::Image)))
    });

    // With charset parameter
    group.bench_function("with_charset", |b| {
        b.iter(|| {
            verify_content_type(
                black_box("image/png; charset=utf-8"),
                black_box(MediaType::Image),
            )
        })
    });

    // Octet-stream (always accepted)
    group.bench_function("octet_stream", |b| {
        b.iter(|| {
            verify_content_type(
                black_box("application/octet-stream"),
                black_box(MediaType::Video),
            )
        })
    });

    // Mismatch (rejection)
    group.bench_function("mismatch_rejection", |b| {
        b.iter(|| verify_content_type(black_box("text/html"), black_box(MediaType::Image)))
    });

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════
// MEDIA ASSET BUILDER BENCHMARKS
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_media_asset_builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("media_asset_builder");

    // Minimal required fields
    group.bench_function("minimal", |b| {
        b.iter(|| {
            MediaAsset::builder()
                .id(black_box("123"))
                .provider(black_box("test"))
                .media_type(black_box(MediaType::Image))
                .title(black_box("Test"))
                .download_url(black_box("https://example.com/img.jpg"))
                .source_url(black_box("https://example.com"))
                .build()
        })
    });

    // All fields
    group.bench_function("all_fields", |b| {
        b.iter(|| {
            MediaAsset::builder()
                .id(black_box("123"))
                .provider(black_box("test"))
                .media_type(black_box(MediaType::Image))
                .title(black_box("Test Image"))
                .download_url(black_box("https://example.com/img.jpg"))
                .source_url(black_box("https://example.com"))
                .preview_url(black_box("https://example.com/thumb.jpg"))
                .author(black_box("John Doe"))
                .author_url(black_box("https://example.com/john"))
                .license(black_box(License::Cc0))
                .dimensions(black_box(1920), black_box(1080))
                .file_size(black_box(1024000))
                .mime_type(black_box("image/jpeg"))
                .tags(black_box(vec![
                    "nature".to_string(),
                    "landscape".to_string(),
                ]))
                .build()
        })
    });

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════
// SEARCH QUERY BENCHMARKS
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_search_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_query");

    // Simple query
    group.bench_function("simple", |b| {
        b.iter(|| SearchQuery::new(black_box("nature")))
    });

    // Query with type
    group.bench_function("with_type", |b| {
        b.iter(|| SearchQuery::for_type(black_box("nature"), black_box(MediaType::Image)))
    });

    // Full query with all options
    group.bench_function("full_options", |b| {
        b.iter(|| {
            SearchQuery::new(black_box("nature"))
                .media_type(black_box(MediaType::Image))
                .count(black_box(50))
                .page(black_box(1))
                .min_dimensions(black_box(1920), black_box(1080))
        })
    });

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════
// SEARCH RESULT BENCHMARKS
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_search_result(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_result");

    // Create result
    group.bench_function("create", |b| {
        b.iter(|| SearchResult::new(black_box("nature")))
    });

    // Merge small results
    group.bench_function("merge_small", |b| {
        b.iter(|| {
            let mut r1 = SearchResult::new("nature");
            r1.total_count = 10;
            r1.providers_searched = vec!["p1".to_string()];

            let mut r2 = SearchResult::new("nature");
            r2.total_count = 15;
            r2.providers_searched = vec!["p2".to_string()];

            r1.merge(black_box(r2));
            r1
        })
    });

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERIALIZATION BENCHMARKS
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    // Create a sample asset
    let asset = MediaAsset::builder()
        .id("test-123")
        .provider("openverse")
        .media_type(MediaType::Image)
        .title("Beautiful Sunset")
        .download_url("https://example.com/sunset.jpg")
        .source_url("https://example.com/photos/sunset")
        .preview_url("https://example.com/sunset_thumb.jpg")
        .author("Jane Photographer")
        .license(License::CcBy)
        .dimensions(1920, 1080)
        .tags(vec![
            "sunset".to_string(),
            "nature".to_string(),
            "sky".to_string(),
        ])
        .build()
        .unwrap();

    // Serialize MediaAsset
    group.bench_function("serialize_asset", |b| {
        b.iter(|| serde_json::to_string(black_box(&asset)))
    });

    // Serialize and deserialize (round-trip)
    let json = serde_json::to_string(&asset).unwrap();
    group.bench_function("deserialize_asset", |b| {
        b.iter(|| serde_json::from_str::<MediaAsset>(black_box(&json)))
    });

    // Create a sample search result with multiple assets
    let mut result = SearchResult::new("sunset");
    result.total_count = 100;
    result.providers_searched = vec!["openverse".to_string(), "pixabay".to_string()];
    for i in 0..10 {
        let a = MediaAsset::builder()
            .id(format!("asset-{}", i))
            .provider("test")
            .media_type(MediaType::Image)
            .title(format!("Asset {}", i))
            .download_url(format!("https://example.com/{}.jpg", i))
            .source_url("https://example.com")
            .build()
            .unwrap();
        result.assets.push(a);
    }

    // Serialize SearchResult
    group.bench_function("serialize_result_10_assets", |b| {
        b.iter(|| serde_json::to_string(black_box(&result)))
    });

    let result_json = serde_json::to_string(&result).unwrap();
    group.bench_function("deserialize_result_10_assets", |b| {
        b.iter(|| serde_json::from_str::<SearchResult>(black_box(&result_json)))
    });

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMIT CONFIG BENCHMARKS
// ═══════════════════════════════════════════════════════════════════════════════

fn bench_rate_limit(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_limit");

    // Create config
    group.bench_function("create", |b| {
        b.iter(|| RateLimitConfig::new(black_box(100), black_box(60)))
    });

    // Calculate delay
    let config = RateLimitConfig::new(100, 60);
    group.bench_function("delay_calculation", |b| {
        b.iter(|| black_box(&config).delay_ms())
    });

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════
// CRITERION CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

criterion_group!(
    benches,
    bench_sanitize_filename,
    bench_validate_url,
    bench_verify_content_type,
    bench_media_asset_builder,
    bench_search_query,
    bench_search_result,
    bench_serialization,
    bench_rate_limit,
);

criterion_main!(benches);
