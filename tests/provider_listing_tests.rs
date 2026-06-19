use std::process::{Command, Output};

use serde_json::Value;

fn run_command(binary_path: &str, args: &[&str]) -> Output {
    Command::new(binary_path)
        .args(args)
        .output()
        .expect("provider listing command should run")
}

fn run_json(binary_path: &str, args: &[&str]) -> Vec<Value> {
    let output = run_command(binary_path, args);

    assert!(
        output.status.success(),
        "provider listing failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("provider listing should emit a JSON array")
}

fn assert_provider_listing_schema(rows: &[Value]) {
    for row in rows {
        let requires_api_key = row
            .get("requires_api_key")
            .and_then(Value::as_bool)
            .expect("provider rows must expose requires_api_key");
        let requires_credentials = row
            .get("requires_credentials")
            .and_then(Value::as_bool)
            .expect("provider rows must expose requires_credentials");

        assert_eq!(
            requires_api_key, requires_credentials,
            "provider credential aliases must stay consistent in {row:?}"
        );
        assert!(
            row.get("source_kind").and_then(Value::as_str).is_some(),
            "provider rows must expose source_kind"
        );
        assert!(
            row.get("credential_status")
                .and_then(Value::as_str)
                .is_some(),
            "provider rows must expose credential_status"
        );
    }
}

fn assert_credentialed_providers_are_provider_backed(rows: &[Value]) {
    let credentialed_rows = rows.iter().filter(|row| {
        row.get("requires_credentials")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });

    let mut found_credentialed_row = false;
    for row in credentialed_rows {
        found_credentialed_row = true;
        assert_eq!(
            row.get("source_kind").and_then(Value::as_str),
            Some("provider-backed"),
            "credential requirements must not replace provider provenance in {row:?}"
        );
        assert!(
            row.get("credential_status")
                .and_then(Value::as_str)
                .is_some(),
            "credentialed provider rows should expose credential_status separately"
        );
    }

    assert!(
        found_credentialed_row,
        "fixture-free provider registry should include credential-gated providers"
    );
}

#[test]
fn legacy_provider_json_keeps_source_kind_separate_from_credentials() {
    let rows = run_json(env!("CARGO_BIN_EXE_dx"), &["--format", "json", "providers"]);

    assert_provider_listing_schema(&rows);
    assert_credentialed_providers_are_provider_backed(&rows);
}

#[test]
fn unified_provider_json_keeps_source_kind_separate_from_credentials() {
    let rows = run_json(
        env!("CARGO_BIN_EXE_media"),
        &["--format", "json", "providers", "--provider-type", "media"],
    );

    assert_provider_listing_schema(&rows);
    assert_credentialed_providers_are_provider_backed(&rows);
}

#[test]
fn legacy_cli_help_and_config_do_not_claim_all_providers_are_keyless() {
    let help = run_command(env!("CARGO_BIN_EXE_dx"), &["--help"]);
    assert!(
        help.status.success(),
        "legacy CLI help should run: {}",
        String::from_utf8_lossy(&help.stderr)
    );

    let help_stdout = String::from_utf8_lossy(&help.stdout);
    assert!(
        !help_stdout.contains("no API keys required"),
        "legacy CLI help must not claim every provider is keyless: {help_stdout}"
    );
    assert!(
        help_stdout.contains("credential"),
        "legacy CLI help should point users toward credential-aware provider metadata: {help_stdout}"
    );

    let config = run_command(env!("CARGO_BIN_EXE_dx"), &["--format", "json", "config"]);
    assert!(
        config.status.success(),
        "legacy CLI config should run: {}",
        String::from_utf8_lossy(&config.stderr)
    );

    let config_stdout = String::from_utf8_lossy(&config.stdout);
    assert!(
        !config_stdout.contains("All providers are FREE"),
        "config JSON must not overclaim provider credential status: {config_stdout}"
    );
    assert!(
        config_stdout.contains("credential_status"),
        "config JSON should direct users to credential_status provider metadata: {config_stdout}"
    );
}

#[test]
fn unified_provider_json_rejects_unknown_provider_type() {
    let output = run_command(
        env!("CARGO_BIN_EXE_media"),
        &["--format", "json", "providers", "--provider-type", "banana"],
    );

    assert!(
        !output.status.success(),
        "unknown provider types should fail closed"
    );
    assert!(
        output.stdout.is_empty(),
        "json output must not include a partial listing for invalid provider filters"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown provider type"),
        "stderr should name the invalid provider filter: {stderr}"
    );
    for provider_type in ["all", "media", "icon", "font"] {
        assert!(
            stderr.contains(provider_type),
            "stderr should list valid provider type '{provider_type}': {stderr}"
        );
    }
}
