use std::process::{Command, Output};

use serde_json::Value;

fn run_media(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_media"))
        .args(args)
        .output()
        .expect("media CLI command should run")
}

fn run_media_json(args: &[&str]) -> Vec<Value> {
    let output = run_media(args);

    assert!(
        output.status.success(),
        "media CLI JSON command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("media CLI should emit a JSON array")
}

fn write_fake_ffmpeg(dir: &std::path::Path) -> std::path::PathBuf {
    #[cfg(windows)]
    {
        let path = dir.join("ffmpeg.cmd");
        std::fs::write(
            &path,
            "@echo off\r\nif \"%1\"==\"-version\" (\r\n  echo fake ffmpeg version\r\n  exit /b 0\r\n)\r\nset \"last=\"\r\n:loop\r\nif \"%~1\"==\"\" goto done\r\nset \"last=%~1\"\r\nshift\r\ngoto loop\r\n:done\r\nif \"%last%\"==\"\" exit /b 2\r\necho fake-audio>\"%last%\"\r\nexit /b 0\r\n",
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
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  echo 'fake ffmpeg version'\n  exit 0\nfi\nlast=\"\"\nfor arg in \"$@\"; do last=\"$arg\"; done\n[ -n \"$last\" ] || exit 2\nprintf 'fake-audio\\n' > \"$last\"\nexit 0\n",
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

#[test]
fn unified_tools_json_exposes_route_local_readiness() {
    let rows = run_media_json(&["--format", "json", "tools", "list"]);
    let audio_convert = rows
        .iter()
        .find(|row| row.get("name").and_then(Value::as_str) == Some("audio.convert"))
        .expect("audio.convert should be listed");
    let command_paths = audio_convert
        .get("command_paths")
        .and_then(Value::as_array)
        .expect("tools JSON should expose command_paths");
    let routes = audio_convert
        .get("routes")
        .and_then(Value::as_array)
        .expect("tools JSON should expose routes");

    assert_eq!(command_paths.len(), routes.len());

    let legacy_route = routes
        .iter()
        .find(|route| {
            route.get("path").and_then(Value::as_str) == Some("media tools audio convert")
        })
        .expect("legacy audio.convert route should be listed");

    assert_eq!(
        legacy_route.get("surface").and_then(Value::as_str),
        Some("legacy-tools-cli")
    );
    assert_eq!(
        legacy_route
            .get("receipt_readiness")
            .and_then(Value::as_str),
        Some("declared-only")
    );
    assert_eq!(
        legacy_route
            .get("type_validation_readiness")
            .and_then(Value::as_str),
        Some("declared-only")
    );

    for row in rows.iter().filter(|row| {
        !row.get("api_only")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }) {
        let name = row
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        let paths = row
            .get("command_paths")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("{name} should expose command_paths"));
        let routes = row
            .get("routes")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("{name} should expose routes"));

        assert_eq!(
            paths.len(),
            routes.len(),
            "{name} should expose one route record per command path"
        );

        for route in routes {
            assert!(
                route.get("surface").and_then(Value::as_str).is_some()
                    && route.get("readiness").and_then(Value::as_str).is_some()
                    && route
                        .get("receipt_readiness")
                        .and_then(Value::as_str)
                        .is_some()
                    && route
                        .get("type_validation_readiness")
                        .and_then(Value::as_str)
                        .is_some(),
                "{name} route should expose local readiness evidence: {route:?}"
            );
        }
    }
}

#[test]
fn unified_tools_table_exposes_route_local_readiness() {
    let output = run_media(&["tools", "list", "--category", "audio"]);

    assert!(
        output.status.success(),
        "media tools table command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("routes"), "{stdout}");
    assert!(stdout.contains("media tools audio convert"), "{stdout}");
    assert!(stdout.contains("legacy-tools-cli"), "{stdout}");
    assert!(stdout.contains("declared-only"), "{stdout}");
}

#[test]
fn audio_convert_json_output_surfaces_receipt_metadata() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let ffmpeg = write_fake_ffmpeg(dir.path());
    let input = dir.path().join("source.wav");
    let output = dir.path().join("converted.mp3");
    std::fs::write(&input, b"fake audio input").expect("audio input fixture should be written");

    let command_output = Command::new(env!("CARGO_BIN_EXE_media"))
        .args([
            "--format",
            "json",
            "audio",
            "convert",
            input.to_str().expect("input path should be UTF-8"),
            output.to_str().expect("output path should be UTF-8"),
        ])
        .env("DX_MEDIA_FFMPEG_BIN", ffmpeg)
        .output()
        .expect("audio convert command should run");

    assert!(
        command_output.status.success(),
        "audio convert failed: {}",
        String::from_utf8_lossy(&command_output.stderr)
    );

    let value: Value =
        serde_json::from_slice(&command_output.stdout).expect("audio convert should emit JSON");
    let metadata = value
        .get("metadata")
        .and_then(Value::as_object)
        .expect("audio convert JSON should include metadata");

    assert!(output.exists());
    assert_eq!(
        metadata.get("tool.name").and_then(Value::as_str),
        Some("audio.convert")
    );
    assert_eq!(
        metadata.get("tool.source_kind").and_then(Value::as_str),
        Some("local-only")
    );
    assert_eq!(
        metadata
            .get("tool.receipt_completeness")
            .and_then(Value::as_str),
        Some("explicit")
    );
    assert_eq!(
        metadata.get("tool.dependency").and_then(Value::as_str),
        Some("ffmpeg")
    );
    assert_eq!(
        metadata.get("tool.type_validation").and_then(Value::as_str),
        Some("pass")
    );
}

#[test]
fn archive_zip_json_output_surfaces_receipt_metadata() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.txt");
    let output = dir.path().join("bundle.zip");
    std::fs::write(&input, b"zip me").expect("archive input fixture should be written");

    let command_output = Command::new(env!("CARGO_BIN_EXE_media"))
        .args([
            "--format",
            "json",
            "archive",
            "zip",
            input.to_str().expect("input path should be UTF-8"),
            "--output",
            output.to_str().expect("output path should be UTF-8"),
        ])
        .output()
        .expect("archive zip command should run");

    assert!(
        command_output.status.success(),
        "archive zip failed: {}",
        String::from_utf8_lossy(&command_output.stderr)
    );

    let value: Value =
        serde_json::from_slice(&command_output.stdout).expect("archive zip should emit JSON");
    let metadata = value
        .get("metadata")
        .and_then(Value::as_object)
        .expect("archive zip JSON should include metadata");

    assert!(output.exists());
    assert_eq!(
        metadata.get("tool.name").and_then(Value::as_str),
        Some("archive.zip.native")
    );
    assert_eq!(
        metadata
            .get("tool.receipt_completeness")
            .and_then(Value::as_str),
        Some("explicit")
    );
    assert_eq!(
        metadata.get("tool.type_validation").and_then(Value::as_str),
        Some("pass")
    );
}

#[test]
fn utility_hash_json_output_surfaces_receipt_metadata() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.txt");
    std::fs::write(&input, b"hash me").expect("hash input fixture should be written");

    let command_output = Command::new(env!("CARGO_BIN_EXE_media"))
        .args([
            "--format",
            "json",
            "utility",
            "hash",
            input.to_str().expect("input path should be UTF-8"),
        ])
        .output()
        .expect("utility hash command should run");

    assert!(
        command_output.status.success(),
        "utility hash failed: {}",
        String::from_utf8_lossy(&command_output.stderr)
    );

    let value: Value =
        serde_json::from_slice(&command_output.stdout).expect("utility hash should emit JSON");
    let metadata = value
        .get("metadata")
        .and_then(Value::as_object)
        .expect("utility hash JSON should include metadata");

    assert_eq!(
        metadata.get("tool.name").and_then(Value::as_str),
        Some("utility.hash")
    );
    assert_eq!(
        metadata
            .get("tool.receipt_completeness")
            .and_then(Value::as_str),
        Some("explicit")
    );
    assert_eq!(
        metadata.get("tool.type_validation").and_then(Value::as_str),
        Some("not-applicable")
    );
}

#[test]
fn json_mode_runtime_failures_emit_machine_readable_error() {
    let output = run_media(&[
        "--format",
        "json",
        "utility",
        "convert-csv",
        "input.csv",
        "output.json",
    ]);

    assert!(!output.status.success(), "declared-only tool should fail");
    assert!(
        output.stdout.is_empty(),
        "JSON failure mode should not print human text to stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let value: Value =
        serde_json::from_slice(&output.stderr).expect("JSON mode failures should emit JSON stderr");

    assert_eq!(value.get("success").and_then(Value::as_bool), Some(false));
    assert!(
        value
            .get("error")
            .and_then(Value::as_str)
            .is_some_and(|error| error.contains("utility.convert-csv")),
        "{value}"
    );
}

#[test]
fn archive_declared_json_failures_do_not_pollute_stdout() {
    let dir = tempfile::tempdir().expect("temp dir should be created");
    let input = dir.path().join("source.txt");
    let output = dir.path().join("bundle.tar");
    std::fs::write(&input, b"tar me later").expect("archive input fixture should be written");

    let command_output = Command::new(env!("CARGO_BIN_EXE_media"))
        .args([
            "--format",
            "json",
            "archive",
            "tar",
            input.to_str().expect("input path should be UTF-8"),
            "--output",
            output.to_str().expect("output path should be UTF-8"),
        ])
        .output()
        .expect("archive tar command should run");

    assert!(
        !command_output.status.success(),
        "declared-only archive tar should fail until wired"
    );
    assert!(
        command_output.stdout.is_empty(),
        "declared archive JSON failure should not print human text to stdout: {}",
        String::from_utf8_lossy(&command_output.stdout)
    );

    let value: Value = serde_json::from_slice(&command_output.stderr)
        .expect("declared archive JSON failure should emit JSON stderr");

    assert_eq!(value.get("success").and_then(Value::as_bool), Some(false));
    assert!(
        value
            .get("error")
            .and_then(Value::as_str)
            .is_some_and(|error| error.contains("archive.tar")),
        "{value}"
    );
}
