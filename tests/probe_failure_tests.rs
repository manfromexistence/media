//! Regression tests for ffprobe boundary failures.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

struct EnvGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let previous = std::env::var_os(key);
        // Environment mutation is guarded by a process-wide mutex for these tests.
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // Environment mutation is guarded by a process-wide mutex for these tests.
        unsafe {
            if let Some(previous) = self.previous.take() {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn write_failing_ffprobe(dir: &Path, message: &str) -> PathBuf {
    #[cfg(windows)]
    {
        let path = dir.join("ffprobe.cmd");
        std::fs::write(
            &path,
            format!("@echo off\r\necho {message} 1>&2\r\nexit /b 7\r\n"),
        )
        .expect("fake ffprobe should be written");
        path
    }

    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = dir.join("ffprobe");
        std::fs::write(&path, format!("#!/bin/sh\necho '{message}' >&2\nexit 7\n"))
            .expect("fake ffprobe should be written");
        let mut permissions = std::fs::metadata(&path)
            .expect("fake ffprobe metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).expect("fake ffprobe should be executable");
        path
    }
}

fn with_fake_ffprobe<T>(message: &str, run: impl FnOnce(&Path) -> T) -> T {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let ffprobe = write_failing_ffprobe(temp_dir.path(), message);
    let _ffprobe_guard = EnvGuard::set("DX_MEDIA_FFPROBE_BIN", ffprobe.as_os_str());

    run(temp_dir.path())
}

fn write_failing_ffmpeg(dir: &Path, message: &str) -> PathBuf {
    #[cfg(windows)]
    {
        let path = dir.join("ffmpeg.cmd");
        std::fs::write(
            &path,
            format!(
                "@echo off\r\nif \"%1\"==\"-version\" (\r\n  echo fake ffmpeg version\r\n  exit /b 0\r\n)\r\necho {message} 1>&2\r\nexit /b 9\r\n"
            ),
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
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  echo 'fake ffmpeg version'\n  exit 0\nfi\necho '{message}' >&2\nexit 9\n"
            ),
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

fn write_successful_ffmpeg_without_output(dir: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let path = dir.join("ffmpeg.cmd");
        std::fs::write(
            &path,
            "@echo off\r\necho fake ffmpeg success\r\nexit /b 0\r\n",
        )
        .expect("fake ffmpeg should be written");
        path
    }

    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = dir.join("ffmpeg");
        std::fs::write(&path, "#!/bin/sh\necho 'fake ffmpeg success'\nexit 0\n")
            .expect("fake ffmpeg should be written");
        let mut permissions = std::fs::metadata(&path)
            .expect("fake ffmpeg metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).expect("fake ffmpeg should be executable");
        path
    }
}

fn with_fake_ffmpeg<T>(message: &str, run: impl FnOnce(&Path) -> T) -> T {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let ffmpeg = write_failing_ffmpeg(temp_dir.path(), message);
    let _ffmpeg_guard = EnvGuard::set("DX_MEDIA_FFMPEG_BIN", ffmpeg.as_os_str());

    run(temp_dir.path())
}

fn with_successful_ffmpeg_without_output<T>(run: impl FnOnce(&Path) -> T) -> T {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let ffmpeg = write_successful_ffmpeg_without_output(temp_dir.path());
    let _ffmpeg_guard = EnvGuard::set("DX_MEDIA_FFMPEG_BIN", ffmpeg.as_os_str());

    run(temp_dir.path())
}

#[test]
fn audio_metadata_reports_ffprobe_stderr_on_nonzero_exit() {
    with_fake_ffprobe("probe failed hard", |dir| {
        let input = dir.join("fixture.wav");
        std::fs::write(&input, b"not real audio").expect("fixture should be written");

        let error = dx_media::tools::audio::read_metadata(&input)
            .expect_err("nonzero ffprobe exit should be an error");

        assert!(
            error.to_string().contains("ffprobe failed"),
            "error should identify ffprobe failure: {error}"
        );
        assert!(
            error.to_string().contains("probe failed hard"),
            "error should preserve stderr: {error}"
        );
    });
}

#[test]
fn audio_prepare_for_transcription_uses_injected_ffmpeg_and_reports_stderr() {
    with_fake_ffmpeg("prepare failed hard", |dir| {
        let input = dir.join("fixture.wav");
        let output = dir.join("prepared.wav");
        std::fs::write(&input, b"not real audio").expect("fixture should be written");

        let error = dx_media::tools::audio::prepare_for_transcription(&input, &output)
            .expect_err("nonzero injected ffmpeg exit should be an error");

        assert!(
            error.to_string().contains("ffmpeg failed"),
            "error should identify ffmpeg failure: {error}"
        );
        assert!(
            error.to_string().contains("prepare failed hard"),
            "error should preserve stderr: {error}"
        );
    });
}

#[test]
fn audio_convert_uses_injected_ffmpeg_and_reports_stderr() {
    with_fake_ffmpeg("convert failed hard", |dir| {
        let input = dir.join("fixture.wav");
        let output = dir.join("converted.mp3");
        std::fs::write(&input, b"not real audio").expect("fixture should be written");

        let error = dx_media::tools::audio::convert_audio(
            &input,
            &output,
            dx_media::tools::audio::ConvertOptions::mp3(128),
        )
        .expect_err("nonzero injected ffmpeg exit should be an error");

        assert!(
            error
                .to_string()
                .contains("ffmpeg failed while converting audio"),
            "error should identify the standardized ffmpeg operation: {error}"
        );
        assert!(
            error.to_string().contains("convert failed hard"),
            "error should preserve stderr: {error}"
        );
    });
}

#[test]
fn audio_convert_rejects_successful_ffmpeg_without_output_receipt() {
    with_successful_ffmpeg_without_output(|dir| {
        let input = dir.join("fixture.wav");
        let output = dir.join("converted.mp3");
        std::fs::write(&input, b"not real audio").expect("fixture should be written");

        let error = dx_media::tools::audio::convert_audio(
            &input,
            &output,
            dx_media::tools::audio::ConvertOptions::mp3(128),
        )
        .expect_err("successful ffmpeg without an output file must not produce a receipt");
        let message = error.to_string();

        assert!(message.contains("audio.convert"), "{message}");
        assert!(message.contains("missing output"), "{message}");
        assert!(message.contains("no tool receipt"), "{message}");
    });
}

#[test]
fn audio_normalize_rejects_successful_ffmpeg_without_output_receipt() {
    with_successful_ffmpeg_without_output(|dir| {
        let input = dir.join("fixture.wav");
        let output = dir.join("normalized.wav");
        std::fs::write(&input, b"not real audio").expect("fixture should be written");

        let error = dx_media::tools::audio::normalize_audio(
            &input,
            &output,
            dx_media::tools::audio::NormalizeOptions::peak(),
        )
        .expect_err("successful ffmpeg without an output file must not produce a receipt");
        let message = error.to_string();

        assert!(message.contains("audio.normalize"), "{message}");
        assert!(message.contains("missing output"), "{message}");
        assert!(message.contains("no tool receipt"), "{message}");
    });
}

#[test]
fn audio_analyze_levels_uses_injected_ffmpeg_and_reports_stderr() {
    with_fake_ffmpeg("analyze failed hard", |dir| {
        let input = dir.join("fixture.wav");
        std::fs::write(&input, b"not real audio").expect("fixture should be written");

        let error = dx_media::tools::audio::analyze_levels(&input)
            .expect_err("nonzero injected ffmpeg exit should be an error");

        assert!(
            error.to_string().contains("ffmpeg failed"),
            "error should identify ffmpeg failure: {error}"
        );
        assert!(
            error.to_string().contains("analyze failed hard"),
            "error should preserve stderr: {error}"
        );
    });
}

#[test]
fn video_transcode_uses_injected_ffmpeg_and_reports_stderr() {
    with_fake_ffmpeg("transcode failed hard", |dir| {
        let input = dir.join("fixture.mp4");
        let output = dir.join("out.mp4");
        std::fs::write(&input, b"not real video").expect("fixture should be written");

        let error = dx_media::tools::video::transcode_video(
            &input,
            &output,
            dx_media::tools::video::TranscodeOptions::default(),
        )
        .expect_err("nonzero injected ffmpeg exit should be an error");

        assert!(
            error.to_string().contains("ffmpeg failed"),
            "error should identify ffmpeg failure: {error}"
        );
        assert!(
            error.to_string().contains("transcode failed hard"),
            "error should preserve stderr: {error}"
        );
    });
}

#[test]
fn public_ffmpeg_availability_helpers_use_injected_binary() {
    with_fake_ffmpeg("unused runtime failure", |_| {
        assert!(
            dx_media::tools::audio::check_ffmpeg_audio(),
            "audio availability check should use DX_MEDIA_FFMPEG_BIN"
        );
        assert!(
            dx_media::tools::video::check_ffmpeg(),
            "video availability check should use DX_MEDIA_FFMPEG_BIN"
        );

        let version = dx_media::tools::video::ffmpeg_version()
            .expect("injected fake ffmpeg version should be readable");
        assert!(
            version.contains("fake ffmpeg version"),
            "version should come from injected ffmpeg, got {version:?}"
        );
    });
}

#[test]
fn video_subtitle_list_reports_ffprobe_stderr_on_nonzero_exit() {
    with_fake_ffprobe("subtitle probe failed", |dir| {
        let input = dir.join("fixture.mp4");
        std::fs::write(&input, b"not real video").expect("fixture should be written");

        let error = dx_media::tools::video::list_subtitle_streams(&input)
            .expect_err("nonzero ffprobe exit should be an error");

        assert!(
            error.to_string().contains("ffprobe failed"),
            "error should identify ffprobe failure: {error}"
        );
        assert!(
            error.to_string().contains("subtitle probe failed"),
            "error should preserve stderr: {error}"
        );
    });
}
