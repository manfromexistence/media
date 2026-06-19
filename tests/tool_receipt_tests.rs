use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dx_media::tools::{
    ToolCategory, ToolOutput, ToolReadiness, ToolReceipt, ToolReceiptReadiness, ToolSourceKind,
    ToolTypeValidationReadiness, all_tool_descriptors, tool_descriptor_records,
    tool_descriptor_records_for_category,
};
use dx_media::{DownloadUrlKind, Downloader, License, MediaAsset, MediaType, SearchResult};

struct EnvGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let previous = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(previous) = self.previous.take() {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }
}

fn write_fake_magick(dir: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let path = dir.join("magick.cmd");
        std::fs::write(
            &path,
            "@echo off\r\nif \"%1\"==\"-version\" (\r\n  echo fake ImageMagick version\r\n  exit /b 0\r\n)\r\nset \"last=\"\r\n:loop\r\nif \"%~1\"==\"\" goto done\r\nset \"last=%~1\"\r\nshift\r\ngoto loop\r\n:done\r\nif \"%last%\"==\"\" exit /b 2\r\necho fake-image>\"%last%\"\r\nexit /b 0\r\n",
        )
        .expect("fake ImageMagick command should be written");
        path
    }

    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = dir.join("magick");
        std::fs::write(
            &path,
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  echo 'fake ImageMagick version'\n  exit 0\nfi\nlast=\"\"\nfor arg in \"$@\"; do last=\"$arg\"; done\n[ -n \"$last\" ] || exit 2\nprintf 'fake-image\\n' > \"$last\"\nexit 0\n",
        )
        .expect("fake ImageMagick command should be written");
        let mut permissions = std::fs::metadata(&path)
            .expect("fake ImageMagick metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions)
            .expect("fake ImageMagick should be executable");
        path
    }
}

#[test]
fn tool_output_constructors_include_receipt_metadata() {
    let output = ToolOutput::success_with_path("converted", "out.png")
        .with_receipt(ToolReceipt::local("image.convert"))
        .with_output_type_validation("out.png", MediaType::Image);

    assert_eq!(
        output
            .metadata
            .get("tool.receipt_version")
            .map(String::as_str),
        Some("dx-media-tool-receipt-v1")
    );
    assert_eq!(
        output.metadata.get("tool.source_kind").map(String::as_str),
        Some("local-only")
    );
    assert_eq!(
        output.metadata.get("tool.name").map(String::as_str),
        Some("image.convert")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[test]
fn tool_output_default_receipt_records_callsite() {
    let output = ToolOutput::success("generated");

    assert_eq!(
        output
            .metadata
            .get("tool.receipt_completeness")
            .map(String::as_str),
        Some("default")
    );
    assert_eq!(
        output.metadata.get("tool.source_kind").map(String::as_str),
        Some("local-only")
    );
    assert!(
        output
            .metadata
            .get("tool.name")
            .is_some_and(|name| name.contains("tool_receipt_tests"))
    );
    assert!(
        output
            .metadata
            .get("tool.callsite")
            .is_some_and(|callsite| callsite.contains("tool_receipt_tests"))
    );
}

#[test]
fn explicit_receipts_support_provider_fixture_and_credentials_sources() {
    let provider_output = ToolOutput::success("provider asset").with_receipt(
        ToolReceipt::provider_backed("media.search", "openverse")
            .with_license("CC-BY")
            .with_source("https://openverse.org/image/123"),
    );

    assert_eq!(
        provider_output
            .metadata
            .get("tool.receipt_completeness")
            .map(String::as_str),
        Some("explicit")
    );
    assert_eq!(
        provider_output
            .metadata
            .get("tool.source_kind")
            .map(String::as_str),
        Some("provider-backed")
    );
    assert_eq!(
        provider_output
            .metadata
            .get("tool.provider")
            .map(String::as_str),
        Some("openverse")
    );
    assert_eq!(
        provider_output
            .metadata
            .get("tool.license")
            .map(String::as_str),
        Some("CC-BY")
    );

    let fixture_output =
        ToolOutput::success("fixture").with_receipt(ToolReceipt::fixture_backed("media.fixture"));
    assert_eq!(
        fixture_output
            .metadata
            .get("tool.source_kind")
            .map(String::as_str),
        Some("fixture-backed")
    );

    let credential_output = ToolOutput::failure("missing key")
        .with_receipt(ToolReceipt::requires_credentials("audio.transcribe"));
    assert_eq!(
        credential_output
            .metadata
            .get("tool.source_kind")
            .map(String::as_str),
        Some("requires-credentials")
    );
}

#[test]
fn explicit_receipts_support_direct_url_without_provider_identity() {
    let direct_url_output = ToolOutput::success("downloaded").with_receipt(
        ToolReceipt::direct_url("media.download.direct-url")
            .with_license("unknown")
            .with_source("https://example.com/photo.png"),
    );

    assert_eq!(
        direct_url_output
            .metadata
            .get("tool.source_kind")
            .map(String::as_str),
        Some("direct-url")
    );
    assert_eq!(
        direct_url_output
            .metadata
            .get("tool.source")
            .map(String::as_str),
        Some("https://example.com/photo.png")
    );
    assert!(
        !direct_url_output.metadata.contains_key("tool.provider"),
        "direct URLs must not be reported as fake providers"
    );

    let receipt_tool_name = direct_url_output
        .metadata
        .get("tool.name")
        .expect("direct URL receipt should identify the tool name");
    assert!(
        all_tool_descriptors()
            .iter()
            .any(|tool| tool.name == receipt_tool_name),
        "direct URL receipt tool name should match a registry descriptor"
    );
}

#[test]
fn tool_registry_declares_source_kind_and_readiness_for_every_tool() {
    let descriptors = all_tool_descriptors();
    let valid_source_kinds = [
        ToolSourceKind::LocalOnly.as_str(),
        ToolSourceKind::ProviderBacked.as_str(),
        ToolSourceKind::DirectUrl.as_str(),
        ToolSourceKind::FixtureBacked.as_str(),
        ToolSourceKind::RequiresCredentials.as_str(),
    ];
    let valid_readiness = [
        ToolReadiness::Local.as_str(),
        ToolReadiness::DeclaredOnly.as_str(),
        ToolReadiness::FeatureGated.as_str(),
        ToolReadiness::ExternalDependency.as_str(),
        ToolReadiness::RequiresCredentials.as_str(),
    ];
    let valid_receipt_readiness = [
        ToolReceiptReadiness::RuntimeReceipt.as_str(),
        ToolReceiptReadiness::AssetProvenance.as_str(),
        ToolReceiptReadiness::DeclaredOnly.as_str(),
        ToolReceiptReadiness::RequiresCredentials.as_str(),
    ];
    let valid_type_validation = [
        ToolTypeValidationReadiness::Extension.as_str(),
        ToolTypeValidationReadiness::ExtensionPresence.as_str(),
        ToolTypeValidationReadiness::ProviderMetadata.as_str(),
        ToolTypeValidationReadiness::ProviderAndOutput.as_str(),
        ToolTypeValidationReadiness::DeclaredOnly.as_str(),
        ToolTypeValidationReadiness::NotApplicable.as_str(),
        ToolTypeValidationReadiness::RequiresCredentials.as_str(),
    ];

    assert!(descriptors.len() >= 67);
    for tool in descriptors {
        assert!(!tool.name.is_empty());
        assert!(!tool.description.is_empty());
        assert!(
            valid_source_kinds.contains(&tool.source_kind.as_str()),
            "{} has an unknown source kind: {}",
            tool.name,
            tool.source_kind.as_str()
        );
        assert!(
            valid_readiness.contains(&tool.readiness.as_str()),
            "{} has an unknown readiness: {}",
            tool.name,
            tool.readiness.as_str()
        );
        assert!(
            valid_receipt_readiness.contains(&tool.receipt_readiness().as_str()),
            "{} has an unknown receipt readiness: {}",
            tool.name,
            tool.receipt_readiness().as_str()
        );
        assert!(
            valid_type_validation.contains(&tool.type_validation_readiness().as_str()),
            "{} has an unknown type-validation readiness: {}",
            tool.name,
            tool.type_validation_readiness().as_str()
        );
        let description = tool.description.to_ascii_lowercase();
        for marker in ["fake", "stub", "todo", "placeholder"] {
            assert!(
                !description.contains(marker),
                "{} description must not contain placeholder marker `{marker}`: {}",
                tool.name,
                tool.description
            );
        }
    }

    let credential_tools = [
        "audio.transcribe",
        "audio.generate-subtitles",
        "audio.detect-language",
    ];
    for name in credential_tools {
        let descriptor = descriptors
            .iter()
            .find(|tool| tool.name == name)
            .expect("credential-backed audio tool should be discoverable");
        assert_eq!(descriptor.source_kind, ToolSourceKind::ProviderBacked);
        assert_eq!(descriptor.readiness.as_str(), "requires-credentials");
        assert!(
            descriptor.requires_credentials(),
            "credential-backed provider tools must keep access requirements explicit"
        );
        assert_eq!(descriptor.dependency, Some("speech-provider-credentials"));
    }
}

#[test]
fn tool_registry_exports_machine_readable_honest_records() {
    let records = tool_descriptor_records();

    assert_eq!(records.len(), all_tool_descriptors().len());

    let search = records
        .iter()
        .find(|tool| tool.name == "media.search")
        .expect("provider-backed media search should be discoverable");
    assert_eq!(search.category, "media");
    assert_eq!(search.source_kind, "provider-backed");
    assert_eq!(search.readiness, "local");
    assert_eq!(search.feature, None);
    assert_eq!(search.dependency, None);
    assert!(search.input_types.contains(&"query"));
    assert!(search.output_types.contains(&"media-assets"));

    let download = records
        .iter()
        .find(|tool| tool.name == "media.download")
        .expect("provider-backed media download should be discoverable");
    assert_eq!(download.source_kind, "provider-backed");
    assert!(download.output_types.contains(&"tool-receipt"));

    let json = serde_json::to_value(&records).expect("tool records should serialize");
    let first = json
        .as_array()
        .and_then(|items| items.first())
        .expect("tool records should be a non-empty JSON array");
    assert!(first.get("name").is_some());
    assert!(first.get("category").is_some());
    assert!(first.get("source_kind").is_some());
    assert!(first.get("readiness").is_some());
    assert!(first.get("receipt_readiness").is_some());
    assert!(first.get("type_validation_readiness").is_some());
    assert!(first.get("implementation_receipt_names").is_some());
    assert!(first.get("command_paths").is_some());
    assert!(first.get("routes").is_some());
    assert!(first.get("input_types").is_some());
    assert!(first.get("output_types").is_some());

    let api_only_audio_tools = [
        "audio.transcribe",
        "audio.generate-subtitles",
        "audio.detect-language",
        "audio.prepare-for-transcription",
        "audio.extract-speech-segments",
        "audio.analyze-levels",
    ];
    for name in api_only_audio_tools {
        let tool = records
            .iter()
            .find(|tool| tool.name == name)
            .expect("audio API-only tool should be discoverable");
        assert!(
            tool.command_paths.is_empty(),
            "{name} should not advertise a CLI command until one is wired"
        );
    }
    assert!(
        records
            .iter()
            .filter(|tool| !api_only_audio_tools.contains(&tool.name))
            .all(|tool| !tool.command_paths.is_empty()),
        "non API-only tools should keep discoverable CLI paths"
    );

    let declared_only = records
        .iter()
        .find(|tool| tool.name == "image.watermark")
        .expect("declared image watermark should be discoverable");
    assert_eq!(declared_only.receipt_readiness, "declared-only");
    assert_eq!(declared_only.type_validation_readiness, "declared-only");

    let qr = records
        .iter()
        .find(|tool| tool.name == "image.qr")
        .expect("declared image QR command should be discoverable");
    assert_eq!(qr.readiness, "declared-only");
    assert_eq!(qr.receipt_readiness, "declared-only");
    assert_eq!(qr.type_validation_readiness, "declared-only");
    assert_eq!(qr.feature, Some("image-qr"));

    let convert = records
        .iter()
        .find(|tool| tool.name == "image.convert")
        .expect("image convert should be discoverable");
    assert_eq!(convert.receipt_readiness, "runtime-receipt");
    assert_eq!(convert.type_validation_readiness, "extension");
    assert_eq!(convert.dependency, Some("imagemagick"));
    assert!(
        convert.description.contains("ImageMagick"),
        "image.convert should disclose that its compatibility API still uses ImageMagick"
    );
    assert!(
        convert
            .command_paths
            .iter()
            .any(|path| path == "media image convert")
    );
    assert!(
        convert
            .command_paths
            .iter()
            .any(|path| path == "media tools image convert")
    );

    assert!(
        search
            .command_paths
            .iter()
            .any(|path| path == "media search")
    );

    let image_records = tool_descriptor_records_for_category("image");
    assert!(!image_records.is_empty());
    assert!(image_records.iter().all(|tool| tool.category == "image"));
    assert!(!image_records.iter().any(|tool| tool.name == "media.search"));
}

#[test]
fn tool_registry_routes_expose_cli_readiness_without_overclaiming_receipts() {
    let records = tool_descriptor_records();
    let route_for = |tool_name: &str, path: &str| {
        records
            .iter()
            .find(|tool| tool.name == tool_name)
            .unwrap_or_else(|| panic!("{tool_name} should be discoverable"))
            .routes
            .iter()
            .find(|route| route.path == path)
            .unwrap_or_else(|| panic!("{tool_name} should expose route {path}"))
    };

    let image_unified = route_for("image.convert", "media image convert");
    assert_eq!(image_unified.surface, "unified-cli");
    assert_eq!(image_unified.readiness, "feature-gated");
    assert_eq!(image_unified.receipt_readiness, "runtime-receipt");
    assert_eq!(image_unified.type_validation_readiness, "extension");

    let image_legacy = route_for("image.convert", "media tools image convert");
    assert_eq!(image_legacy.surface, "legacy-tools-cli");
    assert_eq!(image_legacy.readiness, "feature-gated");
    assert_eq!(image_legacy.receipt_readiness, "declared-only");
    assert_eq!(image_legacy.type_validation_readiness, "declared-only");

    let video_unwired = route_for("video.transcode", "media video transcode");
    assert_eq!(video_unwired.surface, "unified-cli");
    assert_eq!(video_unwired.readiness, "declared-only");
    assert_eq!(video_unwired.receipt_readiness, "declared-only");
    assert_eq!(video_unwired.type_validation_readiness, "declared-only");

    let audio_convert = route_for("audio.convert", "media audio convert");
    assert_eq!(audio_convert.surface, "unified-cli");
    assert_eq!(audio_convert.readiness, "external-dependency");
    assert_eq!(audio_convert.receipt_readiness, "runtime-receipt");
    assert_eq!(audio_convert.type_validation_readiness, "extension");

    let audio_legacy = route_for("audio.convert", "media tools audio convert");
    assert_eq!(audio_legacy.surface, "legacy-tools-cli");
    assert_eq!(audio_legacy.readiness, "external-dependency");
    assert_eq!(audio_legacy.receipt_readiness, "declared-only");
    assert_eq!(audio_legacy.type_validation_readiness, "declared-only");

    let provider_asset = route_for(
        "media.download",
        "media download --provider <provider> <asset-id>",
    );
    assert_eq!(provider_asset.surface, "unified-cli");
    assert_eq!(provider_asset.readiness, "declared-only");
    assert_eq!(provider_asset.receipt_readiness, "declared-only");
    assert_eq!(provider_asset.type_validation_readiness, "declared-only");
}

#[test]
fn tool_registry_routes_match_every_command_path_with_local_evidence() {
    let records = tool_descriptor_records();

    for record in records {
        if record.api_only {
            assert!(
                record.command_paths.is_empty(),
                "API-only tools should not advertise CLI command paths: {}",
                record.name
            );
            assert!(
                record.routes.is_empty(),
                "API-only tools should not advertise CLI routes: {}",
                record.name
            );
            continue;
        }

        assert!(
            !record.command_paths.is_empty(),
            "CLI tools should advertise command paths: {}",
            record.name
        );
        assert_eq!(
            record.command_paths.len(),
            record.routes.len(),
            "each command path needs route-local evidence: {}",
            record.name
        );

        for path in &record.command_paths {
            let route = record
                .routes
                .iter()
                .find(|route| &route.path == path)
                .unwrap_or_else(|| panic!("missing route metadata for {} at {path}", record.name));

            assert!(
                !route.surface.is_empty()
                    && !route.readiness.is_empty()
                    && !route.receipt_readiness.is_empty()
                    && !route.type_validation_readiness.is_empty(),
                "route-local evidence should be complete for {} at {path}",
                record.name
            );
        }
    }
}

#[test]
fn tool_registry_exposes_runtime_receipt_aliases_without_overclaiming() {
    let records = tool_descriptor_records();
    let aliases_for = |name| {
        records
            .iter()
            .find(|tool| tool.name == name)
            .unwrap_or_else(|| panic!("{name} should be discoverable"))
            .implementation_receipt_names
    };

    assert_eq!(aliases_for("archive.zip"), &["archive.zip.native"][..]);
    assert_eq!(aliases_for("archive.unzip"), &["archive.unzip.native"][..]);
    assert_eq!(
        aliases_for("archive.list"),
        &["archive.list-zip.native"][..]
    );
    assert_eq!(
        aliases_for("document.markdown-to-html"),
        &["document.markdown-to-html.native"][..]
    );
    assert_eq!(
        aliases_for("document.extract-text"),
        &["document.extract-text", "document.extract-text.native"][..]
    );
    assert_eq!(
        aliases_for("image.favicon"),
        &["image.generate-icons-from-svg"][..]
    );
    assert_eq!(aliases_for("image.convert"), &["image.convert"][..]);
    assert_eq!(aliases_for("image.resize"), &["image.resize"][..]);
    assert_eq!(aliases_for("image.compress"), &["image.compress"][..]);
    assert_eq!(aliases_for("image.palette"), &["image.palette"][..]);
    assert_eq!(
        aliases_for("utility.base64-decode"),
        &["utility.base64-decode"][..]
    );
    assert_eq!(
        aliases_for("utility.json-to-yaml"),
        &["utility.json-to-yaml"][..]
    );
    assert_eq!(
        aliases_for("utility.yaml-to-json"),
        &["utility.yaml-to-json"][..]
    );

    for name in [
        "archive.tar",
        "archive.untar",
        "archive.gzip",
        "archive.gunzip",
        "audio.normalize",
        "video.transcode",
        "image.watermark",
    ] {
        assert!(
            aliases_for(name).is_empty(),
            "{name} should not advertise an implementation receipt alias until its listed runtime path is wired"
        );
    }

    for alias in [
        "archive.zip.native",
        "archive.unzip.native",
        "archive.list-zip.native",
        "document.markdown-to-html.native",
        "document.extract-text.native",
        "image.generate-icons-from-svg",
        "image.convert",
        "image.resize",
        "image.compress",
        "image.palette",
        "utility.base64-decode",
        "utility.json-to-yaml",
        "utility.yaml-to-json",
    ] {
        let owners = records
            .iter()
            .filter(|tool| tool.implementation_receipt_names.contains(&alias))
            .count();
        assert_eq!(
            owners, 1,
            "{alias} should resolve to exactly one registry descriptor"
        );
    }
}

#[test]
fn tool_category_filters_reject_unknown_values() {
    assert_eq!(
        ToolCategory::from_filter("Image"),
        Some(ToolCategory::Image)
    );
    assert_eq!(
        ToolCategory::valid_names(),
        vec![
            "media", "image", "video", "audio", "document", "archive", "utility"
        ]
    );
    assert_eq!(ToolCategory::from_filter("nope"), None);
}

#[test]
fn tool_registry_splits_direct_url_and_provider_asset_downloads() {
    let records = tool_descriptor_records();

    let direct_url = records
        .iter()
        .find(|tool| tool.name == "media.download.direct-url")
        .expect("direct URL media download should be discoverable");
    assert_eq!(direct_url.category, "media");
    assert_eq!(direct_url.source_kind, "direct-url");
    assert_eq!(direct_url.readiness, "local");
    assert_eq!(direct_url.receipt_readiness, "runtime-receipt");
    assert_eq!(direct_url.type_validation_readiness, "extension");
    assert!(direct_url.input_types.contains(&"direct-url"));
    assert!(direct_url.output_types.contains(&"file"));
    assert!(direct_url.output_types.contains(&"tool-receipt"));
    assert!(
        direct_url
            .command_paths
            .iter()
            .any(|path| path == "media download <url>")
    );
    assert!(!direct_url.input_types.contains(&"provider-asset-id"));

    let provider_asset = records
        .iter()
        .find(|tool| tool.name == "media.download")
        .expect("provider asset-id media download should be discoverable");
    assert_eq!(provider_asset.source_kind, "provider-backed");
    assert_eq!(provider_asset.readiness, "declared-only");
    assert_eq!(provider_asset.receipt_readiness, "declared-only");
    assert_eq!(provider_asset.type_validation_readiness, "declared-only");
    assert!(provider_asset.input_types.contains(&"provider-name"));
    assert!(provider_asset.input_types.contains(&"provider-asset-id"));
    assert!(
        provider_asset
            .command_paths
            .iter()
            .any(|path| path == "media download --provider <provider> <asset-id>")
    );
    assert!(!provider_asset.input_types.contains(&"direct-url"));
}

#[test]
fn tool_registry_distinguishes_asset_provenance_from_tool_receipts() {
    let records = tool_descriptor_records();
    let search = records
        .iter()
        .find(|tool| tool.name == "media.search")
        .expect("media search should be discoverable");

    assert_eq!(search.source_kind, "provider-backed");
    assert_eq!(search.receipt_readiness, "asset-provenance");
    assert_eq!(search.type_validation_readiness, "provider-metadata");
    assert!(search.output_types.contains(&"provenance"));
    assert!(
        !search.output_types.contains(&"tool-receipt"),
        "search results expose per-asset provenance, not a ToolOutput receipt"
    );
}

#[test]
fn tool_registry_marks_stdout_only_utilities_as_no_runtime_receipt_or_type_validation() {
    let records = tool_descriptor_records();
    let stdout_only_tools = [
        "utility.url-encode",
        "utility.url-decode",
        "utility.uuid",
        "utility.validate-uuid",
        "utility.timestamp",
    ];

    for name in stdout_only_tools {
        let tool = records
            .iter()
            .find(|tool| tool.name == name)
            .expect("stdout utility should be discoverable");

        assert_eq!(tool.source_kind, "local-only", "{name}");
        assert_eq!(tool.receipt_readiness, "declared-only", "{name}");
        assert_eq!(tool.type_validation_readiness, "not-applicable", "{name}");
    }
}

#[test]
fn tool_registry_marks_receipted_metadata_utilities_with_receipt_names() {
    let records = tool_descriptor_records();
    let metadata_tools = [
        ("utility.hash", "utility.hash"),
        ("utility.base64-encode", "utility.base64-encode"),
        ("utility.find-duplicates", "utility.find-duplicates"),
        ("utility.verify-checksum", "utility.verify-checksum"),
    ];

    for (name, receipt_name) in metadata_tools {
        let tool = records
            .iter()
            .find(|tool| tool.name == name)
            .expect("metadata utility should be discoverable");

        assert_eq!(tool.source_kind, "local-only", "{name}");
        assert_eq!(tool.readiness, "local", "{name}");
        assert_eq!(tool.receipt_readiness, "runtime-receipt", "{name}");
        assert_eq!(tool.type_validation_readiness, "not-applicable", "{name}");
        assert!(
            tool.implementation_receipt_names.contains(&receipt_name),
            "{name} should advertise its concrete receipt name"
        );
    }
}

#[test]
fn tool_registry_keeps_favicon_svg_only_until_bitmap_path_is_wired() {
    let records = tool_descriptor_records();
    let favicon = records
        .iter()
        .find(|tool| tool.name == "image.favicon")
        .expect("image.favicon should be discoverable");

    assert_eq!(favicon.input_types, ["svg"]);
    assert!(
        !favicon.description.contains("or image input"),
        "registry must not claim bitmap favicon input until the CLI can handle it"
    );
    assert_eq!(
        favicon.implementation_receipt_names,
        ["image.generate-icons-from-svg"]
    );
}

#[test]
fn tool_registry_does_not_overclaim_unwired_audio_video_receipts() {
    let records = tool_descriptor_records();
    let unwired_external_tools = [
        "video.transcode",
        "video.extract-audio",
        "video.trim",
        "video.scale",
        "video.to-gif",
        "video.thumbnail",
        "video.mute",
        "video.watermark",
        "video.speed",
        "video.concat",
        "video.subtitles",
        "audio.trim",
        "audio.merge",
        "audio.normalize",
        "audio.remove-silence",
        "audio.split",
        "audio.effects",
        "audio.spectrum",
        "audio.metadata",
    ];

    for name in unwired_external_tools {
        let tool = records
            .iter()
            .find(|tool| tool.name == name)
            .expect("declared audio/video tool should be discoverable");

        assert_eq!(tool.source_kind, "local-only", "{name}");
        assert_eq!(tool.readiness, "external-dependency", "{name}");
        assert_eq!(tool.receipt_readiness, "declared-only", "{name}");
        assert_eq!(tool.type_validation_readiness, "declared-only", "{name}");
    }
}

#[test]
fn tool_registry_marks_audio_convert_runtime_receipt_and_extension_validation() {
    let records = tool_descriptor_records();
    let convert = records
        .iter()
        .find(|tool| tool.name == "audio.convert")
        .expect("audio.convert should be discoverable");

    assert_eq!(convert.source_kind, "local-only");
    assert_eq!(convert.readiness, "external-dependency");
    assert_eq!(convert.dependency, Some("ffmpeg"));
    assert_eq!(convert.external_dependency_status, "not-checked");
    assert_eq!(convert.receipt_readiness, "runtime-receipt");
    assert_eq!(convert.type_validation_readiness, "extension");
    assert!(
        convert
            .implementation_receipt_names
            .contains(&"audio.convert"),
        "audio.convert should advertise its runtime receipt name"
    );
    assert!(
        convert
            .command_paths
            .iter()
            .any(|path| path == "media audio convert"),
        "audio.convert should advertise its wired extended CLI path"
    );
    assert!(
        convert
            .command_paths
            .iter()
            .any(|path| path == "media tools audio convert"),
        "audio.convert should advertise the legacy facade with route-local declared-only honesty"
    );
}

#[test]
fn tool_registry_exposes_access_and_dependency_status() {
    let records = tool_descriptor_records();

    let analyze = records
        .iter()
        .find(|tool| tool.name == "audio.analyze-levels")
        .expect("audio.analyze-levels should be discoverable as an API-only tool");
    assert_eq!(analyze.source_kind, "local-only");
    assert_eq!(analyze.readiness, "external-dependency");
    assert!(analyze.api_only);
    assert!(!analyze.requires_credentials);
    assert_eq!(analyze.credential_status, "not-required");
    assert_eq!(analyze.dependency, Some("ffmpeg"));
    assert_eq!(analyze.external_dependency_status, "not-checked");
    assert_eq!(analyze.receipt_readiness, "runtime-receipt");
    assert_eq!(analyze.type_validation_readiness, "not-applicable");
    assert!(analyze.command_paths.is_empty());

    let transcribe = records
        .iter()
        .find(|tool| tool.name == "audio.transcribe")
        .expect("audio.transcribe should remain discoverable");
    assert_eq!(transcribe.source_kind, "provider-backed");
    assert_eq!(transcribe.readiness, "requires-credentials");
    assert!(transcribe.api_only);
    assert!(transcribe.requires_credentials);
    assert_eq!(transcribe.credential_status, "required");
    assert_eq!(
        transcribe.external_dependency_status,
        "requires-credentials"
    );
}

#[test]
fn tool_registry_does_not_overclaim_placeholder_document_archive_utility_receipts() {
    let records = tool_descriptor_records();
    let placeholder_tools = [
        "archive.tar",
        "archive.untar",
        "archive.gzip",
        "archive.gunzip",
        "document.pdf-merge",
        "document.pdf-split",
        "document.pdf-compress",
        "document.pdf-encrypt",
        "document.pdf-watermark",
        "document.pdf-to-image",
        "document.html-to-pdf",
        "utility.convert-csv",
    ];

    for name in placeholder_tools {
        let tool = records
            .iter()
            .find(|tool| tool.name == name)
            .expect("declared placeholder tool should be discoverable");

        assert_eq!(tool.source_kind, "local-only", "{name}");
        if name == "utility.convert-csv" {
            assert_eq!(tool.readiness, "declared-only", "{name}");
            assert_eq!(tool.feature, Some("utility-core"), "{name}");
        }
        assert_eq!(tool.receipt_readiness, "declared-only", "{name}");
        assert_eq!(tool.type_validation_readiness, "declared-only", "{name}");
    }
}

#[test]
fn tool_registry_matches_extended_archive_and_document_cli_honesty() {
    let records = tool_descriptor_records();

    let zip = records
        .iter()
        .find(|tool| tool.name == "archive.zip")
        .expect("archive.zip should be discoverable");
    assert_eq!(zip.source_kind, "local-only");
    assert_eq!(zip.readiness, "feature-gated");
    assert_eq!(zip.feature, Some("archive-core"));
    assert_eq!(zip.receipt_readiness, "runtime-receipt");
    assert_eq!(zip.type_validation_readiness, "extension");

    let unzip = records
        .iter()
        .find(|tool| tool.name == "archive.unzip")
        .expect("archive.unzip should be discoverable");
    assert_eq!(unzip.source_kind, "local-only");
    assert_eq!(unzip.readiness, "feature-gated");
    assert_eq!(unzip.feature, Some("archive-core"));
    assert_eq!(unzip.receipt_readiness, "runtime-receipt");
    assert_eq!(unzip.type_validation_readiness, "extension");

    let list = records
        .iter()
        .find(|tool| tool.name == "archive.list")
        .expect("archive.list should be discoverable");
    assert_eq!(list.source_kind, "local-only");
    assert_eq!(list.readiness, "feature-gated");
    assert_eq!(list.feature, Some("archive-core"));
    assert_eq!(list.input_types, &["zip"]);
    assert_eq!(list.output_types, &["metadata"]);
    assert_eq!(list.receipt_readiness, "runtime-receipt");
    assert_eq!(list.type_validation_readiness, "extension");

    let markdown_to_html = records
        .iter()
        .find(|tool| tool.name == "document.markdown-to-html")
        .expect("document.markdown-to-html should be discoverable");
    assert_eq!(markdown_to_html.source_kind, "local-only");
    assert_eq!(markdown_to_html.readiness, "feature-gated");
    assert_eq!(markdown_to_html.feature, Some("document-core"));
    assert_eq!(markdown_to_html.dependency, None);
    assert_eq!(markdown_to_html.receipt_readiness, "runtime-receipt");
    assert_eq!(markdown_to_html.type_validation_readiness, "extension");

    let extract_text = records
        .iter()
        .find(|tool| tool.name == "document.extract-text")
        .expect("document.extract-text should be discoverable");
    assert_eq!(extract_text.source_kind, "local-only");
    assert_eq!(extract_text.readiness, "external-dependency");
    assert_eq!(extract_text.feature, Some("document-core"));
    assert_eq!(
        extract_text.dependency,
        Some("pdftotext/xpdf/tika/antiword/docx2txt/libreoffice")
    );
    assert_eq!(extract_text.external_dependency_status, "not-checked");
    assert_eq!(extract_text.receipt_readiness, "runtime-receipt");
    assert_eq!(extract_text.type_validation_readiness, "extension");

    let format_json = records
        .iter()
        .find(|tool| tool.name == "utility.format-json")
        .expect("utility.format-json should be discoverable");
    assert_eq!(format_json.source_kind, "local-only");
    assert_eq!(format_json.readiness, "local");
    assert_eq!(format_json.receipt_readiness, "runtime-receipt");
    assert_eq!(format_json.type_validation_readiness, "extension");
    assert!(
        format_json
            .implementation_receipt_names
            .contains(&"utility.format-json"),
        "utility.format-json should advertise the receipt name used by its runtime path"
    );

    let base64_decode = records
        .iter()
        .find(|tool| tool.name == "utility.base64-decode")
        .expect("utility.base64-decode should be discoverable");
    assert_eq!(base64_decode.source_kind, "local-only");
    assert_eq!(base64_decode.readiness, "local");
    assert_eq!(base64_decode.receipt_readiness, "runtime-receipt");
    assert_eq!(
        base64_decode.type_validation_readiness,
        "extension-presence"
    );
    assert!(
        base64_decode
            .implementation_receipt_names
            .contains(&"utility.base64-decode"),
        "utility.base64-decode should advertise the receipt name used by its runtime path"
    );

    for name in ["utility.json-to-yaml", "utility.yaml-to-json"] {
        let tool = records
            .iter()
            .find(|tool| tool.name == name)
            .expect("utility conversion tool should be discoverable");
        assert_eq!(tool.source_kind, "local-only", "{name}");
        assert_eq!(tool.readiness, "local", "{name}");
        assert_eq!(tool.receipt_readiness, "runtime-receipt", "{name}");
        assert_eq!(tool.type_validation_readiness, "extension", "{name}");
        assert!(
            tool.implementation_receipt_names.contains(&name),
            "{name} should advertise the receipt name used by its runtime path"
        );
    }
}

#[test]
fn tool_registry_runtime_receipt_tools_advertise_concrete_receipt_names() {
    let records = tool_descriptor_records();

    for tool in records
        .iter()
        .filter(|tool| tool.receipt_readiness == "runtime-receipt")
    {
        assert!(
            !tool.implementation_receipt_names.is_empty(),
            "{} should advertise at least one concrete runtime receipt name",
            tool.name
        );
    }
}

#[test]
fn media_asset_rejects_empty_required_urls() {
    let missing_download = MediaAsset::builder()
        .id("asset-1")
        .provider("fixture")
        .media_type(MediaType::Image)
        .title("Fixture")
        .download_url("")
        .source_url("https://example.com/source")
        .build();

    assert!(missing_download.is_err());

    let missing_source = MediaAsset::builder()
        .id("asset-1")
        .provider("fixture")
        .media_type(MediaType::Image)
        .title("Fixture")
        .download_url("https://example.com/image.jpg")
        .source_url(" ")
        .build();

    assert!(missing_source.is_err());
}

#[test]
fn media_asset_provenance_preserves_provider_metadata() {
    let mut provider_metadata = HashMap::new();
    provider_metadata.insert("openverse.source".to_string(), "wikimedia".to_string());
    provider_metadata.insert(
        "openverse.license_url".to_string(),
        "https://creativecommons.org/licenses/by/4.0/".to_string(),
    );

    let asset = MediaAsset::builder()
        .id("abc")
        .provider("openverse")
        .media_type(MediaType::Image)
        .title("Image")
        .direct_download_url("https://example.com/image.jpg")
        .source_url("https://example.com/source")
        .license(License::CcBy)
        .mime_type("image/jpeg")
        .provider_metadata(provider_metadata)
        .build()
        .expect("valid asset should build");

    let provenance = asset.provenance();

    assert!(provenance.license_known);
    assert_eq!(
        provenance
            .provider_metadata
            .get("openverse.source")
            .map(String::as_str),
        Some("wikimedia")
    );
    assert!(provenance.type_validation.is_valid());
}

#[test]
fn media_asset_provenance_labels_mime_evidence_source() {
    let asset = MediaAsset::builder()
        .id("abc")
        .provider("fixture")
        .media_type(MediaType::Image)
        .title("Image")
        .download_url("https://example.com/image.jpg")
        .source_url("https://example.com/source")
        .license(License::CcBy)
        .mime_type("image/jpeg")
        .build()
        .expect("valid asset should build");

    let provenance_json = serde_json::to_value(asset.provenance()).unwrap();

    assert_eq!(provenance_json["mime_evidence_source"], "provider-supplied");
    assert_eq!(
        provenance_json["type_validation"]["mime_evidence_source"],
        "provider-supplied"
    );
}

#[test]
fn media_asset_provenance_labels_download_url_kind() {
    let asset = MediaAsset::builder()
        .id("archive-item")
        .provider("archive")
        .media_type(MediaType::Document)
        .title("Archive Item")
        .download_url("https://archive.org/download/archive-item")
        .download_url_kind(DownloadUrlKind::AssetManifest)
        .source_url("https://archive.org/details/archive-item")
        .license(License::Other("Various".to_string()))
        .build()
        .expect("valid asset should build");

    let provenance_json = serde_json::to_value(asset.provenance()).unwrap();

    assert_eq!(asset.download_url_kind, DownloadUrlKind::AssetManifest);
    assert_eq!(provenance_json["download_url_kind"], "asset-manifest");
}

#[test]
fn media_asset_provenance_marks_missing_download_url_kind_as_unknown() {
    let asset = MediaAsset::builder()
        .id("ambiguous-item")
        .provider("ambiguous-provider")
        .media_type(MediaType::Image)
        .title("Ambiguous Item")
        .download_url("https://example.com/asset")
        .source_url("https://example.com/source")
        .build()
        .expect("valid asset should build");

    let provenance_json = serde_json::to_value(asset.provenance()).unwrap();

    assert_eq!(asset.download_url_kind, DownloadUrlKind::Unknown);
    assert_eq!(provenance_json["download_url_kind"], "unknown");
}

#[test]
fn search_result_output_includes_asset_provenance() {
    let mut provider_metadata = HashMap::new();
    provider_metadata.insert("openverse.source".to_string(), "wikimedia".to_string());
    provider_metadata.insert(
        "openverse.foreign_landing_url".to_string(),
        "https://commons.wikimedia.org/wiki/File:nebula.jpg".to_string(),
    );

    let asset = MediaAsset::builder()
        .id("search-asset-1")
        .provider("openverse")
        .media_type(MediaType::Image)
        .title("Nebula")
        .download_url("https://images.example.test/nebula.jpg")
        .source_url("https://openverse.org/image/search-asset-1")
        .author("Wikimedia User")
        .author_url("https://commons.wikimedia.org/wiki/User:Example")
        .license(License::CcBy)
        .mime_type("image/jpeg")
        .provider_metadata(provider_metadata)
        .build()
        .unwrap();

    let mut result = SearchResult::for_type("nebula", MediaType::Image);
    result.total_count = 1;
    result.providers_searched.push("openverse".to_string());
    result.provider_timings.insert("openverse".to_string(), 42);
    result.assets.push(asset);

    let output = result.with_asset_provenance();
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(json["query"], "nebula");
    assert_eq!(json["assets"][0]["id"], "search-asset-1");
    assert_eq!(
        json["assets"][0]["provenance"]["source_url"],
        "https://openverse.org/image/search-asset-1"
    );
    assert_eq!(
        json["assets"][0]["provenance"]["provider_metadata"]["openverse.source"],
        "wikimedia"
    );
    assert_eq!(json["assets"][0]["provenance"]["license_known"], true);
    assert_eq!(
        json["assets"][0]["provenance"]["type_validation"]["mime_matches"],
        true
    );
    assert_eq!(
        json["assets"][0]["provenance"]["type_validation"]["extension_matches"],
        true
    );
}

#[test]
fn download_receipt_preserves_asset_provenance_and_type_evidence() {
    let mut provider_metadata = HashMap::new();
    provider_metadata.insert("openverse.source".to_string(), "wikimedia".to_string());
    provider_metadata.insert(
        "openverse.license_url".to_string(),
        "https://creativecommons.org/licenses/by/4.0/".to_string(),
    );

    let asset = MediaAsset::builder()
        .id("abc")
        .provider("openverse")
        .media_type(MediaType::Image)
        .title("Image")
        .direct_download_url("https://example.com/image.jpg")
        .source_url("https://example.com/source")
        .license(License::CcBy)
        .mime_type("image/jpeg")
        .provider_metadata(provider_metadata)
        .build()
        .expect("valid asset should build");

    let output = Downloader::download_receipt_for_asset(
        &asset,
        Path::new("downloads/openverse-abc.jpg"),
        Some("image/jpeg"),
        42,
    );

    assert_eq!(
        output.metadata.get("tool.name").map(String::as_str),
        Some("media.download")
    );
    assert_eq!(
        output.metadata.get("tool.source_kind").map(String::as_str),
        Some("provider-backed")
    );
    assert_eq!(
        output.metadata.get("tool.provider").map(String::as_str),
        Some("openverse")
    );
    assert_eq!(
        output.metadata.get("tool.source").map(String::as_str),
        Some("https://example.com/source")
    );
    assert_eq!(
        output.metadata.get("tool.download_url").map(String::as_str),
        Some("https://example.com/image.jpg")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.download_url_kind")
            .map(String::as_str),
        Some("direct-file")
    );
    assert_eq!(
        output.metadata.get("tool.license").map(String::as_str),
        Some("CC-BY")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.license_known")
            .map(String::as_str),
        Some("true")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.actual_mime_type")
            .map(String::as_str),
        Some("image/jpeg")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.provider_mime_evidence_source")
            .map(String::as_str),
        Some("provider-supplied")
    );
    assert_eq!(
        output
            .metadata
            .get("provider.openverse.source")
            .map(String::as_str),
        Some("wikimedia")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.provider_type_validation")
            .map(String::as_str),
        Some("pass")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.bytes_written")
            .map(String::as_str),
        Some("42")
    );
}

#[test]
fn download_receipt_records_actual_mime_extension_mismatch() {
    let asset = MediaAsset::builder()
        .id("png-1")
        .provider("fixture")
        .media_type(MediaType::Image)
        .title("PNG bytes")
        .direct_download_url("https://example.com/image")
        .source_url("https://example.com/source")
        .license(License::Cc0)
        .build()
        .expect("valid asset should build");

    let output = Downloader::download_receipt_for_asset(
        &asset,
        Path::new("downloads/fixture-png-1.jpg"),
        Some("image/png"),
        42,
    );

    assert_eq!(
        output
            .metadata
            .get("tool.actual_mime_type")
            .map(String::as_str),
        Some("image/png")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.actual_mime_extension_matches")
            .map(String::as_str),
        Some("false")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("fail")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.actual_file_validation")
            .map(String::as_str),
        Some("fail")
    );
}

#[test]
fn ambiguous_provider_license_labels_are_not_marked_known() {
    let asset = MediaAsset::builder()
        .id("gif-1")
        .provider("giphy")
        .media_type(MediaType::Gif)
        .title("Gif")
        .download_url("https://media.giphy.com/media/example/giphy.gif")
        .source_url("https://giphy.com/gifs/example")
        .license(License::Other("Giphy".to_string()))
        .mime_type("image/gif")
        .build()
        .expect("valid asset should build");

    assert!(!asset.provenance().license_known);
}

#[test]
fn unverified_rights_labels_are_not_marked_known() {
    let labels = [
        "Unknown - Check source",
        "Rights status not verified",
        "License not verified by provider response",
        "Usage rights not provided",
        "Varies by provider style metadata",
    ];

    for label in labels {
        let asset = MediaAsset::builder()
            .id("rights-1")
            .provider("fixture")
            .media_type(MediaType::Image)
            .title("Unverified rights")
            .download_url("https://example.com/image.jpg")
            .source_url("https://example.com/source")
            .license(License::Other(label.to_string()))
            .mime_type("image/jpeg")
            .build()
            .expect("valid asset should build");

        assert!(
            !asset.provenance().license_known,
            "{label:?} should not be receipt-known"
        );
    }
}

#[test]
fn media_type_validation_without_evidence_is_not_valid() {
    let asset = MediaAsset::builder()
        .id("asset-1")
        .provider("fixture")
        .media_type(MediaType::Image)
        .title("Image without type evidence")
        .download_url("https://example.com/download")
        .source_url("https://example.com/source")
        .build()
        .expect("valid asset should build");

    let validation = asset.validate_type_metadata();

    assert!(!validation.has_evidence());
    assert!(!validation.is_valid());
}

#[test]
fn document_type_accepts_openxml_mime_evidence() {
    let docx_mime = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
    let xlsx_mime = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
    let pptx_mime = "application/vnd.openxmlformats-officedocument.presentationml.presentation";

    assert!(MediaType::Document.matches_mime(docx_mime));
    assert!(MediaType::Document.matches_mime(xlsx_mime));
    assert!(MediaType::Document.matches_mime(pptx_mime));
    assert!(MediaType::Document.matches_mime(&format!("{docx_mime}; charset=binary")));
    assert!(!MediaType::Document.matches_mime("application/vnd.openxmlformats-officedocumentevil"));

    let asset = MediaAsset::builder()
        .id("doc-1")
        .provider("fixture")
        .media_type(MediaType::Document)
        .title("Document with DOCX evidence")
        .download_url("https://example.com/report.docx")
        .source_url("https://example.com/source")
        .mime_type(docx_mime)
        .build()
        .expect("valid document asset should build");

    let validation = &asset.provenance().type_validation;

    assert_eq!(validation.mime_matches, Some(true));
    assert_eq!(validation.extension_matches, Some(true));
    assert!(validation.is_valid());
}

#[test]
fn generic_image_type_rejects_modeled_gif_and_vector_mime_evidence() {
    assert!(!MediaType::Image.matches_mime("image/gif"));
    assert!(!MediaType::Image.matches_mime("image/gif; charset=binary"));
    assert!(!MediaType::Image.matches_mime("image/svg+xml"));
    assert_eq!(
        MediaType::Image.mime_exclusions(),
        &["image/gif", "image/svg+xml"]
    );
    assert!(MediaType::Gif.matches_mime("image/gif"));
    assert!(MediaType::Vector.matches_mime("image/svg+xml"));

    let asset = MediaAsset::builder()
        .id("asset-1")
        .provider("fixture")
        .media_type(MediaType::Image)
        .title("Generic image with GIF evidence")
        .download_url("https://example.com/asset.gif")
        .source_url("https://example.com/source")
        .mime_type("image/gif")
        .build()
        .expect("asset with mismatched type evidence should still build with failed validation");
    let validation = &asset.provenance().type_validation;

    assert_eq!(validation.extension_matches, Some(false));
    assert_eq!(validation.mime_matches, Some(false));
    assert!(!validation.is_valid());

    let svg_asset = MediaAsset::builder()
        .id("asset-2")
        .provider("fixture")
        .media_type(MediaType::Image)
        .title("Generic image with SVG evidence")
        .download_url("https://example.com/asset.svg")
        .source_url("https://example.com/source")
        .mime_type("image/svg+xml")
        .build()
        .expect("asset with mismatched SVG evidence should still build with failed validation");
    let svg_validation = &svg_asset.provenance().type_validation;

    assert_eq!(svg_validation.extension_matches, Some(false));
    assert_eq!(svg_validation.mime_matches, Some(false));
    assert!(!svg_validation.is_valid());
}

#[test]
fn output_type_validation_records_extension_mismatch() {
    let output = ToolOutput::success_with_path("bad extension", "out.txt")
        .with_output_type_validation(Path::new("out.txt"), MediaType::Image);

    assert_eq!(
        output
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("fail")
    );
}

#[test]
fn imagemagick_converter_records_receipt_dependency_source_and_type_validation() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let magick = write_fake_magick(dir.path());
    let _magick_guard = EnvGuard::set("DX_MEDIA_MAGICK_BIN", magick.as_os_str());
    let input = dir.path().join("source.png");
    let output = dir.path().join("converted.jpg");
    std::fs::write(&input, b"fake-image-input").expect("input image fixture should be written");

    let result = dx_media::tools::image::converter::convert(&input, &output)
        .expect("ImageMagick converter should use fake command successfully");
    let input_source = input.display().to_string();

    assert!(result.success);
    assert!(output.exists());
    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("image.convert")
    );
    assert_eq!(
        result.metadata.get("tool.source_kind").map(String::as_str),
        Some("local-only")
    );
    assert_eq!(
        result.metadata.get("tool.source").map(String::as_str),
        Some(input_source.as_str())
    );
    assert_eq!(
        result.metadata.get("tool.dependency").map(String::as_str),
        Some("imagemagick")
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
}

#[cfg(feature = "image-core")]
#[test]
fn image_palette_receipt_is_metadata_only_and_validates_input_separately() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("palette-source.png");
    let image = image::RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 255]));
    image.save(&input).expect("PNG fixture should be writable");

    let output = dx_media::tools::image::native::extract_palette_native(&input, 2)
        .expect("palette extraction should succeed");

    assert!(output.success);
    assert!(
        output.output_paths.is_empty(),
        "palette extraction returns metadata, not a generated artifact path"
    );
    assert_eq!(
        output.metadata.get("tool.name").map(String::as_str),
        Some("image.palette")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.expected_media_type")
            .map(String::as_str),
        Some("metadata")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("not-applicable")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.input_extension")
            .map(String::as_str),
        Some("png")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.input_type_validation")
            .map(String::as_str),
        Some("pass")
    );
    assert!(
        !output.metadata.contains_key("tool.output_extension"),
        "metadata-only palette receipts should not treat the input as an output"
    );
}

#[test]
fn audio_type_validation_accepts_all_audio_convert_output_extensions() {
    for extension in ["mp3", "wav", "flac", "ogg", "aac", "m4a", "wma", "opus"] {
        let path = format!("out.{extension}");
        let output = ToolOutput::success_with_path("audio output", &path)
            .with_output_type_validation(Path::new(&path), MediaType::Audio);

        assert_eq!(
            output
                .metadata
                .get("tool.type_validation")
                .map(String::as_str),
            Some("pass"),
            "{extension} should validate as audio"
        );
    }
}
