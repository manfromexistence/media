//! Media search and download commands

use anyhow::Result;
use console::style;
use std::path::Path;

use crate::cli_unified::args::OutputFormat;
use crate::cli_unified::output::{
    print_info, print_json, print_success, print_table_header, print_table_row,
};
use crate::http::validate_url;
use crate::providers::listing::{credential_status, provider_json_row, source_kind};
use crate::{DxMedia, MediaType};

pub async fn cmd_search(
    query: &str,
    media_type: &str,
    provider: Option<&str>,
    limit: usize,
    format: &OutputFormat,
) -> Result<()> {
    if let Some(message) = search_status_message(query, format) {
        print_info(&message);
    }

    let dx = DxMedia::new()?;
    let mut search = dx.search(query);

    if let Some(media_type) = parse_search_media_type(media_type)? {
        search = search.media_type(media_type);
    }

    // Set provider if specified
    if let Some(p) = provider {
        search = search.provider(p);
    }

    let results = search.execute().await?;

    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&results.with_asset_provenance())?
            );
        }
        OutputFormat::Table => {
            println!(
                "\n{}",
                style(format!("Found {} assets", results.total_count)).green()
            );
            print_table_header(&["Title", "Provider", "Type", "License"]);

            for asset in results.assets.iter().take(limit) {
                print_table_row(&[
                    asset.title.clone(),
                    asset.provider.clone(),
                    format!("{:?}", asset.media_type),
                    format!("{:?}", asset.license),
                ]);
            }
            println!("{}", "â”€".repeat(80));
        }
        OutputFormat::Simple => {
            for asset in results.assets.iter().take(limit) {
                println!("{} ({})", asset.title, asset.provider);
            }
        }
    }

    Ok(())
}

fn search_status_message(query: &str, format: &OutputFormat) -> Option<String> {
    if matches!(format, OutputFormat::Json) {
        None
    } else {
        Some(format!("Searching for {}...", query))
    }
}

fn parse_search_media_type(value: &str) -> Result<Option<MediaType>> {
    let media_type = match value.trim().to_ascii_lowercase().as_str() {
        "" | "all" | "*" => return Ok(None),
        "image" | "images" | "img" => MediaType::Image,
        "video" | "videos" | "vid" => MediaType::Video,
        "audio" | "audios" | "sound" | "sounds" => MediaType::Audio,
        "gif" | "gifs" => MediaType::Gif,
        "vector" | "vectors" | "svg" => MediaType::Vector,
        "document" | "documents" | "doc" | "docs" => MediaType::Document,
        "data" | "dataset" | "datasets" => MediaType::Data,
        "model3d" | "model" | "models" | "3d" => MediaType::Model3D,
        "code" | "source" => MediaType::Code,
        "text" | "texts" => MediaType::Text,
        other => {
            anyhow::bail!(
                "Unknown media type '{other}'. Valid types: all, image, video, audio, gif, vector, document, data, model3d, code, text"
            );
        }
    };

    Ok(Some(media_type))
}

#[cfg(test)]
mod tests {
    use super::{
        cmd_download, direct_url_filename, download_status_message, infer_media_type_from_path,
        parse_search_media_type, providers_status_message, search_status_message,
    };
    use crate::MediaType;
    use crate::cli_unified::args::OutputFormat;

    #[test]
    fn json_search_status_message_is_suppressed() {
        assert!(search_status_message("nebula", &OutputFormat::Json).is_none());
    }

    #[test]
    fn non_json_search_status_message_is_preserved() {
        let message = search_status_message("nebula", &OutputFormat::Table)
            .expect("table output keeps a human status line");

        assert!(message.contains("nebula"));
    }

    #[test]
    fn search_media_type_all_leaves_query_unfiltered() {
        assert_eq!(
            parse_search_media_type("all").expect("all should parse"),
            None
        );
    }

    #[test]
    fn search_media_type_supports_non_image_types() {
        assert_eq!(
            parse_search_media_type("vector").expect("vector should parse"),
            Some(MediaType::Vector)
        );
        assert_eq!(
            parse_search_media_type("gif").expect("gif should parse"),
            Some(MediaType::Gif)
        );
        assert_eq!(
            parse_search_media_type("document").expect("document should parse"),
            Some(MediaType::Document)
        );
        assert_eq!(
            parse_search_media_type("3d").expect("3d should parse"),
            Some(MediaType::Model3D)
        );
    }

    #[test]
    fn search_media_type_rejects_unknown_values() {
        let error = parse_search_media_type("banana")
            .expect_err("unknown media type should not silently default to image");

        assert!(error.to_string().contains("Unknown media type"));
    }

    #[tokio::test]
    async fn provider_asset_download_errors_until_lookup_is_wired() {
        let output = tempfile::tempdir().expect("temporary output dir should be created");
        let error = cmd_download(
            "asset-123",
            output.path(),
            Some("openverse"),
            &OutputFormat::Table,
        )
        .await
        .expect_err("asset-id download should fail until provider lookup is wired");

        assert!(
            error
                .to_string()
                .contains("Provider asset ID downloads are not implemented")
        );
    }

    #[test]
    fn direct_url_filename_preserves_url_path_filename() {
        let filename = direct_url_filename("https://example.com/assets/photo.png?size=large")
            .expect("URL path filename should be parsed");

        assert_eq!(filename, "photo.png");
    }

    #[test]
    fn direct_url_filename_requires_a_url_path_filename() {
        let error = direct_url_filename("https://example.com/")
            .expect_err("directory URL should not imply a validated output filename");

        assert!(
            error
                .to_string()
                .contains("require a filename in the URL path")
        );
    }

    #[test]
    fn direct_url_media_type_requires_recognized_extension() {
        assert_eq!(
            infer_media_type_from_path("photo.png"),
            Some(MediaType::Image)
        );
        assert_eq!(infer_media_type_from_path("download"), None);
    }

    #[test]
    fn json_download_status_message_is_suppressed() {
        assert!(
            download_status_message("https://example.com/photo.png", &OutputFormat::Json).is_none()
        );
    }

    #[test]
    fn json_providers_status_message_is_suppressed() {
        assert!(providers_status_message(&OutputFormat::Json).is_none());
    }

    #[test]
    fn non_json_providers_status_message_is_preserved() {
        let message = providers_status_message(&OutputFormat::Simple)
            .expect("simple output keeps a human status line");

        assert!(message.contains("Available Providers"));
    }
}

pub async fn cmd_download(
    asset_id: &str,
    output: &Path,
    _provider: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    if let Some(message) = download_status_message(asset_id, format) {
        print_info(&message);
    }

    let dx = DxMedia::new()?;

    // If it's a URL, download directly
    if asset_id.starts_with("http://") || asset_id.starts_with("https://") {
        validate_url(asset_id)?;
        let filename = direct_url_filename(asset_id)?;
        let media_type = infer_media_type_from_path(&filename).ok_or_else(|| {
            anyhow::anyhow!(
                "Direct URL downloads require a recognized media file extension so type validation can be recorded: {}",
                filename
            )
        })?;
        let filepath = output.join(&filename);
        let receipt = dx
            .downloader()
            .download_url_to_with_receipt(asset_id, &filepath, media_type, None)
            .await?;
        match format {
            OutputFormat::Json => print_json(&receipt)?,
            OutputFormat::Table | OutputFormat::Simple => {
                let receipt_completeness = receipt
                    .metadata
                    .get("tool.receipt_completeness")
                    .map(String::as_str)
                    .unwrap_or("unknown");
                let source_kind = receipt
                    .metadata
                    .get("tool.source_kind")
                    .map(String::as_str)
                    .unwrap_or("unknown");
                let type_validation = receipt
                    .metadata
                    .get("tool.type_validation")
                    .map(String::as_str)
                    .unwrap_or("unknown");
                print_success(&format!(
                    "Downloaded to: {} (source={}, receipt={}, type_validation={})",
                    filepath.display(),
                    source_kind,
                    receipt_completeness,
                    type_validation
                ));
            }
        }
        return Ok(());
    }

    // Otherwise, need provider
    let provider_name = _provider.ok_or_else(|| {
        anyhow::anyhow!("Provider required when using asset ID. Use --provider <name>")
    })?;

    anyhow::bail!(
        "Provider asset ID downloads are not implemented yet for provider '{}'; use a direct URL until provider-backed lookup can return a tool receipt",
        provider_name
    );
}

fn download_status_message(asset_id: &str, format: &OutputFormat) -> Option<String> {
    if matches!(format, OutputFormat::Json) {
        None
    } else {
        Some(format!("Downloading asset: {}", asset_id))
    }
}

fn direct_url_filename(asset_url: &str) -> Result<String> {
    let parsed = url::Url::parse(asset_url)?;
    let filename = parsed
        .path_segments()
        .and_then(std::iter::Iterator::last)
        .filter(|segment| !segment.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Direct URL downloads require a filename in the URL path so the output can be validated"
            )
        })?;

    Ok(filename.to_string())
}

fn infer_media_type_from_path(path: &str) -> Option<MediaType> {
    let extension = path
        .split('?')
        .next()
        .and_then(|path| path.rsplit('.').next())
        .map(str::to_ascii_lowercase)?;

    MediaType::all()
        .iter()
        .copied()
        .find(|media_type| media_type.matches_extension(&extension))
}

pub async fn cmd_providers(provider_type: &str, format: &OutputFormat) -> Result<()> {
    validate_provider_type(provider_type)?;

    if let Some(message) = providers_status_message(format) {
        print_info(message);
    }

    let show_media_requested = provider_type == "all" || provider_type == "media";

    if show_media_requested {
        let dx = DxMedia::new()?;
        let mut providers = dx.registry().all();
        providers.sort_by_key(|provider| provider.name());

        match format {
            OutputFormat::Json => {
                let rows: Vec<_> = providers
                    .iter()
                    .map(|provider| {
                        let mut row = provider_json_row(provider.as_ref(), "supported_media_types");
                        row["base_url"] = serde_json::json!(provider.base_url());
                        row
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&rows)?);
                return Ok(());
            }
            OutputFormat::Table => {
                print_table_header(&[
                    "Provider",
                    "Status",
                    "Source",
                    "Credentials",
                    "Types",
                    "Base URL",
                ]);
                for provider in providers {
                    let status = if provider.is_available() {
                        "available"
                    } else if provider.requires_api_key() {
                        "missing credentials"
                    } else {
                        "unavailable"
                    };
                    let media_types = provider
                        .supported_media_types()
                        .iter()
                        .map(MediaType::as_str)
                        .collect::<Vec<_>>()
                        .join(",");

                    print_table_row(&[
                        provider.display_name().to_string(),
                        status.to_string(),
                        source_kind(provider.as_ref()).to_string(),
                        credential_status(provider.as_ref()).to_string(),
                        media_types,
                        provider.base_url().to_string(),
                    ]);
                }
            }
            OutputFormat::Simple => {
                for provider in providers {
                    let status = if provider.is_available() {
                        "available"
                    } else if provider.requires_api_key() {
                        "missing credentials"
                    } else {
                        "unavailable"
                    };
                    println!(
                        "{}\t{}\t{}\t{}",
                        provider.name(),
                        status,
                        source_kind(provider.as_ref()),
                        credential_status(provider.as_ref())
                    );
                }
            }
        }

        if provider_type == "media" {
            return Ok(());
        }
    }

    let show_media = false;
    let show_icon = provider_type == "all" || provider_type == "icon";
    let show_font = provider_type == "all" || provider_type == "font";

    if show_media {
        println!("{}", style("Media Providers").yellow().bold());
        println!(
            "â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€"
        );
        println!("  â€¢ Openverse         700M+ images/audio  https://openverse.org");
        println!("  â€¢ Wikimedia         90M+ images         https://commons.wikimedia.org");
        println!("  â€¢ NASA Images       140K+ images        https://images.nasa.gov");
        println!("  â€¢ Met Museum        470K+ images        https://metmuseum.org");
        println!("  â€¢ Rijksmuseum       700K+ images        https://rijksmuseum.nl");
        println!("  â€¢ Cleveland Museum  36K+ images         https://clevelandart.org");
        println!("  â€¢ Library Congress  25M+ items          https://loc.gov");
        println!("  â€¢ DPLA              40M+ items          https://dp.la");
        println!("  â€¢ Europeana         50M+ items          https://europeana.eu");
        println!("  â€¢ Lorem Picsum      Provider-backed images  https://picsum.photos");
        println!("  â€¢ Poly Haven        3D assets           https://polyhaven.com");
        println!();
        println!(
            "{}",
            style("Media Providers Requiring Credentials")
                .yellow()
                .bold()
        );
        println!(
            "â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€"
        );
        println!("  â€¢ Unsplash          5M+ photos          https://unsplash.com");
        println!("  â€¢ Pexels            3.5M+ photos/videos https://pexels.com");
        println!("  â€¢ Pixabay           4.2M+ images/videos https://pixabay.com");
        println!("  â€¢ Giphy             Millions of GIFs    https://giphy.com");
        println!("  â€¢ Freesound         600K+ sounds        https://freesound.org");
        println!("  â€¢ Smithsonian       4.5M+ images        https://si.edu");
        println!();
    }

    if show_icon {
        println!("{}", style("Icon Providers").yellow().bold());
        println!(
            "â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€"
        );
        println!("  â€¢ 200+ icon packs with 100K+ icons");
        println!("  â€¢ Lucide, Solar, Material, FontAwesome, Heroicons, and more");
        println!("  â€¢ Use: media icon packs");
        println!();
    }

    if show_font {
        println!("{}", style("Font Providers").yellow().bold());
        println!(
            "â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€"
        );
        println!("  â€¢ Google Fonts      1,562 fonts        https://fonts.google.com");
        println!("  â€¢ Bunny Fonts       1,478 fonts        https://fonts.bunny.net");
        println!("  â€¢ Fontsource        1,562 fonts        https://fontsource.org");
        println!("  â€¢ Font Squirrel     1,082 fonts        https://fontsquirrel.com");
        println!("  â€¢ DaFont            8,500 fonts        https://dafont.com");
        println!("  â€¢ FontShare         100 fonts          https://fontshare.com");
        println!();
    }

    println!("{}", "â•".repeat(70));

    Ok(())
}

fn validate_provider_type(provider_type: &str) -> Result<()> {
    if matches!(provider_type, "all" | "media" | "icon" | "font") {
        return Ok(());
    }

    anyhow::bail!(
        "Unknown provider type '{}'. Valid provider types: all, media, icon, font",
        provider_type
    );
}

fn providers_status_message(format: &OutputFormat) -> Option<&'static str> {
    if matches!(format, OutputFormat::Json) {
        None
    } else {
        Some("Available Providers\n")
    }
}

pub async fn cmd_health() -> Result<()> {
    print_info("ðŸ¥ Checking provider health...\n");

    let dx = DxMedia::new()?;
    let health = dx.health_check().await;

    println!("{}", style("Provider Health Status").green().bold());
    println!("{}", "â”€".repeat(50));

    for result in &health.providers {
        let status = if result.available {
            style("âœ… OK").green()
        } else {
            style("âŒ Error").red()
        };
        println!("{:<30} {}", result.provider, status);
    }

    println!("{}", "â”€".repeat(50));
    println!(
        "Healthy: {} / {}",
        health.providers.iter().filter(|r| r.available).count(),
        health.providers.len()
    );

    Ok(())
}
