use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use dx_font::prelude::*;
use tokio::runtime::Runtime;

fn bench_search_single_provider(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("search_single_provider", |b| {
        b.to_async(&rt).iter(|| async {
            let search = FontSearch::new().unwrap();
            let results = search.search(black_box("roboto")).await.unwrap();
            black_box(results);
        });
    });
}

fn bench_search_parallel(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("search_parallel_all_providers", |b| {
        b.to_async(&rt).iter(|| async {
            let search = FontSearch::new().unwrap();
            let results = search.search(black_box("open sans")).await.unwrap();
            black_box(results);
        });
    });
}

fn bench_search_with_cache(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Prime the cache
    rt.block_on(async {
        let s = FontSearch::new().unwrap();
        let _ = s.search("roboto").await;
    });

    c.bench_function("search_cached", |b| {
        b.to_async(&rt).iter(|| async {
            let search = FontSearch::new().unwrap();
            let results = search.search(black_box("roboto")).await.unwrap();
            black_box(results);
        });
    });
}

fn bench_search_query_length(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("search_by_query_length");

    for query in &["a", "rob", "roboto", "open sans mono"] {
        group.bench_with_input(BenchmarkId::from_parameter(query), query, |b, &q| {
            b.to_async(&rt).iter(|| async {
                let search = FontSearch::new().unwrap();
                let results = search.search(black_box(q)).await.unwrap();
                black_box(results);
            });
        });
    }

    group.finish();
}

fn bench_concurrent_searches(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_5_searches", |b| {
        b.to_async(&rt).iter(|| async {
            let queries = vec!["roboto", "inter", "lato", "montserrat", "poppins"];

            let mut handles = Vec::new();
            for query in queries {
                let handle = tokio::spawn(async move {
                    let search = FontSearch::new().unwrap();
                    search.search(query).await.unwrap()
                });
                handles.push(handle);
            }

            for handle in handles {
                black_box(handle.await.unwrap());
            }
        });
    });
}

criterion_group!(
    benches,
    bench_search_single_provider,
    bench_search_parallel,
    bench_search_with_cache,
    bench_search_query_length,
    bench_concurrent_searches
);
criterion_main!(benches);
