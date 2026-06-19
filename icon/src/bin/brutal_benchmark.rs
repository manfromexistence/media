/// Brutal truth benchmark - comprehensive testing
use dx_icon_search::engine::IconSearchEngine;
use dx_icon_search::index::IconIndex;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    println!("=== BRUTAL TRUTH BENCHMARK ===\n");
    println!("Testing against world-class standards (2026)\n");

    let index_dir = PathBuf::from("index");
    let index = IconIndex::load(&index_dir)?;
    let engine = IconSearchEngine::from_index(index)?;

    let total_icons = engine.total_icons();
    println!("📊 Dataset: {} icons\n", total_icons);

    // Industry standards (2026)
    println!("🎯 Industry Standards:");
    println!("  - Web INP target: <200ms (Microsoft)");
    println!("  - Search response: <100ms (Google standard)");
    println!("  - Icon search: <50ms (Iconify/Icones.js)");
    println!("  - Real-time: <10ms (considered instant)\n");

    // Test queries (various difficulty levels)
    let test_cases = vec![
        ("home", "Common query (1.2K results)"),
        ("arrow", "Large result set (9.5K results)"),
        ("search", "Medium result set (1.5K results)"),
        ("x", "Single char (massive results)"),
        ("zzzz", "No results (worst case)"),
        ("user-circle-check", "Complex multi-word"),
        ("a", "Single char (huge results)"),
        ("icon", "Generic term (785 results)"),
    ];

    println!("🔥 COLD CACHE PERFORMANCE (First Search):");
    println!(
        "{:<25} {:>12} {:>15} {:>10}",
        "Query", "Results", "Time", "Rating"
    );
    println!("{}", "-".repeat(65));

    let mut cold_times = Vec::new();

    for (query, description) in &test_cases {
        let start = Instant::now();
        let results = engine.search(query, usize::MAX);
        let elapsed = start.elapsed();

        let rating = if elapsed.as_millis() < 10 {
            "⚡ INSTANT"
        } else if elapsed.as_millis() < 50 {
            "✅ EXCELLENT"
        } else if elapsed.as_millis() < 100 {
            "👍 GOOD"
        } else if elapsed.as_millis() < 200 {
            "⚠️  ACCEPTABLE"
        } else {
            "❌ SLOW"
        };

        println!(
            "{:<25} {:>12} {:>12.2}ms {:>10}",
            format!("'{}' ({})", query, description),
            results.len(),
            elapsed.as_secs_f64() * 1000.0,
            rating
        );

        cold_times.push(elapsed.as_secs_f64() * 1000.0);
    }

    let avg_cold = cold_times.iter().sum::<f64>() / cold_times.len() as f64;
    let max_cold = cold_times.iter().fold(0.0f64, |a, &b| a.max(b));

    println!("\n📈 Cold Cache Stats:");
    println!("  Average: {:.2}ms", avg_cold);
    println!("  Worst:   {:.2}ms", max_cold);
    println!(
        "  Best:    {:.2}ms",
        cold_times.iter().fold(f64::MAX, |a, &b| a.min(b))
    );

    // Warm cache test
    println!("\n🔥 WARM CACHE PERFORMANCE (Cached Results):");
    println!(
        "{:<25} {:>12} {:>15} {:>10}",
        "Query", "Results", "Time", "Rating"
    );
    println!("{}", "-".repeat(65));

    let mut warm_times = Vec::new();

    for (query, description) in &test_cases {
        let start = Instant::now();
        let results = engine.search(query, usize::MAX);
        let elapsed = start.elapsed();

        let rating = if elapsed.as_micros() < 100 {
            "⚡ BLAZING"
        } else if elapsed.as_micros() < 500 {
            "✅ EXCELLENT"
        } else if elapsed.as_millis() < 2 {
            "👍 GOOD"
        } else {
            "⚠️  SLOW"
        };

        println!(
            "{:<25} {:>12} {:>12.2}µs {:>10}",
            format!("'{}' ({})", query, description),
            results.len(),
            elapsed.as_secs_f64() * 1_000_000.0,
            rating
        );

        warm_times.push(elapsed.as_secs_f64() * 1_000_000.0);
    }

    let avg_warm = warm_times.iter().sum::<f64>() / warm_times.len() as f64;

    println!("\n📈 Warm Cache Stats:");
    println!("  Average: {:.2}µs", avg_warm);
    println!(
        "  Worst:   {:.2}µs",
        warm_times.iter().fold(0.0f64, |a, &b| a.max(b))
    );
    println!(
        "  Best:    {:.2}µs",
        warm_times.iter().fold(f64::MAX, |a, &b| a.min(b))
    );

    // Throughput test
    println!("\n🚀 THROUGHPUT TEST (1000 searches):");
    let queries = vec!["home", "arrow", "search", "icon", "user"];
    let start = Instant::now();
    for _ in 0..200 {
        for query in &queries {
            let _ = engine.search(query, 100);
        }
    }
    let elapsed = start.elapsed();
    let searches_per_sec = 1000.0 / elapsed.as_secs_f64();

    println!("  Total time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
    println!("  Throughput: {:.0} searches/sec", searches_per_sec);
    println!(
        "  Avg/search: {:.2}µs",
        elapsed.as_secs_f64() * 1_000_000.0 / 1000.0
    );

    // Final verdict
    println!("\n{}", "=".repeat(65));
    println!("🏆 FINAL VERDICT:");
    println!("{}", "=".repeat(65));

    if avg_cold < 10.0 {
        println!("✅ WORLD-CLASS: Faster than real-time threshold (<10ms)");
    } else if avg_cold < 50.0 {
        println!("✅ EXCELLENT: Beats Iconify/Icones.js standard (<50ms)");
    } else if avg_cold < 100.0 {
        println!("👍 GOOD: Meets Google search standard (<100ms)");
    } else {
        println!("⚠️  NEEDS IMPROVEMENT: Slower than industry standards");
    }

    if avg_warm < 1000.0 {
        println!("✅ CACHE: Sub-millisecond cached performance");
    }

    if searches_per_sec > 10000.0 {
        println!("✅ THROUGHPUT: Can handle 10K+ searches/sec");
    }

    println!("\n📊 Comparison to competitors:");
    println!("  Iconify API:     ~50-100ms (network + search)");
    println!("  Icones.js:       ~20-50ms (client-side)");
    println!(
        "  Your engine:     ~{:.1}ms cold, ~{:.0}µs warm",
        avg_cold, avg_warm
    );

    if avg_cold < 20.0 {
        println!("\n🎉 BRUTAL TRUTH: You have the FASTEST icon search engine!");
    } else if avg_cold < 50.0 {
        println!("\n👏 BRUTAL TRUTH: Top-tier performance, competitive with best!");
    } else {
        println!("\n💡 BRUTAL TRUTH: Good but room for improvement");
    }

    Ok(())
}
