//! Extended tool command handlers for all 56+ tools

use crate::cli_unified::args::OutputFormat;
use crate::cli_unified::args_extended::*;
use crate::cli_unified::config::MediaConfig;
use crate::cli_unified::output::{print_info, print_json, print_success};
use crate::tools::ToolOutput;
use anyhow::Result;
use std::path::Path;

pub async fn execute_video_extended(command: VideoToolsExtended) -> Result<()> {
    let tool_name = match command {
        VideoToolsExtended::Transcode { .. } => "video.transcode",
        VideoToolsExtended::ExtractAudio { .. } => "video.extract-audio",
        VideoToolsExtended::Trim { .. } => "video.trim",
        VideoToolsExtended::Scale { .. } => "video.scale",
        VideoToolsExtended::ToGif { .. } => "video.to-gif",
        VideoToolsExtended::Thumbnail { .. } => "video.thumbnail",
        VideoToolsExtended::Mute { .. } => "video.mute",
        VideoToolsExtended::Watermark { .. } => "video.watermark",
        VideoToolsExtended::Speed { .. } => "video.speed",
        VideoToolsExtended::Concat { .. } => "video.concat",
        VideoToolsExtended::Subtitles { .. } => "video.subtitles",
    };

    declared_external_tool_not_wired(tool_name, "FFmpeg")
}

pub async fn execute_audio_extended(
    command: AudioToolsExtended,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        AudioToolsExtended::Convert { input, output } => {
            if !matches!(format, OutputFormat::Json) {
                print_info(&format!(
                    "Converting {} to {}...",
                    input.display(),
                    output.display()
                ));
            }
            let options = audio_convert_options_for_output(&output)?;
            let result = crate::tools::audio::convert_audio(&input, &output, options)?;
            ensure_tool_output_type_validation_pass(&result)?;
            print_tool_output(&result, format)?;
            Ok(())
        }
        AudioToolsExtended::Trim { .. } => declared_external_tool_not_wired("audio.trim", "FFmpeg"),
        AudioToolsExtended::Merge { .. } => {
            declared_external_tool_not_wired("audio.merge", "FFmpeg")
        }
        AudioToolsExtended::Normalize { .. } => {
            declared_external_tool_not_wired("audio.normalize", "FFmpeg")
        }
        AudioToolsExtended::RemoveSilence { .. } => {
            declared_external_tool_not_wired("audio.remove-silence", "FFmpeg")
        }
        AudioToolsExtended::Split { .. } => {
            declared_external_tool_not_wired("audio.split", "FFmpeg")
        }
        AudioToolsExtended::Effects { .. } => {
            declared_external_tool_not_wired("audio.effects", "FFmpeg")
        }
        AudioToolsExtended::Spectrum { .. } => {
            declared_external_tool_not_wired("audio.spectrum", "FFmpeg")
        }
        AudioToolsExtended::Metadata { .. } => {
            declared_external_tool_not_wired("audio.metadata", "FFprobe")
        }
    }
}

fn print_tool_output(output: &ToolOutput, format: &OutputFormat) -> Result<()> {
    if matches!(format, OutputFormat::Json) {
        print_json(output)?;
        return Ok(());
    }

    let source_kind = output
        .metadata
        .get("tool.source_kind")
        .map_or("unknown", String::as_str);
    let receipt = output
        .metadata
        .get("tool.receipt_completeness")
        .map_or("unknown", String::as_str);
    let type_validation = output
        .metadata
        .get("tool.type_validation")
        .map_or("unknown", String::as_str);
    let dependency = output
        .metadata
        .get("tool.dependency")
        .map_or("none", String::as_str);

    print_success(&format!(
        "{} (source={}, receipt={}, type_validation={}, dependency={})",
        output.message, source_kind, receipt, type_validation, dependency
    ));
    Ok(())
}

fn declared_external_tool_not_wired(tool_name: &str, dependency: &str) -> Result<()> {
    anyhow::bail!(
        "{tool_name} is declared as an external-dependency tool requiring {dependency}, \
         but this extended CLI path is not wired to execute it yet; no output file or \
         no tool receipt was produced"
    )
}

fn audio_convert_options_for_output(output: &Path) -> Result<crate::tools::audio::ConvertOptions> {
    let extension = output
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let Some(format) = crate::tools::audio::AudioOutputFormat::from_extension(extension) else {
        let shown_extension = if extension.is_empty() {
            "<none>"
        } else {
            extension
        };
        anyhow::bail!(
            "audio.convert unsupported output extension `{}` for {}; supported audio extensions: \
             mp3, wav, flac, ogg, aac, m4a, wma, opus; no output file or no tool receipt was produced",
            shown_extension,
            output.display()
        );
    };

    let mut options = crate::tools::audio::ConvertOptions::default();
    options.format = format;
    match format {
        crate::tools::audio::AudioOutputFormat::Wav => {
            options.bitrate = None;
            options.sample_rate = Some(44_100);
        }
        crate::tools::audio::AudioOutputFormat::Flac => {
            options.bitrate = None;
            options.sample_rate = None;
        }
        _ => {}
    }

    Ok(options)
}

fn require_existing_input(tool_name: &str, input: &std::path::Path) -> Result<()> {
    if !input.exists() {
        anyhow::bail!(
            "{tool_name} input file not found: {}; no output file or no tool receipt was produced",
            input.display()
        );
    }

    Ok(())
}

#[cfg(not(feature = "image-core"))]
fn image_core_feature_required(tool_name: &str) -> Result<ToolOutput> {
    feature_required_tool_output(tool_name, "image-core")
}

#[cfg(any(not(feature = "image-core"), not(feature = "image-svg")))]
fn feature_required_tool_output(tool_name: &str, feature: &str) -> Result<ToolOutput> {
    anyhow::bail!(
        "{tool_name} requires building with --features {feature}; no output file or no tool \
         receipt was produced"
    )
}

fn declared_feature_tool_not_wired(tool_name: &str, feature: &str) -> Result<()> {
    anyhow::bail!(
        "{tool_name} is declared as a feature-gated tool requiring --features {feature}, \
         but this extended CLI path is not wired to execute it yet; no output file or no \
         tool receipt was produced"
    )
}

fn declared_feature_tool_not_wired_output(tool_name: &str, feature: &str) -> Result<ToolOutput> {
    anyhow::bail!(
        "{tool_name} is declared as a feature-gated tool requiring --features {feature}, \
         but this extended CLI path is not wired to execute it yet; no output file or no \
         tool receipt was produced"
    )
}

#[cfg(not(feature = "document-core"))]
fn feature_required_output(tool_name: &str, feature: &str) -> Result<ToolOutput> {
    anyhow::bail!(
        "{tool_name} requires building with --features {feature}; no output file or no tool \
         receipt was produced"
    )
}

fn declared_external_tool_not_wired_output(
    tool_name: &str,
    dependency: &str,
) -> Result<ToolOutput> {
    anyhow::bail!(
        "{tool_name} is declared as an external-dependency tool requiring {dependency}, \
         but this extended CLI path is not wired to execute it yet; no output file or \
         no tool receipt was produced"
    )
}

fn ensure_tool_output_type_validation_pass(output: &ToolOutput) -> Result<()> {
    if output
        .metadata
        .get("tool.type_validation")
        .is_some_and(|validation| validation == "fail")
    {
        let tool_name = output
            .metadata
            .get("tool.name")
            .map_or("media tool", String::as_str);
        let reason = output
            .metadata
            .get("tool.type_validation_reason")
            .map_or("type-validation-failed", String::as_str);

        anyhow::bail!("{tool_name} failed type validation: {reason}");
    }

    Ok(())
}

fn ensure_output_extension(
    tool_name: &str,
    output: &Path,
    allowed_extensions: &[&str],
    expected_media_type: &str,
) -> Result<()> {
    let extension = output
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| "unknown".to_string());

    if allowed_extensions.contains(&extension.as_str()) {
        return Ok(());
    }

    anyhow::bail!(
        "{tool_name} failed type validation: extension-mismatch; expected {expected_media_type} \
         output extension, got `{extension}` for {}; no output file or no tool receipt was \
         produced",
        output.display()
    )
}

fn ensure_output_has_extension(tool_name: &str, output: &Path) -> Result<()> {
    if output.extension().and_then(|ext| ext.to_str()).is_some() {
        return Ok(());
    }

    anyhow::bail!(
        "{tool_name} failed type validation: missing-output-extension; output file {} must have \
         an extension; no output file or no tool receipt was produced",
        output.display()
    )
}

fn ensure_cli_input_exists(tool_name: &'static str, path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    anyhow::bail!(
        "{tool_name} missing input: {} (no output file or no tool receipt was produced)",
        path.display()
    )
}

fn utility_runtime_error(tool_name: &'static str, error: impl std::fmt::Display) -> anyhow::Error {
    anyhow::anyhow!("{tool_name} failed: {error}; no output file or no tool receipt was produced")
}

pub async fn execute_image_extended(
    command: ImageToolsExtended,
    format: &OutputFormat,
) -> Result<()> {
    let result = run_image_extended(command)?;
    ensure_tool_output_type_validation_pass(&result)?;

    match format {
        OutputFormat::Json => print_json(&result)?,
        OutputFormat::Table | OutputFormat::Simple => print_success(&result.message),
    }

    Ok(())
}

fn run_image_extended(command: ImageToolsExtended) -> Result<ToolOutput> {
    match command {
        ImageToolsExtended::Convert {
            input,
            output,
            quality,
        } => {
            require_existing_input("image.convert", &input)?;

            #[cfg(feature = "image-core")]
            {
                use crate::tools::image::native::convert_native;
                convert_native(&input, &output, quality).map_err(|error| {
                    anyhow::anyhow!(
                        "image.convert failed for {}: {}; no tool receipt was produced",
                        input.display(),
                        error
                    )
                })
            }
            #[cfg(not(feature = "image-core"))]
            {
                let _ = (&output, quality);
                image_core_feature_required("image.convert")
            }
        }
        ImageToolsExtended::Resize {
            input,
            output,
            width,
            height,
        } => {
            require_existing_input("image.resize", &input)?;
            #[cfg(feature = "image-core")]
            {
                use crate::tools::image::native::resize_native;
                resize_native(&input, &output, width, height, true).map_err(|error| {
                    anyhow::anyhow!(
                        "image.resize failed for {}: {}; no tool receipt was produced",
                        input.display(),
                        error
                    )
                })
            }
            #[cfg(not(feature = "image-core"))]
            {
                let _ = (&output, width, height);
                image_core_feature_required("image.resize")
            }
        }
        ImageToolsExtended::Compress {
            input,
            output,
            quality,
        } => {
            require_existing_input("image.compress", &input)?;
            #[cfg(feature = "image-core")]
            {
                use crate::tools::image::native::compress_native;
                compress_native(&input, &output, quality).map_err(|error| {
                    anyhow::anyhow!(
                        "image.compress failed for {}: {}; no tool receipt was produced",
                        input.display(),
                        error
                    )
                })
            }
            #[cfg(not(feature = "image-core"))]
            {
                let _ = (&output, quality);
                image_core_feature_required("image.compress")
            }
        }
        ImageToolsExtended::Favicon { input, output_dir } => {
            require_existing_input("image.favicon", &input)?;
            #[cfg(feature = "image-svg")]
            {
                use crate::tools::image::svg::generate_web_icons;
                generate_web_icons(&input, &output_dir).map_err(|error| {
                    anyhow::anyhow!(
                        "image.favicon failed for {}: {}; no tool receipt was produced",
                        input.display(),
                        error
                    )
                })
            }
            #[cfg(not(feature = "image-svg"))]
            {
                let _ = &output_dir;
                feature_required_tool_output("image.favicon", "image-svg")
            }
        }
        ImageToolsExtended::Watermark {
            input,
            output,
            text,
        } => {
            require_existing_input("image.watermark", &input)?;
            let _ = (&output, &text);
            declared_feature_tool_not_wired_output("image.watermark", "image-core")
        }
        ImageToolsExtended::Filter {
            input,
            output,
            filter,
        } => {
            require_existing_input("image.filter", &input)?;
            let _ = (&output, &filter);
            declared_feature_tool_not_wired_output("image.filter", "image-core")
        }
        ImageToolsExtended::Exif { input } => {
            require_existing_input("image.exif", &input)?;
            declared_feature_tool_not_wired_output("image.exif", "image-core")
        }
        ImageToolsExtended::Qr { text, output } => {
            let _ = (&text, &output);
            declared_feature_tool_not_wired_output("image.qr", "image-qr")
        }
        ImageToolsExtended::Palette { input, colors } => {
            require_existing_input("image.palette", &input)?;
            #[cfg(feature = "image-core")]
            {
                use crate::tools::image::native::extract_palette_native;
                extract_palette_native(&input, colors).map_err(|error| {
                    anyhow::anyhow!(
                        "image.palette failed for {}: {}; no tool receipt was produced",
                        input.display(),
                        error
                    )
                })
            }
            #[cfg(not(feature = "image-core"))]
            {
                let _ = colors;
                image_core_feature_required("image.palette")
            }
        }
        ImageToolsExtended::Ocr { input } => {
            require_existing_input("image.ocr", &input)?;
            declared_external_tool_not_wired_output("image.ocr", "Tesseract")
        }
    }
}

pub async fn execute_archive_extended(
    command: ArchiveToolsExtended,
    config: &MediaConfig,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        ArchiveToolsExtended::Zip { files, mut output } => {
            // Use config directory if output is just a filename (no path components)
            if !output.is_absolute() && output.components().count() == 1 {
                let archive_dir = config.get_archive_dir();
                config.ensure_dir(&archive_dir)?;
                output = archive_dir.join(&output);
            }

            if !matches!(format, OutputFormat::Json) {
                print_info(&format!(
                    "📦 Creating ZIP archive with {} files...",
                    files.len()
                ));
            }

            let result = crate::tools::archive::native::create_zip_native(&files, &output, None)?;
            ensure_tool_output_type_validation_pass(&result)?;

            print_tool_output(&result, format)?;
            Ok(())
        }
        ArchiveToolsExtended::Unzip { input, output } => {
            if !matches!(format, OutputFormat::Json) {
                print_info(&format!("📂 Extracting {}...", input.display()));
            }
            let result = crate::tools::archive::native::extract_zip_native(&input, &output)?;
            ensure_tool_output_type_validation_pass(&result)?;

            print_tool_output(&result, format)?;
            Ok(())
        }
        ArchiveToolsExtended::Tar { files, output } => {
            if !matches!(format, OutputFormat::Json) {
                print_info(&format!(
                    "📦 Creating TAR archive with {} files...",
                    files.len()
                ));
            }
            let _ = &output;
            declared_feature_tool_not_wired("archive.tar", "archive-core")
        }
        ArchiveToolsExtended::Untar { input, output } => {
            if !matches!(format, OutputFormat::Json) {
                print_info(&format!("📂 Extracting TAR {}...", input.display()));
            }
            let _ = &output;
            declared_feature_tool_not_wired("archive.untar", "archive-core")
        }
        ArchiveToolsExtended::Gzip { input, output } => {
            if !matches!(format, OutputFormat::Json) {
                print_info("🗜️  Compressing with gzip...");
            }
            let _ = (input, output);
            declared_feature_tool_not_wired("archive.gzip", "archive-core")
        }
        ArchiveToolsExtended::Gunzip { input, output } => {
            if !matches!(format, OutputFormat::Json) {
                print_info("📂 Decompressing gzip...");
            }
            let _ = (input, output);
            declared_feature_tool_not_wired("archive.gunzip", "archive-core")
        }
        ArchiveToolsExtended::List { input } => {
            if !matches!(format, OutputFormat::Json) {
                print_info(&format!("📋 Listing contents of {}...", input.display()));
            }
            let result = crate::tools::archive::native::list_zip_native(&input)?;
            ensure_tool_output_type_validation_pass(&result)?;
            print_tool_output(&result, format)?;
            if !matches!(format, OutputFormat::Json) {
                if let Some(files) = result.metadata.get("files") {
                    for file in files.split(';').filter(|file| !file.is_empty()) {
                        println!("  {}", file);
                    }
                }
            }
            Ok(())
        }
    }
}

pub async fn execute_document_extended(
    command: DocumentToolsExtended,
    format: &OutputFormat,
) -> Result<()> {
    let result = run_document_extended(command)?;
    ensure_tool_output_type_validation_pass(&result)?;

    match format {
        OutputFormat::Json => print_json(&result)?,
        OutputFormat::Table | OutputFormat::Simple => print_success(&result.message),
    }

    Ok(())
}

fn run_document_extended(command: DocumentToolsExtended) -> Result<ToolOutput> {
    match command {
        DocumentToolsExtended::MarkdownToHtml { input, output } => {
            #[cfg(feature = "document-core")]
            {
                require_existing_input("document.markdown-to-html", &input)?;
                crate::tools::document::native::markdown_to_html_native(&input, &output, true)
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "document.markdown-to-html failed for {}: {}; no tool receipt was produced",
                            input.display(),
                            error
                        )
                    })
            }
            #[cfg(not(feature = "document-core"))]
            {
                let _ = (input, output);
                feature_required_output("document.markdown-to-html", "document-core")
            }
        }
        DocumentToolsExtended::ExtractText { input, output } => {
            #[cfg(feature = "document-core")]
            {
                require_existing_input("document.extract-text", &input)?;
                crate::tools::document::text_extract::extract_to_file(&input, &output).map_err(
                    |error| {
                        anyhow::anyhow!(
                            "document.extract-text failed for {}: {}; no tool receipt was produced",
                            input.display(),
                            error
                        )
                    },
                )
            }
            #[cfg(not(feature = "document-core"))]
            {
                let _ = (input, output);
                feature_required_output("document.extract-text", "document-core")
            }
        }
        DocumentToolsExtended::PdfMerge { inputs, output } => {
            let _ = (inputs, output);
            declared_external_tool_not_wired_output("document.pdf-merge", "Ghostscript")
        }
        DocumentToolsExtended::PdfSplit { input, output_dir } => {
            let _ = (input, output_dir);
            declared_external_tool_not_wired_output("document.pdf-split", "Ghostscript")
        }
        DocumentToolsExtended::PdfCompress { input, output } => {
            let _ = (input, output);
            declared_external_tool_not_wired_output("document.pdf-compress", "Ghostscript")
        }
        DocumentToolsExtended::PdfEncrypt {
            input,
            output,
            password,
        } => {
            let _ = (input, output, password);
            declared_external_tool_not_wired_output("document.pdf-encrypt", "Ghostscript")
        }
        DocumentToolsExtended::PdfWatermark {
            input,
            output,
            text,
        } => {
            let _ = (input, output, text);
            declared_external_tool_not_wired_output("document.pdf-watermark", "Ghostscript")
        }
        DocumentToolsExtended::PdfToImage { input, output_dir } => {
            let _ = (input, output_dir);
            declared_external_tool_not_wired_output("document.pdf-to-image", "Ghostscript")
        }
        DocumentToolsExtended::HtmlToPdf { input, output } => {
            let _ = (input, output);
            declared_external_tool_not_wired_output("document.html-to-pdf", "wkhtmltopdf")
        }
    }
}

pub async fn execute_utility_extended(
    command: UtilityToolsExtended,
    format: &OutputFormat,
) -> Result<()> {
    match command {
        UtilityToolsExtended::Hash { input, algorithm } => {
            ensure_cli_input_exists("utility.hash", &input)?;
            if !matches!(format, OutputFormat::Json) {
                print_info(&format!("Calculating {algorithm} hash..."));
            }
            use crate::tools::utility::hash::{HashAlgorithm, hash_file};
            let algo = match algorithm.as_str() {
                "md5" => HashAlgorithm::Md5,
                "sha1" => HashAlgorithm::Sha1,
                "sha256" => HashAlgorithm::Sha256,
                "sha384" => HashAlgorithm::Sha384,
                "sha512" => HashAlgorithm::Sha512,
                _ => HashAlgorithm::Sha256,
            };
            let result =
                hash_file(&input, algo).map_err(|e| utility_runtime_error("utility.hash", e))?;
            ensure_tool_output_type_validation_pass(&result)?;
            if matches!(format, OutputFormat::Json) {
                print_json(&result)?;
            } else if let Some(hash) = result.metadata.get("hash") {
                println!("{hash}");
            }
            Ok(())
        }
        UtilityToolsExtended::Base64Encode { input } => {
            ensure_cli_input_exists("utility.base64-encode", &input)?;
            if !matches!(format, OutputFormat::Json) {
                print_info("Encoding to base64...");
            }
            use crate::tools::utility::base64::encode_file;
            let result = encode_file(&input)
                .map_err(|e| utility_runtime_error("utility.base64-encode", e))?;
            ensure_tool_output_type_validation_pass(&result)?;
            if matches!(format, OutputFormat::Json) {
                print_json(&result)?;
            } else {
                println!("{}", result.message);
            }
            Ok(())
        }
        UtilityToolsExtended::Base64Decode { input, output } => {
            ensure_output_has_extension("utility.base64-decode", &output)?;
            if !matches!(format, OutputFormat::Json) {
                print_info("Decoding from base64...");
            }
            use crate::tools::utility::base64::decode_string_to_file;
            let result = decode_string_to_file(&input, &output)
                .map_err(|e| utility_runtime_error("utility.base64-decode", e))?;
            ensure_tool_output_type_validation_pass(&result)?;
            if matches!(format, OutputFormat::Json) {
                print_json(&result)?;
            } else {
                print_success(&format!("Decoded to {}", output.display()));
            }
            Ok(())
        }
        UtilityToolsExtended::UrlEncode { text } => {
            print_info("URL encoding...");
            println!("{}", urlencoding::encode(&text));
            Ok(())
        }
        UtilityToolsExtended::UrlDecode { text } => {
            print_info("URL decoding...");
            let decoded = urlencoding::decode(&text)
                .map_err(|e| utility_runtime_error("utility.url-decode", e))?;
            println!("{}", decoded);
            Ok(())
        }
        UtilityToolsExtended::Uuid => {
            use uuid::Uuid;
            let id = Uuid::new_v4();
            println!("{}", id);
            Ok(())
        }
        UtilityToolsExtended::ValidateUuid { uuid } => {
            use uuid::Uuid;
            match Uuid::parse_str(&uuid) {
                Ok(_) => print_success("Valid UUID"),
                Err(_) => print_success("Invalid UUID"),
            }
            Ok(())
        }
        UtilityToolsExtended::Timestamp { unix } => {
            if let Some(ts) = unix {
                use chrono::{TimeZone, Utc};
                let dt = Utc.timestamp_opt(ts, 0).unwrap();
                println!("{}", dt.to_rfc3339());
            } else {
                let now = chrono::Utc::now();
                println!("Unix: {}", now.timestamp());
                println!("RFC3339: {}", now.to_rfc3339());
            }
            Ok(())
        }
        UtilityToolsExtended::FindDuplicates { directory } => {
            ensure_cli_input_exists("utility.find-duplicates", &directory)?;
            if !matches!(format, OutputFormat::Json) {
                print_info(&format!("Finding duplicates in {}...", directory.display()));
            }
            use crate::tools::utility::duplicate::{DuplicateOptions, find_duplicates_tool};
            let options = DuplicateOptions::default();
            let result = find_duplicates_tool(&directory, Some(options));
            ensure_tool_output_type_validation_pass(&result)?;
            if matches!(format, OutputFormat::Json) {
                print_json(&result)?;
            } else {
                print_success(&result.message);
            }
            Ok(())
        }
        UtilityToolsExtended::VerifyChecksum { file, checksum } => {
            ensure_cli_input_exists("utility.verify-checksum", &file)?;
            if !matches!(format, OutputFormat::Json) {
                print_info(&format!("Verifying checksum for {}...", file.display()));
            }
            use crate::tools::utility::hash::{HashAlgorithm, verify_hash};
            let result = verify_hash(&file, &checksum, HashAlgorithm::Sha256)
                .map_err(|e| utility_runtime_error("utility.verify-checksum", e))?;
            ensure_tool_output_type_validation_pass(&result)?;
            if matches!(format, OutputFormat::Json) {
                print_json(&result)?;
            } else {
                print_success(&result.message);
            }
            Ok(())
        }
        UtilityToolsExtended::JsonToYaml { input, output } => {
            ensure_cli_input_exists("utility.json-to-yaml", &input)?;
            ensure_output_extension("utility.json-to-yaml", &output, &["yaml", "yml"], "yaml")?;
            if !matches!(format, OutputFormat::Json) {
                print_info("Converting JSON to YAML...");
            }
            use crate::tools::utility::yaml_convert::json_to_yaml;
            let result = json_to_yaml(&input, &output)
                .map_err(|e| utility_runtime_error("utility.json-to-yaml", e))?;
            ensure_tool_output_type_validation_pass(&result)?;
            print_tool_output(&result, format)?;
            Ok(())
        }
        UtilityToolsExtended::YamlToJson { input, output } => {
            ensure_cli_input_exists("utility.yaml-to-json", &input)?;
            ensure_output_extension("utility.yaml-to-json", &output, &["json"], "json")?;
            if !matches!(format, OutputFormat::Json) {
                print_info("Converting YAML to JSON...");
            }
            use crate::tools::utility::yaml_convert::yaml_to_json;
            let result = yaml_to_json(&input, &output)
                .map_err(|e| utility_runtime_error("utility.yaml-to-json", e))?;
            ensure_tool_output_type_validation_pass(&result)?;
            print_tool_output(&result, format)?;
            Ok(())
        }
        UtilityToolsExtended::FormatJson { input, output } => {
            ensure_cli_input_exists("utility.format-json", &input)?;
            ensure_output_extension("utility.format-json", &output, &["json"], "json")?;
            if !matches!(format, OutputFormat::Json) {
                print_info("Formatting JSON...");
            }
            use crate::tools::utility::json_format::format_json_file;
            let result = format_json_file(&input, &output)
                .map_err(|e| utility_runtime_error("utility.format-json", e))?;
            ensure_tool_output_type_validation_pass(&result)?;
            print_tool_output(&result, format)?;
            Ok(())
        }
        UtilityToolsExtended::ConvertCsv {
            input,
            output,
            csv_format,
        } => {
            let _ = (input, output, csv_format);
            declared_feature_tool_not_wired("utility.convert-csv", "utility-core")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct EnvGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
            let previous = std::env::var_os(key);
            // Tests using this guard do not run concurrently with other env-mutating tests.
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // Restore the process environment after this focused CLI test.
            unsafe {
                if let Some(previous) = self.previous.take() {
                    std::env::set_var(self.key, previous);
                } else {
                    std::env::remove_var(self.key);
                }
            }
        }
    }

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    fn write_version_only_ffmpeg(dir: &std::path::Path) -> PathBuf {
        #[cfg(windows)]
        {
            let path = dir.join("ffmpeg.cmd");
            std::fs::write(
                &path,
                "@echo off\r\nif \"%1\"==\"-version\" (\r\n  echo fake ffmpeg version\r\n  exit /b 0\r\n)\r\nexit /b 9\r\n",
            )
            .expect("fake ffmpeg should be written");
            path
        }

        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = dir.join("ffmpeg");
            std::fs::write(
                &path,
                "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  echo 'fake ffmpeg version'\n  exit 0\nfi\nexit 9\n",
            )
            .expect("fake ffmpeg should be written");
            let mut permissions = std::fs::metadata(&path)
                .expect("fake ffmpeg metadata should be readable")
                .permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&path, permissions).expect("fake ffmpeg should be executable");
            path
        }
    }

    fn external_document_error_commands() -> Vec<(&'static str, &'static str, DocumentToolsExtended)>
    {
        vec![
            (
                "document.pdf-merge",
                "Ghostscript",
                DocumentToolsExtended::PdfMerge {
                    inputs: vec![fixture_path("one.pdf"), fixture_path("two.pdf")],
                    output: fixture_path("merged.pdf"),
                },
            ),
            (
                "document.pdf-split",
                "Ghostscript",
                DocumentToolsExtended::PdfSplit {
                    input: fixture_path("input.pdf"),
                    output_dir: fixture_path("pages"),
                },
            ),
            (
                "document.pdf-compress",
                "Ghostscript",
                DocumentToolsExtended::PdfCompress {
                    input: fixture_path("input.pdf"),
                    output: fixture_path("compressed.pdf"),
                },
            ),
            (
                "document.pdf-encrypt",
                "Ghostscript",
                DocumentToolsExtended::PdfEncrypt {
                    input: fixture_path("input.pdf"),
                    output: fixture_path("encrypted.pdf"),
                    password: "secret".to_string(),
                },
            ),
            (
                "document.pdf-watermark",
                "Ghostscript",
                DocumentToolsExtended::PdfWatermark {
                    input: fixture_path("input.pdf"),
                    output: fixture_path("watermarked.pdf"),
                    text: "DX".to_string(),
                },
            ),
            (
                "document.pdf-to-image",
                "Ghostscript",
                DocumentToolsExtended::PdfToImage {
                    input: fixture_path("input.pdf"),
                    output_dir: fixture_path("images"),
                },
            ),
            (
                "document.html-to-pdf",
                "wkhtmltopdf",
                DocumentToolsExtended::HtmlToPdf {
                    input: fixture_path("input.html"),
                    output: fixture_path("output.pdf"),
                },
            ),
        ]
    }

    #[tokio::test]
    async fn extended_declared_external_video_tools_return_errors_without_receipts() {
        let commands = [
            (
                "video.transcode",
                VideoToolsExtended::Transcode {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("output.webm"),
                },
            ),
            (
                "video.extract-audio",
                VideoToolsExtended::ExtractAudio {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("output.wav"),
                },
            ),
            (
                "video.trim",
                VideoToolsExtended::Trim {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("trimmed.mp4"),
                    start: 1.0,
                    end: 3.0,
                },
            ),
            (
                "video.scale",
                VideoToolsExtended::Scale {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("scaled.mp4"),
                    width: Some(640),
                    height: Some(360),
                },
            ),
            (
                "video.to-gif",
                VideoToolsExtended::ToGif {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("output.gif"),
                    fps: 10,
                },
            ),
            (
                "video.thumbnail",
                VideoToolsExtended::Thumbnail {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("thumb.png"),
                    timestamp: 1.0,
                },
            ),
            (
                "video.mute",
                VideoToolsExtended::Mute {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("muted.mp4"),
                },
            ),
            (
                "video.watermark",
                VideoToolsExtended::Watermark {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("watermarked.mp4"),
                    text: Some("DX".to_string()),
                    image: None,
                },
            ),
            (
                "video.speed",
                VideoToolsExtended::Speed {
                    input: fixture_path("input.mp4"),
                    output: fixture_path("fast.mp4"),
                    factor: 1.5,
                },
            ),
            (
                "video.concat",
                VideoToolsExtended::Concat {
                    inputs: vec![fixture_path("one.mp4"), fixture_path("two.mp4")],
                    output: fixture_path("joined.mp4"),
                },
            ),
            (
                "video.subtitles",
                VideoToolsExtended::Subtitles {
                    video: fixture_path("input.mp4"),
                    subtitles: fixture_path("captions.srt"),
                    output: fixture_path("captioned.mp4"),
                },
            ),
        ];

        for (tool_name, command) in commands {
            let err = execute_video_extended(command)
                .await
                .expect_err("declared extended video tool should not report success");
            let message = err.to_string();

            assert!(message.contains(tool_name), "{message}");
            assert!(message.contains("no output file"), "{message}");
            assert!(message.contains("no tool receipt"), "{message}");
        }
    }

    #[tokio::test]
    async fn extended_declared_external_audio_tools_return_errors_without_receipts() {
        let commands = [
            (
                "audio.trim",
                "FFmpeg",
                AudioToolsExtended::Trim {
                    input: fixture_path("input.wav"),
                    output: fixture_path("trimmed.wav"),
                    start: 1.0,
                    duration: 2.0,
                },
            ),
            (
                "audio.merge",
                "FFmpeg",
                AudioToolsExtended::Merge {
                    inputs: vec![fixture_path("one.wav"), fixture_path("two.wav")],
                    output: fixture_path("merged.wav"),
                },
            ),
            (
                "audio.normalize",
                "FFmpeg",
                AudioToolsExtended::Normalize {
                    input: fixture_path("input.wav"),
                    output: fixture_path("normalized.wav"),
                },
            ),
            (
                "audio.remove-silence",
                "FFmpeg",
                AudioToolsExtended::RemoveSilence {
                    input: fixture_path("input.wav"),
                    output: fixture_path("trimmed.wav"),
                },
            ),
            (
                "audio.split",
                "FFmpeg",
                AudioToolsExtended::Split {
                    input: fixture_path("input.wav"),
                    output_dir: fixture_path("segments"),
                },
            ),
            (
                "audio.effects",
                "FFmpeg",
                AudioToolsExtended::Effects {
                    input: fixture_path("input.wav"),
                    output: fixture_path("effect.wav"),
                    effect: "reverb".to_string(),
                },
            ),
            (
                "audio.spectrum",
                "FFmpeg",
                AudioToolsExtended::Spectrum {
                    input: fixture_path("input.wav"),
                    output: fixture_path("spectrum.png"),
                },
            ),
            (
                "audio.metadata",
                "FFprobe",
                AudioToolsExtended::Metadata {
                    input: fixture_path("input.wav"),
                },
            ),
        ];

        for (tool_name, dependency, command) in commands {
            let err = execute_audio_extended(command, &OutputFormat::Table)
                .await
                .expect_err("declared extended audio tool should not report success");
            let message = err.to_string();

            assert!(message.contains(tool_name), "{message}");
            assert!(message.contains(dependency), "{message}");
            assert!(message.contains("no output file"), "{message}");
            assert!(message.contains("no tool receipt"), "{message}");
        }
    }

    #[test]
    fn audio_convert_options_follow_output_extension() {
        let flac = audio_convert_options_for_output(&fixture_path("output.flac"))
            .expect("flac output should be supported");
        assert_eq!(flac.format, crate::tools::audio::AudioOutputFormat::Flac);
        assert_eq!(flac.bitrate, None);
        assert_eq!(flac.sample_rate, None);

        let wav = audio_convert_options_for_output(&fixture_path("output.wav"))
            .expect("wav output should be supported");
        assert_eq!(wav.format, crate::tools::audio::AudioOutputFormat::Wav);
        assert_eq!(wav.bitrate, None);
        assert_eq!(wav.sample_rate, Some(44_100));

        let opus = audio_convert_options_for_output(&fixture_path("output.opus"))
            .expect("opus output should be supported");
        assert_eq!(opus.format, crate::tools::audio::AudioOutputFormat::Opus);
        assert_eq!(opus.bitrate, Some(192));
    }

    #[tokio::test]
    async fn extended_audio_convert_rejects_unknown_output_extension_before_running_converter() {
        let dir = tempfile::tempdir().expect("temp dir should be created");

        let err = execute_audio_extended(
            AudioToolsExtended::Convert {
                input: dir.path().join("missing.wav"),
                output: dir.path().join("output.bin"),
            },
            &OutputFormat::Table,
        )
        .await
        .expect_err("unsupported output extension should fail before conversion");
        let message = err.to_string();

        assert!(message.contains("audio.convert"), "{message}");
        assert!(
            message.contains("unsupported output extension"),
            "{message}"
        );
        assert!(message.contains("no output file"), "{message}");
        assert!(message.contains("no tool receipt"), "{message}");
        assert!(
            !message.contains("Input file not found"),
            "extension validation should run before input probing: {message}"
        );
        assert!(
            !message.contains("not wired"),
            "audio.convert should use its real CLI boundary: {message}"
        );
    }

    #[tokio::test]
    async fn extended_audio_convert_routes_to_real_converter() {
        let dir = tempfile::tempdir().expect("temp dir should be created");
        let ffmpeg = write_version_only_ffmpeg(dir.path());
        let _ffmpeg_guard = EnvGuard::set("DX_MEDIA_FFMPEG_BIN", ffmpeg.as_os_str());

        let err = execute_audio_extended(
            AudioToolsExtended::Convert {
                input: dir.path().join("missing.wav"),
                output: dir.path().join("output.mp3"),
            },
            &OutputFormat::Table,
        )
        .await
        .expect_err("real converter should reject a missing input path");
        let message = err.to_string();

        assert!(message.contains("Input file not found"), "{message}");
        assert!(
            !message.contains("not wired"),
            "audio.convert should no longer use the declared-only CLI boundary: {message}"
        );
    }

    #[tokio::test]
    async fn extended_image_tools_return_errors_for_missing_inputs() {
        let missing = fixture_path("definitely-missing-image.png");
        let commands = [
            (
                "image.convert",
                ImageToolsExtended::Convert {
                    input: missing.clone(),
                    output: fixture_path("out.jpg"),
                    quality: Some(80),
                },
            ),
            (
                "image.resize",
                ImageToolsExtended::Resize {
                    input: missing.clone(),
                    output: fixture_path("out.png"),
                    width: Some(64),
                    height: Some(64),
                },
            ),
            (
                "image.compress",
                ImageToolsExtended::Compress {
                    input: missing.clone(),
                    output: fixture_path("out.jpg"),
                    quality: 80,
                },
            ),
            (
                "image.favicon",
                ImageToolsExtended::Favicon {
                    input: missing.clone(),
                    output_dir: fixture_path("icons"),
                },
            ),
            (
                "image.watermark",
                ImageToolsExtended::Watermark {
                    input: missing.clone(),
                    output: fixture_path("watermarked.png"),
                    text: Some("DX".to_string()),
                },
            ),
            (
                "image.filter",
                ImageToolsExtended::Filter {
                    input: missing.clone(),
                    output: fixture_path("filtered.png"),
                    filter: "grayscale".to_string(),
                },
            ),
            (
                "image.exif",
                ImageToolsExtended::Exif {
                    input: missing.clone(),
                },
            ),
            (
                "image.palette",
                ImageToolsExtended::Palette {
                    input: missing.clone(),
                    colors: 5,
                },
            ),
            ("image.ocr", ImageToolsExtended::Ocr { input: missing }),
        ];

        for (tool_name, command) in commands {
            let err = execute_image_extended(command, &OutputFormat::Json)
                .await
                .expect_err("missing image input should not report success");
            let message = err.to_string();

            assert!(message.contains(tool_name), "{message}");
            assert!(message.contains("input file not found"), "{message}");
            assert!(message.contains("no tool receipt"), "{message}");
        }
    }

    #[cfg(not(feature = "image-core"))]
    #[tokio::test]
    async fn extended_native_image_tools_require_image_core_feature() {
        let input = std::env::temp_dir().join(format!(
            "dx-media-image-core-disabled-{}.png",
            std::process::id()
        ));
        std::fs::write(&input, b"not an image").expect("test fixture should be writable");

        let commands = [
            (
                "image.convert",
                "--features image-core",
                ImageToolsExtended::Convert {
                    input: input.clone(),
                    output: fixture_path("out.jpg"),
                    quality: Some(80),
                },
            ),
            (
                "image.resize",
                "--features image-core",
                ImageToolsExtended::Resize {
                    input: input.clone(),
                    output: fixture_path("out.png"),
                    width: Some(64),
                    height: Some(64),
                },
            ),
            (
                "image.compress",
                "--features image-core",
                ImageToolsExtended::Compress {
                    input: input.clone(),
                    output: fixture_path("out.jpg"),
                    quality: 80,
                },
            ),
            (
                "image.favicon",
                "--features image-svg",
                ImageToolsExtended::Favicon {
                    input: input.clone(),
                    output_dir: fixture_path("icons"),
                },
            ),
            (
                "image.watermark",
                "--features image-core",
                ImageToolsExtended::Watermark {
                    input: input.clone(),
                    output: fixture_path("watermarked.png"),
                    text: Some("DX".to_string()),
                },
            ),
            (
                "image.filter",
                "--features image-core",
                ImageToolsExtended::Filter {
                    input: input.clone(),
                    output: fixture_path("filtered.png"),
                    filter: "grayscale".to_string(),
                },
            ),
            (
                "image.exif",
                "--features image-core",
                ImageToolsExtended::Exif {
                    input: input.clone(),
                },
            ),
            (
                "image.qr",
                "--features image-qr",
                ImageToolsExtended::Qr {
                    text: "https://example.com".to_string(),
                    output: fixture_path("qr.png"),
                },
            ),
            (
                "image.palette",
                "--features image-core",
                ImageToolsExtended::Palette {
                    input: input.clone(),
                    colors: 5,
                },
            ),
            (
                "image.ocr",
                "Tesseract",
                ImageToolsExtended::Ocr {
                    input: input.clone(),
                },
            ),
        ];

        for (tool_name, expected_message, command) in commands {
            let err = execute_image_extended(command, &OutputFormat::Json)
                .await
                .expect_err("feature-disabled native image tool should not report success");
            let message = err.to_string();

            assert!(message.contains(tool_name), "{message}");
            assert!(message.contains(expected_message), "{message}");
            assert!(message.contains("no tool receipt"), "{message}");
        }

        std::fs::remove_file(input).expect("test fixture cleanup should succeed");
    }

    #[cfg(feature = "image-core")]
    #[tokio::test]
    async fn extended_image_convert_returns_native_receipt_for_json_output() {
        let root = std::env::temp_dir().join(format!("dx-media-cli-image-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("image fixture directory should be created");
        let input = root.join("source.png");
        let output = root.join("converted.jpg");

        let image = image::RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 255]));
        image.save(&input).expect("PNG fixture should be writable");

        let result = run_image_extended(ImageToolsExtended::Convert {
            input: input.clone(),
            output: output.clone(),
            quality: Some(80),
        })
        .expect("extended image CLI should route conversion to native receipt tool");

        assert!(output.exists(), "converted image should be written");
        assert_eq!(
            result.metadata.get("tool.name").map(String::as_str),
            Some("image.convert")
        );
        assert_eq!(
            result
                .metadata
                .get("tool.receipt_completeness")
                .map(String::as_str),
            Some("explicit")
        );
        assert_eq!(
            result
                .metadata
                .get("tool.type_validation")
                .map(String::as_str),
            Some("pass")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn extended_declared_archive_tools_return_errors_without_receipts() {
        let config = MediaConfig::default();
        let commands = [
            (
                "archive.tar",
                ArchiveToolsExtended::Tar {
                    files: vec![fixture_path("input.txt")],
                    output: fixture_path("output.tar"),
                },
            ),
            (
                "archive.untar",
                ArchiveToolsExtended::Untar {
                    input: fixture_path("input.tar"),
                    output: fixture_path("output"),
                },
            ),
            (
                "archive.gzip",
                ArchiveToolsExtended::Gzip {
                    input: fixture_path("input.txt"),
                    output: fixture_path("input.txt.gz"),
                },
            ),
            (
                "archive.gunzip",
                ArchiveToolsExtended::Gunzip {
                    input: fixture_path("input.txt.gz"),
                    output: fixture_path("input.txt"),
                },
            ),
        ];

        for (tool_name, command) in commands {
            let err = execute_archive_extended(command, &config, &OutputFormat::Simple)
                .await
                .expect_err("declared archive tool should not report success");
            let message = err.to_string();

            assert!(message.contains(tool_name), "{message}");
            assert!(message.contains("archive-core"), "{message}");
            assert!(message.contains("no output file"), "{message}");
            assert!(message.contains("no tool receipt"), "{message}");
        }
    }

    #[tokio::test]
    async fn extended_declared_document_tools_return_errors_without_receipts() {
        let commands = {
            let commands = external_document_error_commands();

            #[cfg(feature = "document-core")]
            {
                commands
            }

            #[cfg(not(feature = "document-core"))]
            {
                let mut commands = commands;
                commands.push((
                    "document.markdown-to-html",
                    "document-core",
                    DocumentToolsExtended::MarkdownToHtml {
                        input: fixture_path("input.md"),
                        output: fixture_path("output.html"),
                    },
                ));
                commands.push((
                    "document.extract-text",
                    "document-core",
                    DocumentToolsExtended::ExtractText {
                        input: fixture_path("input.pdf"),
                        output: fixture_path("output.txt"),
                    },
                ));
                commands
            }
        };

        for (tool_name, dependency, command) in commands {
            let err = execute_document_extended(command, &OutputFormat::Simple)
                .await
                .expect_err("declared document tool should not report success");
            let message = err.to_string();

            assert!(message.contains(tool_name), "{message}");
            assert!(message.contains(dependency), "{message}");
            assert!(message.contains("no output file"), "{message}");
            assert!(message.contains("no tool receipt"), "{message}");
        }
    }

    #[cfg(feature = "document-core")]
    #[tokio::test]
    async fn extended_document_markdown_to_html_returns_native_receipt() {
        let input =
            std::env::temp_dir().join(format!("dx-media-cli-document-{}.md", std::process::id()));
        let output = input.with_extension("html");
        std::fs::write(&input, "# CLI Document\n\nReceipt proof.")
            .expect("markdown fixture should be writable");

        let result = run_document_extended(DocumentToolsExtended::MarkdownToHtml {
            input: input.clone(),
            output: output.clone(),
        })
        .expect("extended document CLI should route markdown conversion to native tool");

        assert!(output.exists(), "HTML output should be written");
        assert_eq!(
            result.metadata.get("tool.name").map(String::as_str),
            Some("document.markdown-to-html.native")
        );
        assert_eq!(
            result
                .metadata
                .get("tool.receipt_completeness")
                .map(String::as_str),
            Some("explicit")
        );
        assert_eq!(
            result
                .metadata
                .get("tool.type_validation")
                .map(String::as_str),
            Some("pass")
        );

        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[cfg(feature = "archive-core")]
    #[tokio::test]
    async fn extended_archive_zip_rejects_failed_type_validation() {
        let root =
            std::env::temp_dir().join(format!("dx-media-cli-archive-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("archive fixture directory should be created");
        let input = root.join("source.txt");
        let output = root.join("archive.notzip");
        std::fs::write(&input, "archive me").expect("archive fixture should be writable");

        let config = MediaConfig {
            archive_dir: Some(root.clone()),
            ..MediaConfig::default()
        };

        let err = execute_archive_extended(
            ArchiveToolsExtended::Zip {
                files: vec![input],
                output,
            },
            &config,
            &OutputFormat::Simple,
        )
        .await
        .expect_err("CLI should reject archive outputs with failed type validation");
        let message = err.to_string();

        assert!(message.contains("archive.zip.native"), "{message}");
        assert!(message.contains("type validation"), "{message}");
        assert!(message.contains("archive-extension-mismatch"), "{message}");

        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn extended_utility_missing_inputs_return_errors() {
        let root =
            std::env::temp_dir().join(format!("dx-media-cli-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);

        let cases = vec![
            (
                "utility.hash",
                UtilityToolsExtended::Hash {
                    input: root.join("missing.txt"),
                    algorithm: "sha256".to_string(),
                },
            ),
            (
                "utility.base64-encode",
                UtilityToolsExtended::Base64Encode {
                    input: root.join("missing.bin"),
                },
            ),
            (
                "utility.find-duplicates",
                UtilityToolsExtended::FindDuplicates {
                    directory: root.join("missing-dir"),
                },
            ),
            (
                "utility.verify-checksum",
                UtilityToolsExtended::VerifyChecksum {
                    file: root.join("missing.txt"),
                    checksum: "abc123".to_string(),
                },
            ),
            (
                "utility.json-to-yaml",
                UtilityToolsExtended::JsonToYaml {
                    input: root.join("missing.json"),
                    output: root.join("out.yaml"),
                },
            ),
            (
                "utility.yaml-to-json",
                UtilityToolsExtended::YamlToJson {
                    input: root.join("missing.yaml"),
                    output: root.join("out.json"),
                },
            ),
            (
                "utility.format-json",
                UtilityToolsExtended::FormatJson {
                    input: root.join("missing.json"),
                    output: root.join("formatted.json"),
                },
            ),
        ];

        for (tool_name, command) in cases {
            let err = execute_utility_extended(command, &OutputFormat::Simple)
                .await
                .expect_err("missing utility input should not report success");
            let message = err.to_string();

            assert!(message.contains(tool_name), "{message}");
            assert!(message.contains("missing input"), "{message}");
            assert!(message.contains("no output file"), "{message}");
            assert!(message.contains("no tool receipt"), "{message}");
        }
    }

    #[tokio::test]
    async fn extended_declared_csv_conversion_returns_error_without_receipt() {
        let err = execute_utility_extended(
            UtilityToolsExtended::ConvertCsv {
                input: fixture_path("input.csv"),
                output: fixture_path("output.json"),
                csv_format: "json".to_string(),
            },
            &OutputFormat::Simple,
        )
        .await
        .expect_err("declared CSV conversion should not report success");
        let message = err.to_string();

        assert!(message.contains("utility.convert-csv"), "{message}");
        assert!(message.contains("utility-core"), "{message}");
        assert!(message.contains("no output file"), "{message}");
        assert!(message.contains("no tool receipt"), "{message}");
    }
}
