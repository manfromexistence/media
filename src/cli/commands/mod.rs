//! Command execution module.

mod download;
mod providers;
mod scrape;
mod search;

use crate::cli::args::{Args, Command};
use crate::error::Result;

const PROVIDER_METADATA_HINT: &str = "Provider availability is source-specific; run \
`dx providers --format json` for requires_credentials, credential_status, and source_kind.";
const PROVIDER_STATUS_TEXT: &str = "Provider-specific; inspect requires_credentials, \
credential_status, and source_kind before assuming access.";

/// Execute a CLI command.
pub async fn execute(args: Args) -> Result<()> {
    match args.command {
        Command::Search(search_args) => search::execute(search_args, args.format, args.quiet).await,
        Command::Download(download_args) => {
            download::execute(download_args, args.format, args.quiet).await
        }
        Command::Scrape(scrape_args) => scrape::execute(scrape_args, args.format, args.quiet).await,
        Command::Providers(provider_args) => providers::execute(provider_args, args.format).await,
        Command::Config => config_command(args.format).await,
        Command::CheckDeps => check_deps_command().await,
        Command::Interactive => interactive_command().await,
    }
}

/// Execute the config command.
async fn config_command(format: crate::cli::args::OutputFormat) -> Result<()> {
    use crate::DxMedia;
    use colored::Colorize;

    let dx = DxMedia::new()?;
    let config = dx.config();

    match format {
        crate::cli::args::OutputFormat::Json | crate::cli::args::OutputFormat::JsonCompact => {
            let json = serde_json::json!({
                "download_dir": config.download_dir,
                "timeout_secs": config.timeout_secs,
                "retry_attempts": config.retry_attempts,
                "providers": PROVIDER_METADATA_HINT,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            println!("{}", "DX Media Configuration".bold().cyan());
            println!();
            println!(
                "  {} {}",
                "Download Directory:".dimmed(),
                config.download_dir.display()
            );
            println!("  {} {} seconds", "Timeout:".dimmed(), config.timeout_secs);
            println!("  {} {}", "Retry Attempts:".dimmed(), config.retry_attempts);
            println!();
            println!("{}", "Providers:".bold());
            println!("  {} {}", "Status:".dimmed(), PROVIDER_STATUS_TEXT.green());
            println!("  {} {}", "Metadata:".dimmed(), PROVIDER_METADATA_HINT);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_status_text_is_credential_honest() {
        let combined = format!("{PROVIDER_METADATA_HINT}\n{PROVIDER_STATUS_TEXT}");

        assert!(!combined.contains("All providers are FREE"));
        assert!(!combined.contains("no API keys required"));
        assert!(combined.contains("requires_credentials"));
        assert!(combined.contains("credential_status"));
        assert!(combined.contains("source_kind"));
    }
}

/// Execute the interactive command (placeholder).
async fn interactive_command() -> Result<()> {
    use colored::Colorize;

    println!("{}", "Interactive mode coming soon!".yellow());
    println!("For now, use the search and download commands.");

    Ok(())
}

/// Execute the check-deps command.
async fn check_deps_command() -> Result<()> {
    use colored::Colorize;
    use std::process::Command;

    println!("{}", "DX Media External Dependencies".bold().cyan());
    println!();
    println!("{}", "Checking for required external tools...".dimmed());
    println!();

    // List of dependencies to check
    let deps = [
        (
            "ffmpeg",
            "-version",
            "Video/Audio processing (19 tools)",
            true,
        ),
        ("ffprobe", "-version", "Media information", true),
        ("tesseract", "--version", "OCR - text from images", false),
        ("wkhtmltopdf", "--version", "HTML to PDF conversion", false),
        (
            "pandoc",
            "--version",
            "Universal document conversion",
            false,
        ),
        ("7z", "", "7-zip archive support", false),
        ("exiftool", "-ver", "EXIF metadata handling", false),
    ];

    let mut installed = 0;
    let mut required_missing = 0;

    for (name, version_arg, description, required) in deps {
        let result = if version_arg.is_empty() {
            Command::new(name).output()
        } else {
            Command::new(name).arg(version_arg).output()
        };

        match result {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let version = stdout.lines().next().unwrap_or("").trim();
                let version_short = if version.len() > 40 {
                    &version[..40]
                } else {
                    version
                };
                println!(
                    "  {} {} - {} {}",
                    "✓".green(),
                    name.bold(),
                    description,
                    format!("({})", version_short).dimmed()
                );
                installed += 1;
            }
            _ => {
                if required {
                    println!(
                        "  {} {} - {} {}",
                        "✗".red(),
                        name.bold(),
                        description,
                        "(required)".red()
                    );
                    required_missing += 1;
                } else {
                    println!(
                        "  {} {} - {} {}",
                        "○".yellow(),
                        name,
                        description,
                        "(optional)".dimmed()
                    );
                }
            }
        }
    }

    println!();
    println!("{}", "Summary:".bold());
    println!("  {} {} tools installed", "→".cyan(), installed);

    if required_missing > 0 {
        println!(
            "  {} {} required tools missing",
            "!".red().bold(),
            required_missing
        );
        println!();
        println!("{}", "Install FFmpeg:".bold().yellow());
        println!("  Windows: winget install FFmpeg");
        println!("  macOS:   brew install ffmpeg");
        println!("  Linux:   apt install ffmpeg");
    } else {
        println!("  {} All required dependencies available!", "✓".green());
    }

    println!();
    println!("{}", "Tool Availability:".bold());
    println!("  {} 33 tools work natively (no dependencies)", "→".cyan());
    println!(
        "  {} 19 tools require FFmpeg (video/audio)",
        if required_missing == 0 {
            "→".cyan()
        } else {
            "!".red()
        }
    );
    println!("  {} 8 tools enhanced by optional deps", "→".cyan());

    Ok(())
}
