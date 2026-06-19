use dx_icon_search::engine::IconSearchEngine;
use dx_icon_search::gpu::GpuSearchEngineSync;
use dx_icon_search::index::IconIndex;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("=== GPU vs CPU Benchmark ===\n");

    // Load actual icon index
    let index_dir = PathBuf::from("index");

    if !index_dir.exists() {
        println!("❌ Index directory not found. Run build_index first.");
        return Ok(());
    }

    println!("Loading icon index...");
    let index = IconIndex::load(&index_dir)?;
    let cpu_engine = IconSearchEngine::from_index(index)?;

    let total_icons = cpu_engine.total_icons();
    println!("✅ Loaded {} icons\n", total_icons);

    // Check GPU availability
    let gpu_engine = GpuSearchEngineSync::new();

    if !gpu_engine.is_available() {
        println!("❌ GPU not available. CPU-only mode.");
        return Ok(());
    }

    println!("✅ GPU detected!\n");

    // Test queries
    let queries = vec!["home", "arrow", "search", "icon", "user"];

    println!("=== Performance Comparison ===\n");

    for query in queries {
        println!("Query: '{}'", query);

        // CPU benchmark
        let start = std::time::Instant::now();
        let cpu_results = cpu_engine.search(query, usize::MAX);
        let cpu_time = start.elapsed();

        println!(
            "  CPU: {} results in {:?} ({:.0} icons/ms)",
            cpu_results.len(),
            cpu_time,
            total_icons as f64 / cpu_time.as_secs_f64() / 1000.0
        );

        // Note: GPU implementation needs icon names extracted
        // For now, showing CPU performance
        println!();
    }

    println!("=== System Information ===");
    println!("OS: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);
    println!("Total Icons: {}", total_icons);

    Ok(())
}
