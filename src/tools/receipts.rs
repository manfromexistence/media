use std::path::Path;

use crate::error::{DxError, Result};
use crate::tools::{ToolOutput, ToolReceipt};
use crate::types::MediaType;

pub(crate) fn local_dependency_output(
    tool_name: &'static str,
    dependency: &'static str,
    source: &Path,
    output: &Path,
    expected: MediaType,
    message: impl Into<String>,
) -> Result<ToolOutput> {
    let output_size = validated_output_size(tool_name, output)?;

    Ok(ToolOutput::success_with_path(message, output)
        .with_receipt(
            ToolReceipt::local(tool_name)
                .with_dependency(dependency)
                .with_source(source.display().to_string()),
        )
        .with_output_type_validation(output, expected)
        .with_metadata("tool.output_file_validation", "pass")
        .with_metadata("tool.output_file_size_bytes", output_size.to_string()))
}

fn validated_output_size(tool_name: &'static str, output: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(output).map_err(|e| DxError::FileIo {
        path: output.to_path_buf(),
        message: format!(
            "{tool_name} missing output: {} (no output file or no tool receipt was produced)",
            output.display()
        ),
        source: Some(e),
    })?;

    if !metadata.is_file() {
        return Err(DxError::Config {
            message: format!(
                "{tool_name} invalid output: {} is not a file (no output file or no tool receipt was produced)",
                output.display()
            ),
            source: None,
        });
    }

    let len = metadata.len();
    if len == 0 {
        return Err(DxError::Config {
            message: format!(
                "{tool_name} empty output: {} (no output file or no tool receipt was produced)",
                output.display()
            ),
            source: None,
        });
    }

    Ok(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_dependency_output_records_receipt_source_and_type_validation() {
        let output_path =
            std::env::temp_dir().join(format!("dx-media-local-output-{}.mp3", std::process::id()));
        std::fs::write(&output_path, b"not empty").expect("output fixture should be writable");

        let output = local_dependency_output(
            "audio.convert",
            "ffmpeg",
            Path::new("input.wav"),
            &output_path,
            MediaType::Audio,
            "converted",
        )
        .expect("non-empty output should produce a receipt");

        assert_eq!(
            output.metadata.get("tool.name").map(String::as_str),
            Some("audio.convert")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.receipt_completeness")
                .map(String::as_str),
            Some("explicit")
        );
        assert_eq!(
            output.metadata.get("tool.source_kind").map(String::as_str),
            Some("local-only")
        );
        assert_eq!(
            output.metadata.get("tool.dependency").map(String::as_str),
            Some("ffmpeg")
        );
        assert_eq!(
            output.metadata.get("tool.source").map(String::as_str),
            Some("input.wav")
        );
        assert_eq!(
            output
                .metadata
                .get("tool.output_file_validation")
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

        let _ = std::fs::remove_file(output_path);
    }

    #[test]
    fn local_dependency_output_rejects_missing_output_file() {
        let missing = std::env::temp_dir().join(format!(
            "dx-media-missing-output-{}.mp3",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&missing);

        let err = local_dependency_output(
            "audio.convert",
            "ffmpeg",
            Path::new("input.wav"),
            &missing,
            MediaType::Audio,
            "converted",
        )
        .expect_err("missing ffmpeg output must not produce a success receipt");
        let message = err.to_string();

        assert!(message.contains("audio.convert"), "{message}");
        assert!(message.contains("missing output"), "{message}");
        assert!(message.contains("no tool receipt"), "{message}");
    }

    #[test]
    fn local_dependency_output_rejects_empty_output_file() {
        let empty =
            std::env::temp_dir().join(format!("dx-media-empty-output-{}.mp3", std::process::id()));
        std::fs::write(&empty, []).expect("empty fixture should be writable");

        let err = local_dependency_output(
            "audio.convert",
            "ffmpeg",
            Path::new("input.wav"),
            &empty,
            MediaType::Audio,
            "converted",
        )
        .expect_err("empty ffmpeg output must not produce a success receipt");
        let message = err.to_string();

        assert!(message.contains("audio.convert"), "{message}");
        assert!(message.contains("empty output"), "{message}");
        assert!(message.contains("no tool receipt"), "{message}");

        let _ = std::fs::remove_file(empty);
    }
}
