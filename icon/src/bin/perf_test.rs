use dx_icon_search::engine::IconSearchEngine;
use dx_icon_search::index::IconIndex;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let index_dir = PathBuf::from("index");
    let index = IconIndex::load(&index_dir)?;
    let engine = IconSearchEngine::from_index(index)?;

    println!("=== Performance Test (Multiple Runs) ===\n");

    let queries = vec![("home", 1218), ("arrow", 9563)];

    for (query, expected) in queries {
        println!("Query: '{}' (expected {} results)", query, expected);

        // First run (cold cache)
        let start = std::time::Instant::now();
        let results = engine.search(query, usize::MAX);
        let cold_time = start.elapsed();

        // Second run (warm cache)
        let start = std::time::Instant::now();
        let results2 = engine.search(query, usize::MAX);
        let warm_time = start.elapsed();

        // Third run (verify consistency)
        let start = std::time::Instant::now();
        let results3 = engine.search(query, usize::MAX);
        let warm_time2 = start.elapsed();

        println!("  Cold cache: {} results in {:?}", results.len(), cold_time);
        println!(
            "  Warm cache: {} results in {:?}",
            results2.len(),
            warm_time
        );
        println!(
            "  Warm cache: {} results in {:?}",
            results3.len(),
            warm_time2
        );
        println!();
    }

    Ok(())
}
