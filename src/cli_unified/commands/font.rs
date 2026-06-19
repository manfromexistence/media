//! Font search and download commands

use anyhow::Result;
use console::style;
use std::path::{Path, PathBuf};

use crate::cli_unified::args::{FontCommands, OutputFormat};
use crate::cli_unified::config::MediaConfig;
use crate::cli_unified::output::{
    print_info, print_success, print_table_header, print_table_row, truncate,
};

pub async fn execute_font_command(
    command: FontCommands,
    format: &OutputFormat,
    config: &MediaConfig,
) -> Result<()> {
    match command {
        FontCommands::Search {
            query,
            limit,
            provider,
            category,
        } => {
            cmd_search(
                &query,
                limit,
                provider.as_deref(),
                category.as_deref(),
                format,
            )
            .await
        }
        FontCommands::Download {
            font_id,
            provider,
            mut output,
            formats,
            subsets,
        } => {
            // Use config directory if output is default
            if output == PathBuf::from("./fonts") {
                output = config.get_font_dir();
                config.ensure_dir(&output)?;
            }

            // Use config defaults for formats and subsets if not specified
            let formats = if formats.is_empty() {
                config.font_formats.clone()
            } else {
                formats
            };
            let subsets = if subsets.is_empty() {
                config.font_subsets.clone()
            } else {
                subsets
            };

            cmd_download(&font_id, &provider, &output, &formats, &subsets).await
        }
        FontCommands::List {
            provider,
            limit,
            category,
        } => cmd_list(&provider, limit, category.as_deref(), format).await,
        FontCommands::Info { font_id, provider } => cmd_info(&font_id, &provider, format).await,
        FontCommands::Stats => cmd_stats(format).await,
        FontCommands::Providers => cmd_providers().await,
    }
}

async fn cmd_search(
    query: &str,
    limit: usize,
    _provider: Option<&str>,
    _category: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    print_info(&format!("🔍 Searching fonts for '{}'...", query));

    let search = dx_font::FontSearch::new()?;
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
            print_table_header(&["Name", "Provider", "Category", "Variants"]);

            for font in &fonts {
                let category = font
                    .category
                    .as_ref()
                    .map(|c| format!("{:?}", c))
                    .unwrap_or_else(|| "-".to_string());

                print_table_row(&[
                    truncate(&font.name, 28),
                    font.provider.name().to_string(),
                    category,
                    font.variant_count.to_string(),
                ]);
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
    print_info(&format!("📥 Downloading font: {}", font_id));

    // Search for font first to get exact ID
    let search = dx_font::FontSearch::new()?;
    let results = search.search(font_id).await?;
    let font = results
        .fonts
        .iter()
        .find(|f| f.name.to_lowercase() == font_id.to_lowercase())
        .or_else(|| results.fonts.first())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Font '{}' not found. Try 'media font search \"{}\"' first.",
                font_id,
                font_id
            )
        })?;

    print_info(&format!("Found: {} ({})", font.name, font.id));

    let downloader = dx_font::FontDownloader::new()?;

    let formats_ref: Vec<&str> = formats.iter().map(|s| s.as_str()).collect();
    let subsets_ref: Vec<&str> = subsets.iter().map(|s| s.as_str()).collect();

    match provider.to_lowercase().as_str() {
        "google" | "google-fonts" | "googlefonts" => {
            let path = downloader
                .download_google_font(&font.id, output, &formats_ref, &subsets_ref)
                .await?;

            print_success(&format!("✅ Downloaded to: {}", path.display()));
        }
        "fontsource" => {
            let path = downloader
                .download_fontsource_font(font_id, output, 400, "normal")
                .await?;

            print_success(&format!("Downloaded to: {}", path.display()));
        }
        _ => {
            let provider_enum = match provider.to_lowercase().as_str() {
                "bunny" | "bunny-fonts" => dx_font::FontProvider::BunnyFonts,
                "fontshare" => dx_font::FontProvider::FontShare,
                _ => dx_font::FontProvider::GoogleFonts,
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
                print_success(&format!("Downloaded to: {}", path.display()));
            }
        }
    }

    Ok(())
}

async fn cmd_list(
    _provider: &str,
    limit: Option<usize>,
    _category: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    print_info("📋 Listing fonts...");

    let search = dx_font::FontSearch::new()?;
    let results = search.list_all().await?;

    let mut fonts = results.fonts;

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
            print_table_header(&["Name", "Provider", "Category", "Variants"]);

            for font in fonts.iter().take(100) {
                let category = font
                    .category
                    .as_ref()
                    .map(|c| format!("{:?}", c))
                    .unwrap_or_else(|| "-".to_string());

                print_table_row(&[
                    truncate(&font.name, 28),
                    font.provider.name().to_string(),
                    category,
                    font.variant_count.to_string(),
                ]);
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
    print_info(&format!("ℹ️  Getting font info: {}", font_id));

    let search = dx_font::FontSearch::new()?;

    // First search for the font to get exact ID
    let results = search.search(font_id).await?;
    let font = results
        .fonts
        .iter()
        .find(|f| f.name.to_lowercase() == font_id.to_lowercase())
        .or_else(|| results.fonts.first())
        .ok_or_else(|| anyhow::anyhow!("Font '{}' not found", font_id))?;

    let provider_enum = match provider.to_lowercase().as_str() {
        "google" | "google-fonts" => dx_font::FontProvider::GoogleFonts,
        "bunny" | "bunny-fonts" => dx_font::FontProvider::BunnyFonts,
        "fontsource" => dx_font::FontProvider::Fontsource,
        "fontshare" => dx_font::FontProvider::FontShare,
        _ => dx_font::FontProvider::GoogleFonts,
    };

    let family = search.get_font_details(&provider_enum, &font.id).await?;

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
    print_info("📊 Gathering font statistics...");

    let search = dx_font::FontSearch::new()?;
    let stats = search.get_stats().await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        _ => {
            println!("\n{}", style("Font Statistics").green().bold());
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
