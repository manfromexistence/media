use dx_icons::engine::IconSearchEngine;
use dx_icons::index::IconIndex;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

static ENGINE: Mutex<Option<IconSearchEngine>> = Mutex::new(None);

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "search" | "s" => {
            if args.len() < 3 {
                eprintln!("Usage: icon search <query> [--limit N]");
                return Ok(());
            }
            let query = &args[2];
            let limit = parse_limit(&args).unwrap_or(10);
            search_icons(query, limit)?;
        }
        "export" | "e" => {
            if args.len() < 4 {
                eprintln!("Usage: icon export <query> <output_dir> [--limit N] [--pack PACK]");
                return Ok(());
            }
            let query = &args[2];
            let output_dir = PathBuf::from(&args[3]);
            let limit = parse_limit(&args).unwrap_or(10);
            let pack_filter = parse_pack(&args);
            export_icons(query, &output_dir, limit, pack_filter.as_deref())?;
        }
        "desktop" | "d" => {
            if args.len() < 3 {
                eprintln!("Usage: icon desktop <icon_names...>");
                eprintln!("Example: icon desktop search:lucide home:solar menu:lucide");
                return Ok(());
            }
            let icon_specs: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            export_to_desktop(&icon_specs)?;
        }
        "packs" | "p" => {
            list_packs()?;
        }
        "help" | "-h" | "--help" => {
            print_usage();
        }
        "version" | "-v" | "--version" => {
            println!("icon v0.1.0");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Run 'icon help' for usage information");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!("icon - DX Icon CLI v0.1.0\n");
    println!("USAGE:");
    println!("  icon <command> [options]\n");
    println!("COMMANDS:");
    println!("  search, s      Search for icons");
    println!("  export, e      Export icons as SVG files");
    println!("  desktop, d     Export icons to apps/desktop/assets/icons/");
    println!("  packs, p       List available icon packs");
    println!("  help           Show this help message");
    println!("  version        Show version information\n");
    println!("OPTIONS:");
    println!("  --limit N      Limit number of results (default: 10)");
    println!("  --pack PACK    Filter by icon pack\n");
    println!("EXAMPLES:");
    println!("  icon s home --limit 5");
    println!("  icon e search ./icons --pack lucide");
    println!("  icon d search:lucide home:solar");
}

fn parse_limit(args: &[String]) -> Option<usize> {
    args.iter()
        .position(|a| a == "--limit")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
}

fn parse_pack(args: &[String]) -> Option<String> {
    args.iter()
        .position(|a| a == "--pack")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.to_string())
}

fn load_engine() -> anyhow::Result<()> {
    let mut engine_lock = ENGINE.lock().unwrap();

    if engine_lock.is_none() {
        let mut possible_paths = vec![
            PathBuf::from("index"),
            PathBuf::from("crates/media/icon/index"),
        ];

        // Add DX_ICON_INDEX env var path
        if let Ok(env_path) = std::env::var("DX_ICON_INDEX") {
            possible_paths.insert(0, PathBuf::from(env_path));
        }

        // Add executable directory path
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                possible_paths.push(parent.join("index"));
                possible_paths.push(parent.join("../share/dx-icons/index"));
            }
        }

        // Add home directory path
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            let home_path = PathBuf::from(home);
            possible_paths.push(home_path.join(".dx/icon/index"));
        }

        let index_dir = possible_paths
            .iter()
            .find(|p| p.exists())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Index not found in any of these locations:\n{}\n\nRun 'cargo run --release --bin build_index' in crates/media/icon/",
                    possible_paths.iter().map(|p| format!("  - {}", p.display())).collect::<Vec<_>>().join("\n")
                )
            })?;

        let index = IconIndex::load(index_dir)?;
        *engine_lock = Some(IconSearchEngine::from_index(index)?);
    }

    Ok(())
}

fn with_engine<F, R>(f: F) -> anyhow::Result<R>
where
    F: FnOnce(&IconSearchEngine) -> R,
{
    load_engine()?;
    let engine_lock = ENGINE.lock().unwrap();
    Ok(f(engine_lock.as_ref().unwrap()))
}

fn search_icons(query: &str, limit: usize) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let results = with_engine(|engine| engine.search(query, limit))?;
    let elapsed = start.elapsed();

    if results.is_empty() {
        println!("No results for '{}'", query);
        return Ok(());
    }

    println!(
        "Found {} results ({:.2}s):\n",
        results.len(),
        elapsed.as_secs_f64()
    );
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, result.icon.name, result.icon.pack);
    }

    Ok(())
}

fn export_icons(
    query: &str,
    output_dir: &PathBuf,
    limit: usize,
    pack_filter: Option<&str>,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let mut results = with_engine(|engine| engine.search(query, limit * 10))?;

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

    fs::create_dir_all(output_dir)?;

    for result in &results {
        let filename = format!("{}_{}.svg", result.icon.pack, result.icon.name);
        let filepath = output_dir.join(&filename);
        let svg_content = generate_svg(&result.icon.name, &result.icon.pack)?;
        fs::write(&filepath, svg_content)?;
        println!("✓ {}", filename);
    }

    println!(
        "\nExported {} icons ({:.2}s)",
        results.len(),
        start.elapsed().as_secs_f64()
    );
    Ok(())
}

fn export_to_desktop(icon_specs: &[&str]) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let desktop_icons_dir = PathBuf::from("apps/desktop/assets/icons");

    fs::create_dir_all(&desktop_icons_dir)?;

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
        let results = with_engine(|engine| engine.search(name, 100))?;

        let icon = results
            .iter()
            .find(|r| r.icon.pack == pack && r.icon.name == name)
            .or_else(|| results.first());

        if let Some(result) = icon {
            let filename = format!("{}.svg", result.icon.name);
            let filepath = desktop_icons_dir.join(&filename);
            let svg_content = generate_svg(&result.icon.name, &result.icon.pack)?;
            fs::write(&filepath, svg_content)?;
            println!("✓ {} ({})", result.icon.name, result.icon.pack);
            exported += 1;
        } else {
            eprintln!("✗ Not found: {}", spec);
            failed += 1;
        }
    }

    if exported > 0 {
        println!(
            "\nExported {} icons ({:.2}s)",
            exported,
            start.elapsed().as_secs_f64()
        );
    }
    if failed > 0 {
        eprintln!("Failed: {}", failed);
    }

    Ok(())
}

fn list_packs() -> anyhow::Result<()> {
    let results = with_engine(|engine| engine.search("a", 10000))?;

    let mut packs: Vec<String> = results
        .iter()
        .map(|r| r.icon.pack.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    packs.sort();

    println!("Available packs ({}):\n", packs.len());
    for pack in packs {
        println!("  {}", pack);
    }

    Ok(())
}

fn generate_svg(name: &str, pack: &str) -> anyhow::Result<String> {
    let mut possible_data_dirs = vec![];

    // Add DX_ICON_DATA env var path (highest priority)
    if let Ok(env_path) = std::env::var("DX_ICON_DATA") {
        possible_data_dirs.push(PathBuf::from(env_path));
    }

    // Add home directory path (second priority)
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let home_path = PathBuf::from(home);
        possible_data_dirs.push(home_path.join(".dx/icon/data"));
    }

    // Add executable directory paths
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            possible_data_dirs.push(parent.join("data"));
            possible_data_dirs.push(parent.join("../share/dx-icons/data"));
            possible_data_dirs.push(parent.join("../../crates/media/icon/data"));
        }
    }

    // Add relative paths (lowest priority)
    possible_data_dirs.push(PathBuf::from("crates/media/icon/data"));
    possible_data_dirs.push(PathBuf::from("data"));
    possible_data_dirs.push(PathBuf::from("../../data"));

    let data_dir = possible_data_dirs
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Data directory not found in any of these locations:\n{}\n\nSet DX_ICON_DATA env var or copy data/ to ~/.dx/icon/data",
                possible_data_dirs.iter().map(|p| format!("  - {}", p.display())).collect::<Vec<_>>().join("\n")
            )
        })?;

    let pack_file = data_dir.join(format!("{}.json", pack));

    if !pack_file.exists() {
        return Err(anyhow::anyhow!(
            "Pack '{}' not found at: {}\nData dir: {}",
            pack,
            pack_file.display(),
            data_dir.display()
        ));
    }

    let content = fs::read_to_string(&pack_file)?;
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
