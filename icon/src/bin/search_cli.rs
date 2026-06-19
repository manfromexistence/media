use dx_icons::engine::IconSearchEngine;
use dx_icons::index::IconIndex;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    // Try multiple possible index locations
    let possible_paths = vec![
        PathBuf::from("index"),
        PathBuf::from("crates/media/icon/index"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("index")))
            .unwrap_or_else(|| PathBuf::from("index")),
    ];

    let index_dir = possible_paths
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!("Icon index not found. Run 'build_index' first."))?;

    println!("Loading icon search index from {:?}...", index_dir);
    let index = IconIndex::load(index_dir)?;
    let engine = IconSearchEngine::from_index(index)?;

    println!("Loaded {} icons", engine.total_icons());
    println!("Type a query to search (or 'quit' to exit):\n");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut query = String::new();
        let bytes_read = io::stdin().read_line(&mut query)?;

        // EOF reached (e.g., piped input finished)
        if bytes_read == 0 {
            break;
        }

        let query = query.trim();

        if query.is_empty() {
            continue;
        }

        if query == "quit" || query == "exit" {
            break;
        }

        let is_cached = engine.is_cached(query);
        let start = std::time::Instant::now();
        let results = engine.search(query, usize::MAX); // Return ALL matching results
        let elapsed = start.elapsed();

        let cache_status = if is_cached {
            "🔥 CACHED"
        } else {
            "❄️  COLD"
        };
        println!(
            "\n{} - Found {} results in {:?}:",
            cache_status,
            results.len(),
            elapsed
        );
        for (i, result) in results.iter().enumerate() {
            println!(
                "  {}. {} ({}) - score: {:.2} [{:?}]",
                i + 1,
                result.icon.name,
                result.icon.pack,
                result.score,
                result.match_type
            );
        }
        println!();
    }

    Ok(())
}
