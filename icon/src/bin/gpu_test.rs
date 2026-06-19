use dx_icon_search::gpu::GpuSearchEngineSync;

fn main() {
    println!("=== GPU Detection Test ===\n");

    // Check if GPU is available
    let gpu_engine = GpuSearchEngineSync::new();

    if gpu_engine.is_available() {
        println!("✅ GPU DETECTED!");
        println!("GPU acceleration is available on this system.\n");

        // Test GPU search performance
        println!("Testing GPU search performance...");

        let test_icons: Vec<String> = (0..10000).map(|i| format!("icon-{}", i)).collect();

        let query = "icon-5000";

        let start = std::time::Instant::now();
        if let Some(results) = gpu_engine.search(query, &test_icons) {
            let elapsed = start.elapsed();
            println!("✅ GPU Search completed!");
            println!("   Query: '{}'", query);
            println!("   Icons searched: {}", test_icons.len());
            println!("   Results found: {}", results.len());
            println!("   Time: {:?}", elapsed);
            println!(
                "   Throughput: {:.0} icons/ms",
                test_icons.len() as f64 / elapsed.as_secs_f64() / 1000.0
            );
        } else {
            println!("❌ GPU search failed");
        }
    } else {
        println!("❌ NO GPU DETECTED");
        println!("GPU acceleration is NOT available on this system.");
        println!("The engine will fall back to CPU-only mode.");
    }

    println!("\n=== System Information ===");
    println!("OS: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);
}
