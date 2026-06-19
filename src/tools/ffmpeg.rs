use std::ffi::OsString;
use std::process::{Command, Output};

use crate::error::{DxError, Result};

const FFMPEG_ENV: &str = "DX_MEDIA_FFMPEG_BIN";

pub(crate) fn binary() -> OsString {
    std::env::var_os(FFMPEG_ENV).unwrap_or_else(|| OsString::from("ffmpeg"))
}

pub(crate) fn command() -> Command {
    Command::new(binary())
}

pub(crate) fn ensure_success(output: &Output, operation: &str) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }

    let detail = failure_detail(output);

    Err(DxError::Config {
        message: format!("ffmpeg failed while {operation}: {detail}"),
        source: None,
    })
}

pub(crate) fn run_failed(error: impl std::fmt::Display, operation: &str) -> DxError {
    DxError::Config {
        message: format!("failed to run ffmpeg while {operation}: {error}"),
        source: None,
    }
}

fn failure_detail(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }

    let status = output.status.code().map_or_else(
        || "terminated by signal".to_string(),
        |code| code.to_string(),
    );
    format!("exit status {status}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffmpeg_failure_detail_prefers_stderr() {
        let output = output_with_status(9, b"stdout details", b"stderr details");

        let err = ensure_success(&output, "testing failure")
            .expect_err("nonzero ffmpeg output should fail");

        assert!(
            err.to_string()
                .contains("ffmpeg failed while testing failure: stderr details"),
            "{err}"
        );
    }

    #[cfg(windows)]
    fn output_with_status(code: u32, stdout: &[u8], stderr: &[u8]) -> std::process::Output {
        use std::os::windows::process::ExitStatusExt;

        std::process::Output {
            status: std::process::ExitStatus::from_raw(code),
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        }
    }

    #[cfg(unix)]
    fn output_with_status(code: i32, stdout: &[u8], stderr: &[u8]) -> std::process::Output {
        use std::os::unix::process::ExitStatusExt;

        std::process::Output {
            status: std::process::ExitStatus::from_raw(code),
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        }
    }
}
