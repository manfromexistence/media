use std::ffi::OsString;
use std::process::{Command, Output};

use crate::error::{DxError, Result};

const FFMPEG_PROBE_ENV: &str = "DX_MEDIA_FFPROBE_BIN";

pub(crate) fn binary() -> OsString {
    std::env::var_os(FFMPEG_PROBE_ENV).unwrap_or_else(|| OsString::from("ffprobe"))
}

pub(crate) fn command() -> Command {
    Command::new(binary())
}

pub(crate) fn ensure_success(output: &Output, operation: &str) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let status = output.status.code().map_or_else(
        || "terminated by signal".to_string(),
        |code| code.to_string(),
    );
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("exit status {status}")
    };

    Err(DxError::Config {
        message: format!("ffprobe failed while {operation}: {detail}"),
        source: None,
    })
}
