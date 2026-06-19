//! Media processing tools commands

use anyhow::Result;
use std::path::PathBuf;

use crate::cli_unified::args::{
    ArchiveToolCommands, AudioToolCommands, ImageToolCommands, OutputFormat, ToolCommands,
    VideoToolCommands,
};
use crate::cli_unified::output::{print_info, print_json, print_success};
use crate::tools::{
    ToolCategory, all_tool_descriptors, tool_descriptor_records,
    tool_descriptor_records_for_category,
};

pub async fn execute_tool_command(command: ToolCommands, format: &OutputFormat) -> Result<()> {
    match command {
        ToolCommands::List { category } => {
            execute_tool_list(category.as_deref(), format)?;
            Ok(())
        }
        ToolCommands::Image { command } => execute_image_tool(command).await,
        ToolCommands::Video { command } => execute_video_tool(command).await,
        ToolCommands::Audio { command } => execute_audio_tool(command).await,
        ToolCommands::Archive { command } => execute_archive_tool(command).await,
    }
}

fn execute_tool_list(category: Option<&str>, format: &OutputFormat) -> Result<()> {
    let category = category
        .map(|value| {
            ToolCategory::from_filter(value).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown tool category '{value}'. Valid categories: {}",
                    ToolCategory::valid_names().join(", ")
                )
            })
        })
        .transpose()?;

    if matches!(format, OutputFormat::Json) {
        let records = category
            .map(|category| tool_descriptor_records_for_category(category.as_str()))
            .unwrap_or_else(tool_descriptor_records);
        print_json(&records)?;
        return Ok(());
    }

    let filter = category.map(|category| category.as_str());

    println!(
        "{:<30} {:<10} {:<20} {:<22} {:<18} {:<24} {:<28} {:<16} {:<16} {:<9} {:<42} {:<18} {}",
        "tool",
        "category",
        "source",
        "readiness",
        "receipt",
        "type-validation",
        "receipt-aliases",
        "credentials",
        "dep-status",
        "api-only",
        "routes",
        "dependency",
        "feature"
    );
    println!("{}", "-".repeat(320));

    for tool in all_tool_descriptors() {
        if filter.is_some_and(|category| category != tool.category.as_str()) {
            continue;
        }

        let routes = tool
            .command_routes()
            .iter()
            .map(|route| {
                format!(
                    "{}[{}:{}/{}/{}]",
                    route.path,
                    route.surface,
                    route.readiness,
                    route.receipt_readiness,
                    route.type_validation_readiness
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let receipt_aliases = match tool.implementation_receipt_names() {
            [] => "-".to_string(),
            aliases => aliases.join(", "),
        };

        println!(
            "{:<30} {:<10} {:<20} {:<22} {:<18} {:<24} {:<28} {:<16} {:<16} {:<9} {:<42} {:<18} {}",
            tool.name,
            tool.category.as_str(),
            tool.source_kind.as_str(),
            tool.readiness.as_str(),
            tool.receipt_readiness().as_str(),
            tool.type_validation_readiness().as_str(),
            receipt_aliases,
            tool.credential_status(),
            tool.external_dependency_status(),
            tool.api_only(),
            routes,
            tool.dependency.unwrap_or("-"),
            tool.feature.unwrap_or("-")
        );
    }

    Ok(())
}

async fn execute_image_tool(command: ImageToolCommands) -> Result<()> {
    match command {
        ImageToolCommands::Convert {
            input,
            output,
            quality,
        } => {
            print_info(&format!(
                "🖼️  Converting {} to {}...",
                input.display(),
                output.display()
            ));

            #[cfg(feature = "image-core")]
            {
                use crate::tools::image::native::convert_native;
                match convert_native(&input, &output, quality) {
                    Ok(_) => print_success("Image converted successfully!"),
                    Err(e) => anyhow::bail!("Conversion failed: {}", e),
                }
                return Ok(());
            }

            #[cfg(not(feature = "image-core"))]
            {
                let _ = (input, output, quality);
                anyhow::bail!("Image tools not enabled. Rebuild with --features image-core")
            }
        }
        ImageToolCommands::Resize {
            input,
            output,
            width,
            height,
        } => {
            print_info(&format!("🖼️  Resizing {}...", input.display()));

            #[cfg(feature = "image-core")]
            {
                use crate::tools::image::native::resize_native;
                match resize_native(&input, &output, width, height, true) {
                    Ok(_) => print_success("Image resized successfully!"),
                    Err(e) => anyhow::bail!("Resize failed: {}", e),
                }
                return Ok(());
            }

            #[cfg(not(feature = "image-core"))]
            {
                let _ = (input, output, width, height);
                anyhow::bail!("Image tools not enabled. Rebuild with --features image-core")
            }
        }
        ImageToolCommands::Favicon { input, output } => {
            print_info(&format!(
                "🎨 Generating favicons from {}...",
                input.display()
            ));

            #[cfg(feature = "image-svg")]
            {
                use crate::tools::image::svg::generate_web_icons;
                generate_web_icons(&input, &output)?;
                print_success(&format!("Favicons generated in {}", output.display()));
                return Ok(());
            }

            #[cfg(not(feature = "image-svg"))]
            {
                let _ = input;
                let _ = output;
                anyhow::bail!("SVG tools not enabled. Rebuild with --features image-svg")
            }
        }
    }
}

async fn execute_video_tool(command: VideoToolCommands) -> Result<()> {
    match command {
        VideoToolCommands::Convert { input, output } => {
            print_info(&format!(
                "🎬 Converting {} to {}...",
                input.display(),
                output.display()
            ));
            declared_external_tool_not_wired("video.transcode", "ffmpeg")
        }
        VideoToolCommands::ExtractAudio { input, output: _ } => {
            print_info(&format!("🎵 Extracting audio from {}...", input.display()));
            declared_external_tool_not_wired("video.extract-audio", "ffmpeg")
        }
        VideoToolCommands::ToGif {
            input,
            output: _,
            fps,
        } => {
            print_info(&format!(
                "🎞️  Converting {} to GIF ({}fps)...",
                input.display(),
                fps
            ));
            declared_external_tool_not_wired("video.to-gif", "ffmpeg")
        }
    }
}

async fn execute_audio_tool(command: AudioToolCommands) -> Result<()> {
    match command {
        AudioToolCommands::Convert { input, output } => {
            print_info(&format!(
                "🎵 Converting {} to {}...",
                input.display(),
                output.display()
            ));
            declared_external_tool_not_wired("audio.convert", "ffmpeg")
        }
        AudioToolCommands::Trim {
            input,
            output: _,
            start,
            duration,
        } => {
            print_info(&format!(
                "✂️  Trimming {} (start: {}s, duration: {}s)...",
                input.display(),
                start,
                duration
            ));
            declared_external_tool_not_wired("audio.trim", "ffmpeg")
        }
    }
}

fn declared_external_tool_not_wired(tool_name: &str, dependency: &str) -> Result<()> {
    anyhow::bail!(
        "{tool_name} is declared as an external-dependency tool requiring {dependency}, \
         but this unified tools CLI path is not wired yet; no output file or no tool receipt was produced"
    )
}

async fn execute_archive_tool(command: ArchiveToolCommands) -> Result<()> {
    match command {
        ArchiveToolCommands::Zip { files, output } => {
            print_info(&format!("📦 Creating archive {}...", output.display()));

            #[cfg(feature = "archive-core")]
            {
                use crate::tools::ArchiveTools;
                let tools = ArchiveTools::new();

                let file_paths: Vec<&PathBuf> = files.iter().collect();
                tools.create_zip(&file_paths, &output)?;

                print_success(&format!("Archive created: {}", output.display()));
            }

            #[cfg(not(feature = "archive-core"))]
            {
                anyhow::bail!("Archive tools not enabled. Rebuild with --features archive-core");
            }

            Ok(())
        }
        ArchiveToolCommands::Extract { input, output } => {
            print_info(&format!("📂 Extracting {}...", input.display()));

            #[cfg(feature = "archive-core")]
            {
                use crate::tools::ArchiveTools;
                let tools = ArchiveTools::new();

                tools.extract_zip(&input, &output)?;

                print_success(&format!("Extracted to: {}", output.display()));
            }

            #[cfg(not(feature = "archive-core"))]
            {
                anyhow::bail!("Archive tools not enabled. Rebuild with --features archive-core");
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    #[tokio::test]
    async fn declared_external_video_tools_return_errors_without_receipt_claims() {
        let commands = [
            (
                "video.transcode",
                VideoToolCommands::Convert {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("output.webm"),
                },
            ),
            (
                "video.extract-audio",
                VideoToolCommands::ExtractAudio {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("output.wav"),
                },
            ),
            (
                "video.to-gif",
                VideoToolCommands::ToGif {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("output.gif"),
                    fps: 10,
                },
            ),
        ];

        for (tool_name, command) in commands {
            let err = execute_video_tool(command)
                .await
                .expect_err("declared external video tool should not report success");
            let message = err.to_string();

            assert!(message.contains(tool_name), "{message}");
            assert!(message.contains("no output file"), "{message}");
            assert!(message.contains("no tool receipt"), "{message}");
        }
    }

    #[tokio::test]
    async fn declared_external_audio_tools_return_errors_without_receipt_claims() {
        let commands = [
            (
                "audio.convert",
                AudioToolCommands::Convert {
                    input: fixture_path("input.wav"),
                    output: fixture_path("output.mp3"),
                },
            ),
            (
                "audio.trim",
                AudioToolCommands::Trim {
                    input: fixture_path("input.wav"),
                    output: fixture_path("trimmed.wav"),
                    start: 1.0,
                    duration: 2.0,
                },
            ),
        ];

        for (tool_name, command) in commands {
            let err = execute_audio_tool(command)
                .await
                .expect_err("declared external audio tool should not report success");
            let message = err.to_string();

            assert!(message.contains(tool_name), "{message}");
            assert!(message.contains("no output file"), "{message}");
            assert!(message.contains("no tool receipt"), "{message}");
        }
    }
}
