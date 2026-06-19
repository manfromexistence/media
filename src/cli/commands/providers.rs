//! Providers command implementation.

use colored::Colorize;

use crate::DxMedia;
use crate::cli::args::{OutputFormat, ProvidersArgs};
use crate::error::Result;
use crate::providers::listing::{credential_status, provider_json_row, source_kind};

/// Execute the providers command.
pub async fn execute(args: ProvidersArgs, format: OutputFormat) -> Result<()> {
    let dx = DxMedia::new()?;
    let registry = dx.registry();

    let providers = if args.available {
        registry.available()
    } else {
        registry.all()
    };

    match format {
        OutputFormat::Json | OutputFormat::JsonCompact => {
            let json: Vec<serde_json::Value> = providers
                .iter()
                .map(|p| {
                    let mut obj = provider_json_row(p.as_ref(), "supported_types");

                    if args.detailed {
                        // Try to get extended info if available
                        // We can't easily downcast, so we'll use the trait methods we have
                        obj["base_url"] = serde_json::json!(p.base_url());
                        obj["rate_limit"] = serde_json::json!({
                            "requests_per_window": p.rate_limit().requests_per_window(),
                            "window_secs": p.rate_limit().window_secs(),
                        });
                    }

                    obj
                })
                .collect();

            if matches!(format, OutputFormat::JsonCompact) {
                println!("{}", serde_json::to_string(&json)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
        }
        OutputFormat::Tsv => {
            println!(
                "name\tdisplay_name\tavailable\trequires_api_key\tcredential_status\tsource_kind\tunavailable_reason"
            );
            for p in &providers {
                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    p.name(),
                    p.display_name(),
                    p.is_available(),
                    p.requires_api_key(),
                    credential_status(p.as_ref()),
                    source_kind(p.as_ref()),
                    if p.is_available() {
                        ""
                    } else if p.requires_api_key() {
                        "missing credentials or disabled"
                    } else {
                        "provider disabled or unavailable"
                    }
                );
            }
        }
        OutputFormat::Text => {
            let stats = registry.stats();

            println!("{}", "Available Providers".bold().cyan());
            println!(
                "{} {} total, {} available, {} unavailable",
                "Stats:".dimmed(),
                stats.total,
                stats.available.to_string().green(),
                stats.unavailable.to_string().yellow()
            );
            println!();

            for p in &providers {
                let status = if p.is_available() {
                    "✓".green()
                } else {
                    "✗".red()
                };

                let types: Vec<&str> = p
                    .supported_media_types()
                    .iter()
                    .map(|t| t.as_str())
                    .collect();

                println!(
                    "  {} {} {}",
                    status,
                    p.display_name().bold(),
                    format!("({})", p.name()).dimmed()
                );
                println!("      {} {}", "Types:".dimmed(), types.join(", "));
                println!("      {} {}", "Source:".dimmed(), source_kind(p.as_ref()));
                println!(
                    "      {} {}",
                    "Credentials:".dimmed(),
                    credential_status(p.as_ref())
                );

                if args.detailed {
                    let rate = p.rate_limit();
                    if rate.is_limited() {
                        println!(
                            "      {} {}/{}s",
                            "Rate:".dimmed(),
                            rate.requests_per_window(),
                            rate.window_secs()
                        );
                    } else {
                        println!("      {} unlimited", "Rate:".dimmed());
                    }

                    if p.requires_api_key() && !p.is_available() {
                        println!("      {} {}", "Note:".yellow(), "API key required".yellow());
                    } else if !p.is_available() {
                        println!(
                            "      {} {}",
                            "Note:".yellow(),
                            "Provider disabled or unavailable".yellow()
                        );
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}
