//! Icon search and export commands

use anyhow::Result;
use console::style;
use std::path::PathBuf;

use crate::cli_unified::args::{IconCommands, OutputFormat};
use crate::cli_unified::config::MediaConfig;
use crate::cli_unified::output::{print_info, print_success};

pub async fn execute_icon_command(
    command: IconCommands,
    format: &OutputFormat,
    config: &MediaConfig,
) -> Result<()> {
    match command {
        IconCommands::Search { query, limit, pack } => {
            cmd_search(&query, limit, pack.as_deref(), format).await
        }
        IconCommands::Export {
            query,
            mut output,
            limit,
            pack,
        } => {
            // Use config directory if output is default
            if output == PathBuf::from("./icons") {
                output = config.get_icon_dir();
                config.ensure_dir(&output)?;
            }
            cmd_export(&query, &output, limit, pack.as_deref()).await
        }
        IconCommands::Desktop { icons } => cmd_desktop(&icons).await,
        IconCommands::Packs => cmd_packs().await,
    }
}

async fn cmd_search(
    query: &str,
    limit: usize,
    pack_filter: Option<&str>,
    _format: &OutputFormat,
) -> Result<()> {
    print_info(&format!("🔍 Searching icons for '{}'...", query));

    // Load icon engine
    let engine = load_icon_engine()?;
    let start = std::time::Instant::now();
    let mut results = engine.search(query, limit * 10);

    // Filter by pack if specified
    if let Some(pack) = pack_filter {
        results.retain(|r| r.icon.pack == pack);
        results.truncate(limit);
    } else {
        results.truncate(limit);
    }

    let elapsed = start.elapsed();

    if results.is_empty() {
        println!("No results for '{}'", query);
        return Ok(());
    }

    match _format {
        OutputFormat::Json => {
            // JSON output not supported for icons yet
            println!("{{\"results\": [], \"count\": {}}}", results.len());
        }
        _ => {
            println!(
                "\n{} ({:.2}ms):\n",
                style(format!("Found {} icons", results.len())).green(),
                elapsed.as_secs_f64() * 1000.0
            );
            for (i, result) in results.iter().enumerate() {
                println!("  {}. {} ({})", i + 1, result.icon.name, result.icon.pack);
            }
        }
    }

    Ok(())
}

async fn cmd_export(
    query: &str,
    output_dir: &PathBuf,
    limit: usize,
    pack_filter: Option<&str>,
) -> Result<()> {
    print_info(&format!("📦 Exporting icons for '{}'...", query));

    let engine = load_icon_engine()?;
    let start = std::time::Instant::now();
    let mut results = engine.search(query, limit * 10);

    if let Some(pack) = pack_filter {
        results.retain(|r| r.icon.pack == pack);
        results.truncate(limit);
    } else {
        results.truncate(limit);
    }

    if results.is_empty() {
        println!("No icons found");
        return Ok(());
    }

    std::fs::create_dir_all(output_dir)?;

    for result in &results {
        let filename = format!("{}_{}.svg", result.icon.pack, result.icon.name);
        let filepath = output_dir.join(&filename);
        let svg_content = generate_svg(&result.icon.name, &result.icon.pack)?;
        std::fs::write(&filepath, svg_content)?;
        println!("✓ {}", filename);
    }

    print_success(&format!(
        "Exported {} icons in {:.2}s",
        results.len(),
        start.elapsed().as_secs_f64()
    ));

    Ok(())
}

async fn cmd_desktop(icon_specs: &[String]) -> Result<()> {
    print_info("📦 Exporting icons to desktop app...");

    let desktop_icons_dir = PathBuf::from("apps/desktop/assets/icons");
    std::fs::create_dir_all(&desktop_icons_dir)?;

    let engine = load_icon_engine()?;
    let mut exported = 0;
    let mut failed = 0;

    for spec in icon_specs {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() != 2 {
            eprintln!("✗ Invalid: {} (use name:pack)", spec);
            failed += 1;
            continue;
        }

        let (name, pack) = (parts[0], parts[1]);
        let results = engine.search(name, 100);

        let icon = results
            .iter()
            .find(|r| r.icon.pack == pack && r.icon.name == name)
            .or_else(|| results.first());

        if let Some(result) = icon {
            let filename = format!("{}.svg", result.icon.name);
            let filepath = desktop_icons_dir.join(&filename);
            let svg_content = generate_svg(&result.icon.name, &result.icon.pack)?;
            std::fs::write(&filepath, svg_content)?;
            println!("✓ {} ({})", result.icon.name, result.icon.pack);
            exported += 1;
        } else {
            eprintln!("✗ Not found: {}", spec);
            failed += 1;
        }
    }

    if exported > 0 {
        print_success(&format!("Exported {} icons", exported));
    }
    if failed > 0 {
        eprintln!("Failed: {}", failed);
    }

    Ok(())
}

async fn cmd_packs() -> Result<()> {
    print_info("📚 Available icon packs\n");

    let engine = load_icon_engine()?;
    let results = engine.search("a", 10000);

    let mut packs: Vec<String> = results
        .iter()
        .map(|r| r.icon.pack.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    packs.sort();

    println!(
        "{}\n",
        style(format!("Total: {} packs", packs.len()))
            .green()
            .bold()
    );

    for pack in packs {
        println!("  {}", pack);
    }

    Ok(())
}

// Helper functions

fn load_icon_engine() -> Result<dx_icons::IconSearchEngine> {
    use dx_icons::index::IconIndex;

    let possible_paths = vec![
        PathBuf::from("crates/media/icon/index"),
        PathBuf::from("icon/index"),
        PathBuf::from("index"),
        PathBuf::from("../../icon/index"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("index")))
            .unwrap_or_else(|| PathBuf::from("index")),
    ];

    let index_dir = possible_paths
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!("Icon index not found at any of: {:?}", possible_paths))?;

    let index = IconIndex::load(index_dir)?;
    Ok(dx_icons::IconSearchEngine::from_index(index)?)
}

fn generate_svg(name: &str, pack: &str) -> Result<String> {
    let possible_data_dirs = vec![
        PathBuf::from("data"),
        PathBuf::from("crates/media/icon/data"),
        PathBuf::from("../../data"),
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("crates/media/icon/data"))
            })
            .unwrap_or_else(|| PathBuf::from("data")),
    ];

    let data_dir = possible_data_dirs
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!("Icon data directory not found"))?;

    let pack_file = data_dir.join(format!("{}.json", pack));

    if !pack_file.exists() {
        return Err(anyhow::anyhow!("Pack '{}' not found", pack));
    }

    let content = std::fs::read_to_string(&pack_file)?;
    let pack_data: serde_json::Value = serde_json::from_str(&content)?;

    let icon_data = pack_data["icons"]
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("Icon '{}' not found in '{}'", name, pack))?;

    let body = icon_data["body"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Icon body not found"))?;

    let width = icon_data["width"]
        .as_f64()
        .or_else(|| pack_data["width"].as_f64())
        .unwrap_or(24.0);

    let height = icon_data["height"]
        .as_f64()
        .or_else(|| pack_data["height"].as_f64())
        .unwrap_or(24.0);

    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">{}</svg>"#,
        width, height, width, height, body
    ))
}
