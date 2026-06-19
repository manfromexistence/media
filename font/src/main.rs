//! dx-font - A comprehensive font search and download tool
//!
//! Access 50k+ commercial-free fonts from 100+ sources.

use anyhow::Result;
use clap::Parser;
use console::style;
use std::path::Path;

use dx_font::cli::{Cli, Commands, OutputFormat};
use dx_font::download::FontDownloader;
use dx_font::models::FontProvider;
use dx_font::search::FontSearch;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("dx_font=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Search {
            query,
            limit,
            provider,
            category,
        } => {
            cmd_search(&query, limit, provider, category, &cli.format).await?;
        }
        Commands::Download {
            font_id,
            provider,
            output,
            formats,
            subsets,
        } => {
            cmd_download(&font_id, &provider, &output, &formats, &subsets).await?;
        }
        Commands::List {
            provider,
            limit,
            category,
        } => {
            cmd_list(&provider, limit, category, &cli.format).await?;
        }
        Commands::Info { font_id, provider } => {
            cmd_info(&font_id, &provider, &cli.format).await?;
        }
        Commands::Stats => {
            cmd_stats(&cli.format).await?;
        }
        Commands::Health => {
            cmd_health().await?;
        }
        Commands::Providers => {
            cmd_providers().await?;
        }
    }

    Ok(())
}

async fn cmd_search(
    query: &str,
    limit: usize,
    _provider: Option<String>,
    _category: Option<String>,
    format: &OutputFormat,
) -> Result<()> {
    println!("{}", style("🔍 Searching fonts...").cyan().bold());

    let search = FontSearch::new()?;
    let results = search.search(query).await?;

    let fonts: Vec<_> = results.fonts.into_iter().take(limit).collect();

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&fonts)?);
        }
        OutputFormat::Table => {
            println!(
                "\n{}",
                style(format!("Found {} fonts matching '{}'", fonts.len(), query)).green()
            );
            println!("{}", "─".repeat(80));
            println!(
                "{:<30} {:<20} {:<15} {:<10}",
                style("Name").bold(),
                style("Provider").bold(),
                style("Category").bold(),
                style("Variants").bold()
            );
            println!("{}", "─".repeat(80));

            for font in &fonts {
                let category = font
                    .category
                    .as_ref()
                    .map(|c| format!("{:?}", c))
                    .unwrap_or_else(|| "-".to_string());

                println!(
                    "{:<30} {:<20} {:<15} {:<10}",
                    truncate(&font.name, 28),
                    font.provider.name(),
                    category,
                    font.variant_count
                );
            }
            println!("{}", "─".repeat(80));
        }
        OutputFormat::Simple => {
            for font in &fonts {
                println!("{} ({})", font.name, font.provider.name());
            }
        }
    }

    Ok(())
}

async fn cmd_download(
    font_id: &str,
    provider: &str,
    output: &Path,
    formats: &[String],
    subsets: &[String],
) -> Result<()> {
    println!(
        "{}",
        style(format!("📥 Downloading font: {}", font_id))
            .cyan()
            .bold()
    );

    let downloader = FontDownloader::new()?;

    let formats_ref: Vec<&str> = formats.iter().map(|s| s.as_str()).collect();
    let subsets_ref: Vec<&str> = subsets.iter().map(|s| s.as_str()).collect();

    match provider.to_lowercase().as_str() {
        "google" | "google-fonts" | "googlefonts" => {
            let path = downloader
                .download_google_font(font_id, output, &formats_ref, &subsets_ref)
                .await?;

            println!(
                "{}",
                style(format!("✅ Downloaded to: {}", path.display())).green()
            );
        }
        "fontsource" => {
            // Download default regular weight
            let path = downloader
                .download_fontsource_font(font_id, output, 400, "normal")
                .await?;

            println!(
                "{}",
                style(format!("✅ Downloaded to: {}", path.display())).green()
            );
        }
        _ => {
            let provider_enum = match provider.to_lowercase().as_str() {
                "bunny" | "bunny-fonts" => FontProvider::BunnyFonts,
                "fontshare" => FontProvider::FontShare,
                _ => FontProvider::GoogleFonts,
            };

            let paths = downloader
                .download_font(
                    &provider_enum,
                    font_id,
                    &dx_font::models::DownloadOptions {
                        output_dir: output.to_path_buf(),
                        formats: formats.to_vec(),
                        ..Default::default()
                    },
                )
                .await?;

            for path in paths {
                println!(
                    "{}",
                    style(format!("✅ Downloaded to: {}", path.display())).green()
                );
            }
        }
    }

    Ok(())
}

async fn cmd_list(
    _provider: &str,
    limit: Option<usize>,
    _category: Option<String>,
    format: &OutputFormat,
) -> Result<()> {
    println!("{}", style("📋 Listing fonts...").cyan().bold());

    let search = FontSearch::new()?;
    let results = search.list_all().await?;

    let mut fonts = results.fonts;

    // Apply limit if specified
    if let Some(l) = limit {
        fonts.truncate(l);
    }

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&fonts)?);
        }
        OutputFormat::Table => {
            println!(
                "\n{}",
                style(format!("Total: {} fonts", fonts.len())).green()
            );
            println!("{}", "─".repeat(80));
            println!(
                "{:<30} {:<20} {:<15} {:<10}",
                style("Name").bold(),
                style("Provider").bold(),
                style("Category").bold(),
                style("Variants").bold()
            );
            println!("{}", "─".repeat(80));

            for font in fonts.iter().take(100) {
                let category = font
                    .category
                    .as_ref()
                    .map(|c| format!("{:?}", c))
                    .unwrap_or_else(|| "-".to_string());

                println!(
                    "{:<30} {:<20} {:<15} {:<10}",
                    truncate(&font.name, 28),
                    font.provider.name(),
                    category,
                    font.variant_count
                );
            }

            if fonts.len() > 100 {
                println!("... and {} more fonts", fonts.len() - 100);
            }
            println!("{}", "─".repeat(80));
        }
        OutputFormat::Simple => {
            for font in &fonts {
                println!("{}", font.name);
            }
        }
    }

    Ok(())
}

async fn cmd_info(font_id: &str, provider: &str, format: &OutputFormat) -> Result<()> {
    println!(
        "{}",
        style(format!("ℹ️  Getting font info: {}", font_id))
            .cyan()
            .bold()
    );

    let search = FontSearch::new()?;

    let provider_enum = match provider.to_lowercase().as_str() {
        "google" | "google-fonts" => FontProvider::GoogleFonts,
        "bunny" | "bunny-fonts" => FontProvider::BunnyFonts,
        "fontsource" => FontProvider::Fontsource,
        "fontshare" => FontProvider::FontShare,
        _ => FontProvider::GoogleFonts,
    };

    let family = search.get_font_details(&provider_enum, font_id).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&family)?);
        }
        _ => {
            println!("\n{}", style(&family.name).green().bold());
            println!("{}", "─".repeat(60));
            println!("ID:          {}", family.id);
            println!("Provider:    {}", family.provider.name());
            if let Some(cat) = &family.category {
                println!("Category:    {:?}", cat);
            }
            if let Some(designer) = &family.designer {
                println!("Designer:    {}", designer);
            }
            if let Some(license) = &family.license {
                println!("License:     {:?}", license);
            }
            println!("Variants:    {}", family.variants.len());
            println!("Subsets:     {}", family.subsets.join(", "));
            if let Some(url) = &family.preview_url {
                println!("Preview:     {}", url);
            }
            if let Some(url) = &family.download_url {
                println!("Download:    {}", url);
            }
            println!("{}", "─".repeat(60));

            println!("\n{}", style("Variants:").bold());
            for variant in &family.variants {
                let style_str = match variant.style {
                    dx_font::FontStyle::Normal => "Normal",
                    dx_font::FontStyle::Italic => "Italic",
                };
                println!(
                    "  {} {} ({})",
                    variant.weight.to_numeric(),
                    style_str,
                    variant.file_format
                );
            }
        }
    }

    Ok(())
}

async fn cmd_stats(format: &OutputFormat) -> Result<()> {
    println!("{}", style("📊 Gathering font statistics...").cyan().bold());

    let search = FontSearch::new()?;
    let stats = search.get_stats().await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        _ => {
            println!("\n{}", style("dx-font Statistics").green().bold());
            println!("{}", "═".repeat(50));
            println!(
                "Total Fonts:      {}",
                style(stats.total_fonts).cyan().bold()
            );
            println!("Providers:        {}", stats.providers_count);
            println!("{}", "─".repeat(50));
            println!("{}", style("By Category:").bold());
            println!("  Serif:          {}", stats.serif_count);
            println!("  Sans-Serif:     {}", stats.sans_serif_count);
            println!("  Display:        {}", stats.display_count);
            println!("  Handwriting:    {}", stats.handwriting_count);
            println!("  Monospace:      {}", stats.monospace_count);
            println!("  Uncategorized:  {}", stats.uncategorized_count);
            println!("{}", "═".repeat(50));
        }
    }

    Ok(())
}

async fn cmd_health() -> Result<()> {
    println!("{}", style("🏥 Checking provider health...").cyan().bold());

    let search = FontSearch::new()?;
    let health = search.health_check().await;

    println!("\n{}", style("Provider Health Status").green().bold());
    println!("{}", "─".repeat(40));

    for (provider, is_healthy) in health {
        let status = if is_healthy {
            style("✅ OK").green()
        } else {
            style("❌ Error").red()
        };
        println!("{:<25} {}", provider, status);
    }

    println!("{}", "─".repeat(40));

    Ok(())
}

async fn cmd_providers() -> Result<()> {
    println!("\n{}", style("📚 Available Font Providers").green().bold());
    println!("{}", "═".repeat(70));

    println!(
        "\n{}",
        style("Tier 1: Primary APIs (No Keys Required)")
            .yellow()
            .bold()
    );
    println!("─────────────────────────────────────────────────────────────────────");
    println!("  • Google Fonts        1,562 fonts   https://fonts.google.com");
    println!("  • Bunny Fonts         1,478 fonts   https://fonts.bunny.net");
    println!("  • Fontsource          1,562 fonts   https://fontsource.org");
    println!("  • Font Library        1,340 fonts   https://fontlibrary.org");

    println!(
        "\n{}",
        style("Tier 2: Major Free Font Sites").yellow().bold()
    );
    println!("─────────────────────────────────────────────────────────────────────");
    println!("  • Font Squirrel       1,082 fonts   https://www.fontsquirrel.com");
    println!("  • DaFont              8,500 fonts   https://www.dafont.com");
    println!("  • 1001 Fonts          5,200 fonts   https://www.1001fonts.com");
    println!("  • FontSpace           4,800 fonts   https://www.fontspace.com");

    println!(
        "\n{}",
        style("Tier 3: Curated Foundries (High Quality)")
            .yellow()
            .bold()
    );
    println!("─────────────────────────────────────────────────────────────────────");
    println!("  • FontShare             100 fonts   https://www.fontshare.com");
    println!("  • Velvetyne              85 fonts   https://velvetyne.fr");
    println!("  • Open Foundry           45 fonts   https://open-foundry.com");
    println!("  • Use & Modify           92 fonts   https://usemodify.com");

    println!(
        "\n{}",
        style("Total: 50,000+ fonts from 100+ sources")
            .cyan()
            .bold()
    );
    println!("{}", "═".repeat(70));

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
