#[cfg(feature = "cli")]
use dx_media::cli_unified::args::OutputFormat;
#[cfg(feature = "cli")]
use dx_media::cli_unified::args_extended::UtilityToolsExtended;
#[cfg(feature = "cli")]
use dx_media::cli_unified::commands::tools_extended::execute_utility_extended;
use dx_media::tools::utility::base64::{decode_file, decode_string, decode_string_to_file};
use dx_media::tools::utility::json_format::format_json_file;
use dx_media::tools::utility::yaml_convert::{json_to_yaml, yaml_to_json};

#[test]
fn utility_format_json_records_source_receipt_and_json_type_validation() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.json");
    let output = dir.path().join("formatted.json");
    std::fs::write(&input, r#"{"name":"dx","tools":["media"]}"#)
        .expect("fixture json should be written");

    let result = format_json_file(&input, &output).expect("json formatting should succeed");
    let input_source = input.display().to_string();

    assert!(result.success);
    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("utility.format-json")
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
    assert!(
        result.metadata.contains_key("tool.converter"),
        "conversion receipts should identify the converter implementation"
    );
}

#[test]
fn utility_format_json_marks_non_json_output_type_validation_failure() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.json");
    let output = dir.path().join("formatted.txt");
    std::fs::write(&input, r#"{"name":"dx"}"#).expect("fixture json should be written");

    let result = format_json_file(&input, &output).expect("json formatting should succeed");

    assert!(result.success);
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("fail")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation_reason")
            .map(String::as_str),
        Some("extension-mismatch")
    );
}

#[test]
fn utility_format_json_rejects_broad_data_extensions_for_json_output() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.json");
    let output = dir.path().join("formatted.yaml");
    std::fs::write(&input, r#"{"name":"dx"}"#).expect("fixture json should be written");

    let result = format_json_file(&input, &output).expect("json formatting should succeed");

    assert!(result.success);
    assert_eq!(
        result
            .metadata
            .get("tool.expected_media_type")
            .map(String::as_str),
        Some("json")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("fail")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation_reason")
            .map(String::as_str),
        Some("extension-mismatch")
    );
}

#[test]
fn utility_format_json_rejects_invalid_json_before_receipted_output() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("broken.json");
    let output = dir.path().join("formatted.json");
    std::fs::write(&input, r#"{"name":"dx""#).expect("fixture json should be written");

    let err = format_json_file(&input, &output).expect_err("invalid json should fail");
    let message = err.to_string();

    assert!(
        message.contains("Failed to parse JSON response"),
        "{message}"
    );
    assert!(
        !output.exists(),
        "invalid JSON should not produce a receipted output file"
    );
}

#[tokio::test]
#[cfg(feature = "cli")]
async fn extended_utility_format_json_rejects_failed_type_validation() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.json");
    let output = dir.path().join("formatted.txt");
    std::fs::write(&input, r#"{"name":"dx"}"#).expect("fixture json should be written");

    let err = execute_utility_extended(
        UtilityToolsExtended::FormatJson {
            input,
            output: output.clone(),
        },
        &OutputFormat::Table,
    )
    .await
    .expect_err("CLI should reject failed output type validation");
    let message = err.to_string();

    assert!(
        message.contains("utility.format-json failed type validation"),
        "{message}"
    );
    assert!(message.contains("extension-mismatch"), "{message}");
    assert!(
        !output.exists(),
        "CLI should not leave a rejected JSON formatting artifact"
    );
}

#[tokio::test]
#[cfg(feature = "cli")]
async fn extended_utility_format_json_writes_valid_json_output() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.json");
    let output = dir.path().join("formatted.json");
    std::fs::write(&input, r#"{"name":"dx","trusted":true}"#)
        .expect("fixture json should be written");

    execute_utility_extended(
        UtilityToolsExtended::FormatJson {
            input,
            output: output.clone(),
        },
        &OutputFormat::Table,
    )
    .await
    .expect("CLI should write validated JSON output");

    let written = std::fs::read_to_string(&output).expect("formatted json should be readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&written).expect("formatted output should remain valid JSON");

    assert_eq!(parsed["name"], "dx");
    assert_eq!(parsed["trusted"], true);
    assert!(
        written.contains('\n'),
        "formatted JSON output should be pretty-printed"
    );
}

#[tokio::test]
#[cfg(feature = "cli")]
async fn extended_utility_base64_decode_rejects_extensionless_output_before_write() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let output = dir.path().join("decoded");

    let err = execute_utility_extended(
        UtilityToolsExtended::Base64Decode {
            input: "ZHg=".to_string(),
            output: output.clone(),
        },
        &OutputFormat::Table,
    )
    .await
    .expect_err("CLI should reject extensionless decoded file output");
    let message = err.to_string();

    assert!(
        message.contains("utility.base64-decode failed type validation"),
        "{message}"
    );
    assert!(message.contains("missing-output-extension"), "{message}");
    assert!(
        !output.exists(),
        "CLI should not leave a rejected decoded output artifact"
    );
}

#[tokio::test]
#[cfg(feature = "cli")]
async fn extended_utility_base64_decode_writes_valid_output() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let output = dir.path().join("decoded.bin");

    execute_utility_extended(
        UtilityToolsExtended::Base64Decode {
            input: "ZHg=".to_string(),
            output: output.clone(),
        },
        &OutputFormat::Table,
    )
    .await
    .expect("CLI should write decoded output when type validation is acceptable");

    let written = std::fs::read(&output).expect("decoded output should be readable");

    assert_eq!(written, b"dx");
}

#[test]
fn utility_json_to_yaml_records_source_receipt_and_yaml_type_validation() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.json");
    let output = dir.path().join("converted.yaml");
    std::fs::write(&input, r#"{"name":"dx","trusted":true}"#)
        .expect("fixture json should be written");

    let result = json_to_yaml(&input, &output).expect("json to yaml should succeed");
    let input_source = input.display().to_string();

    assert!(result.success);
    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("utility.json-to-yaml")
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
        result
            .metadata
            .get("tool.expected_media_type")
            .map(String::as_str),
        Some("yaml")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[tokio::test]
#[cfg(feature = "cli")]
async fn extended_utility_json_to_yaml_writes_yaml_output() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.json");
    let output = dir.path().join("converted.yaml");
    std::fs::write(&input, r#"{"name":"dx","trusted":true}"#)
        .expect("fixture json should be written");

    execute_utility_extended(
        UtilityToolsExtended::JsonToYaml {
            input,
            output: output.clone(),
        },
        &OutputFormat::Table,
    )
    .await
    .expect("CLI should write validated YAML output");

    let written = std::fs::read_to_string(&output).expect("YAML output should be readable");

    assert!(written.contains("name"), "{written}");
    assert!(written.contains("dx"), "{written}");
    assert!(written.contains("trusted"), "{written}");
}

#[test]
fn utility_json_to_yaml_rejects_broad_data_extensions_for_yaml_output() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.json");
    let output = dir.path().join("converted.json");
    std::fs::write(&input, r#"{"name":"dx"}"#).expect("fixture json should be written");

    let result = json_to_yaml(&input, &output).expect("json to yaml should succeed");

    assert_eq!(
        result
            .metadata
            .get("tool.expected_media_type")
            .map(String::as_str),
        Some("yaml")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("fail")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation_reason")
            .map(String::as_str),
        Some("extension-mismatch")
    );
}

#[test]
fn utility_yaml_to_json_records_source_receipt_and_json_type_validation() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.yaml");
    let output = dir.path().join("converted.json");
    std::fs::write(&input, "name: dx\ntrusted: true\n").expect("fixture yaml should be written");

    let result = yaml_to_json(&input, &output).expect("yaml to json should succeed");
    let input_source = input.display().to_string();

    assert!(result.success);
    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("utility.yaml-to-json")
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
        result
            .metadata
            .get("tool.expected_media_type")
            .map(String::as_str),
        Some("json")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
    assert!(
        result.metadata.contains_key("tool.converter"),
        "conversion receipts should identify the converter implementation"
    );
}

#[tokio::test]
#[cfg(feature = "cli")]
async fn extended_utility_yaml_to_json_writes_json_output() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.yaml");
    let output = dir.path().join("converted.json");
    std::fs::write(&input, "name: dx\ntrusted: true\n").expect("fixture yaml should be written");

    execute_utility_extended(
        UtilityToolsExtended::YamlToJson {
            input,
            output: output.clone(),
        },
        &OutputFormat::Table,
    )
    .await
    .expect("CLI should write validated JSON output");

    let written = std::fs::read_to_string(&output).expect("JSON output should be readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&written).expect("converted output should be valid JSON");

    assert_eq!(parsed["name"], "dx");
    assert_eq!(parsed["trusted"], true);
}

#[test]
fn utility_base64_decode_file_records_source_receipt_decoder_and_extension_presence() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("encoded.txt");
    let output = dir.path().join("decoded.bin");
    std::fs::write(&input, "ZHg=").expect("fixture base64 should be written");

    let result = decode_file(&input, &output).expect("base64 decode should succeed");
    let input_source = input.display().to_string();

    assert!(result.success);
    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("utility.base64-decode")
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
        result
            .metadata
            .get("tool.expected_media_type")
            .map(String::as_str),
        Some("file")
    );
    assert_eq!(
        result.metadata.get("tool.decoder").map(String::as_str),
        Some("rust-fallback")
    );
    assert_eq!(
        result.metadata.get("tool.decoder_kind").map(String::as_str),
        Some("builtin")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("unknown")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation_reason")
            .map(String::as_str),
        Some("decoded-bytes-not-content-validated")
    );
}

#[test]
fn utility_base64_decode_string_to_file_rejects_incomplete_input_before_write() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let output = dir.path().join("decoded.bin");

    let err = decode_string_to_file("Z", &output).expect_err("incomplete base64 should fail");
    let message = err.to_string();

    assert!(message.contains("Invalid base64 length"), "{message}");
    assert!(
        !output.exists(),
        "invalid base64 should not produce a receipted output file"
    );
}

#[test]
fn utility_base64_decode_string_records_inline_receipt_as_text_output() {
    let result = decode_string("ZHg=").expect("base64 decode string should succeed");

    assert!(result.success);
    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("utility.base64-decode")
    );
    assert_eq!(
        result.metadata.get("tool.source").map(String::as_str),
        Some("inline-base64")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.expected_media_type")
            .map(String::as_str),
        Some("text")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("not-applicable")
    );
}

#[tokio::test]
#[cfg(feature = "cli")]
async fn extended_utility_json_to_yaml_rejects_failed_type_validation() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.json");
    let output = dir.path().join("converted.json");
    std::fs::write(&input, r#"{"name":"dx"}"#).expect("fixture json should be written");

    let err = execute_utility_extended(
        UtilityToolsExtended::JsonToYaml {
            input,
            output: output.clone(),
        },
        &OutputFormat::Table,
    )
    .await
    .expect_err("CLI should reject failed output type validation");
    let message = err.to_string();

    assert!(
        message.contains("utility.json-to-yaml failed type validation"),
        "{message}"
    );
    assert!(message.contains("extension-mismatch"), "{message}");
    assert!(
        !output.exists(),
        "CLI should not leave a rejected YAML output artifact"
    );
}

#[tokio::test]
#[cfg(feature = "cli")]
async fn extended_utility_yaml_to_json_rejects_failed_type_validation() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.yaml");
    let output = dir.path().join("converted.yaml");
    std::fs::write(&input, "name: dx\n").expect("fixture yaml should be written");

    let err = execute_utility_extended(
        UtilityToolsExtended::YamlToJson {
            input,
            output: output.clone(),
        },
        &OutputFormat::Table,
    )
    .await
    .expect_err("CLI should reject failed output type validation");
    let message = err.to_string();

    assert!(
        message.contains("utility.yaml-to-json failed type validation"),
        "{message}"
    );
    assert!(message.contains("extension-mismatch"), "{message}");
    assert!(
        !output.exists(),
        "CLI should not leave a rejected JSON output artifact"
    );
}
