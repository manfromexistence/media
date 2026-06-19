//! Scrape command implementation.

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::args::{MediaTypeArg, OutputFormat, ScrapeArgs};
use crate::config::Config;
use crate::engine::{ScrapeOptions, Scraper};
use crate::error::Result;
use crate::types::MediaType;

/// Execute the scrape command.
pub async fn execute(args: ScrapeArgs, format: OutputFormat, quiet: bool) -> Result<()> {
    let scraper = Scraper::new()?.with_depth(args.depth);

    // Build scrape options
    let media_types = match args.media_type {
        MediaTypeArg::Image => vec![MediaType::Image],
        MediaTypeArg::Video => vec![MediaType::Video],
        MediaTypeArg::Audio => vec![MediaType::Audio],
        MediaTypeArg::Gif => vec![MediaType::Gif],
        MediaTypeArg::Vector => vec![MediaType::Vector],
        MediaTypeArg::Document => vec![MediaType::Document],
        MediaTypeArg::Model3D => vec![MediaType::Model3D],
        MediaTypeArg::All => vec![
            MediaType::Image,
            MediaType::Video,
            MediaType::Audio,
            MediaType::Gif,
            MediaType::Vector,
            MediaType::Document,
            MediaType::Model3D,
        ],
    };

    let options = ScrapeOptions {
        max_depth: args.depth,
        pattern: args.pattern.clone(),
        media_types,
        max_assets: args.count,
    };

    // Show progress
    let spinner = if !quiet {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        spinner.set_message(format!("Scraping {}...", args.url));
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(spinner)
    } else {
        None
    };

    // Perform scrape
    let result = scraper.scrape(&args.url, &options).await?;

    if let Some(spinner) = spinner {
        spinner.finish_and_clear();
    }

    // Output results based on format
    match format {
        OutputFormat::Json | OutputFormat::JsonCompact => {
            let output = serde_json::json!({
                "source_url": result.source_url,
                "pages_scraped": result.pages_scraped,
                "assets_found": result.assets.len(),
                "assets": result.assets.iter().map(|a| serde_json::json!({
                    "id": a.id,
                    "title": a.title,
                    "media_type": format!("{:?}", a.media_type),
                    "download_url": a.download_url,
                    "source_url": a.source_url,
                    "width": a.width,
                    "height": a.height,
                })).collect::<Vec<_>>(),
                "errors": result.errors,
            });
            if matches!(format, OutputFormat::JsonCompact) {
                println!("{}", serde_json::to_string(&output)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
        }
        OutputFormat::Tsv => {
            println!("id\ttitle\ttype\tdownload_url");
            for asset in &result.assets {
                println!(
                    "{}\t{}\t{:?}\t{}",
                    asset.id, asset.title, asset.media_type, asset.download_url
                );
            }
        }
        OutputFormat::Text => {
            if !quiet {
                println!(
                    "\n{} Found {} media assets from {}",
                    "✓".green(),
                    result.assets.len().to_string().cyan(),
                    result.source_url.dimmed()
                );
                println!("  Pages scraped: {}", result.pages_scraped);

                if !result.errors.is_empty() {
                    println!(
                        "\n{} {} errors encountered:",
                        "⚠".yellow(),
                        result.errors.len()
                    );
                    for err in &result.errors {
                        println!("  {} {}", "•".dimmed(), err.dimmed());
                    }
                }
                println!();
            }

            for (i, asset) in result.assets.iter().enumerate() {
                if !quiet {
                    let type_badge = match asset.media_type {
                        MediaType::Image => "IMG".on_blue(),
                        MediaType::Video => "VID".on_magenta(),
                        MediaType::Audio => "AUD".on_green(),
                        MediaType::Gif => "GIF".on_yellow(),
                        MediaType::Vector => "SVG".on_cyan(),
                        MediaType::Document => "DOC".on_white(),
                        MediaType::Data => "DAT".on_bright_black(),
                        MediaType::Model3D => "3D".on_red(),
                        MediaType::Code => "COD".on_bright_green(),
                        MediaType::Text => "TXT".on_bright_white(),
                    };

                    println!(
                        "{:>3}. {} {}",
                        i + 1,
                        type_badge,
                        if asset.title.is_empty() {
                            &asset.id
                        } else {
                            &asset.title
                        }
                        .white()
                    );

                    if let (Some(w), Some(h)) = (asset.width, asset.height) {
                        println!("     {} {}x{}", "Size:".dimmed(), w, h);
                    }
                    println!("     {} {}", "URL:".dimmed(), asset.download_url.dimmed());
                } else {
                    println!("{}", asset.download_url);
                }
            }
        }
    }

    // Download if not dry-run
    if !args.dry_run && !result.assets.is_empty() {
        use crate::engine::Downloader;
        use std::path::PathBuf;

        let output_dir = PathBuf::from(&args.output);
        std::fs::create_dir_all(&output_dir)?;

        let config = Config::load()?;
        let downloader = Downloader::new(&config).with_download_dir(&output_dir);

        if !quiet {
            println!(
                "\n{} Downloading {} assets...",
                "↓".cyan(),
                result.assets.len()
            );
        }

        let pb = if !quiet {
            let pb = ProgressBar::new(result.assets.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("█▓▒░"),
            );
            Some(pb)
        } else {
            None
        };

        let mut downloaded = 0;
        for asset in &result.assets {
            if let Some(ref pb) = pb {
                let msg = if asset.title.is_empty() {
                    asset.id.clone()
                } else {
                    asset.title.clone()
                };
                pb.set_message(msg);
            }

            match downloader.download(asset).await {
                Ok(path) => {
                    downloaded += 1;
                    if !quiet && pb.is_none() {
                        println!("  {} {}", "✓".green(), path.display());
                    }
                }
                Err(e) => {
                    if !quiet {
                        eprintln!("  {} Failed: {}", "✗".red(), e);
                    }
                }
            }

            if let Some(ref pb) = pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_and_clear();
        }

        if !quiet {
            println!(
                "\n{} Downloaded {} of {} assets to {}",
                "✓".green(),
                downloaded.to_string().cyan(),
                result.assets.len(),
                output_dir.display()
            );
        }
    } else if args.dry_run && !quiet && !result.assets.is_empty() {
        println!("\n{} Dry run - no files downloaded", "ℹ".blue());
    }

    Ok(())
}
