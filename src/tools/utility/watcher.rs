//! File system watcher for hot folder processing.
//!
//! Provides real-time file change detection for automated
//! media processing workflows.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::tools::ToolOutput;

/// File system event types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEvent {
    /// A new file was created.
    Created(PathBuf),
    /// An existing file was modified.
    Modified(PathBuf),
    /// A file was deleted.
    Deleted(PathBuf),
    /// A file was renamed.
    Renamed {
        /// Original file path before rename.
        from: PathBuf,
        /// New file path after rename.
        to: PathBuf,
    },
}

impl FileEvent {
    /// Get the path associated with this event.
    pub fn path(&self) -> &Path {
        match self {
            Self::Created(p) | Self::Modified(p) | Self::Deleted(p) => p,
            Self::Renamed { to, .. } => to,
        }
    }

    /// Check if this is a creation event.
    pub fn is_created(&self) -> bool {
        matches!(self, Self::Created(_))
    }

    /// Check if this is a modification event.
    pub fn is_modified(&self) -> bool {
        matches!(self, Self::Modified(_))
    }

    /// Check if this is a deletion event.
    pub fn is_deleted(&self) -> bool {
        matches!(self, Self::Deleted(_))
    }

    /// Check if this is a rename event.
    pub fn is_renamed(&self) -> bool {
        matches!(self, Self::Renamed { .. })
    }
}

/// Configuration for the file watcher.
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Watch subdirectories recursively.
    pub recursive: bool,
    /// Debounce duration (ignore rapid successive events).
    pub debounce: Duration,
    /// File extensions to watch (empty = all).
    pub extensions: Vec<String>,
    /// Ignore patterns (glob).
    pub ignore_patterns: Vec<String>,
    /// Ignore hidden files.
    pub ignore_hidden: bool,
    /// Polling interval for systems without native events.
    pub poll_interval: Duration,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            recursive: true,
            debounce: Duration::from_millis(100),
            extensions: Vec::new(),
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.swp".to_string(),
                "*~".to_string(),
                ".git/*".to_string(),
            ],
            ignore_hidden: true,
            poll_interval: Duration::from_secs(1),
        }
    }
}

/// Simple file watcher using polling.
///
/// Note: For production use with native events, use the `notify` crate.
pub struct FileWatcher {
    /// Configuration.
    config: WatcherConfig,
    /// Watched directories.
    watched_paths: Arc<Mutex<HashSet<PathBuf>>>,
    /// Event sender.
    event_tx: Sender<FileEvent>,
    /// Event receiver.
    event_rx: Receiver<FileEvent>,
    /// Known file states (path -> modified time).
    file_states: Arc<Mutex<std::collections::HashMap<PathBuf, u64>>>,
    /// Running flag.
    running: Arc<Mutex<bool>>,
}

impl FileWatcher {
    /// Create a new file watcher.
    pub fn new(config: WatcherConfig) -> Self {
        let (tx, rx) = channel();

        Self {
            config,
            watched_paths: Arc::new(Mutex::new(HashSet::new())),
            event_tx: tx,
            event_rx: rx,
            file_states: Arc::new(Mutex::new(std::collections::HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Create with default configuration.
    pub fn default_watcher() -> Self {
        Self::new(WatcherConfig::default())
    }

    /// Add a path to watch.
    pub fn watch(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref().canonicalize()?;

        if let Ok(mut paths) = self.watched_paths.lock() {
            paths.insert(path.clone());
        }

        // Scan initial state
        self.scan_directory(&path)?;

        Ok(())
    }

    /// Remove a path from watching.
    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref().canonicalize()?;

        if let Ok(mut paths) = self.watched_paths.lock() {
            paths.remove(&path);
        }

        Ok(())
    }

    /// Get the next file event (blocking).
    pub fn next_event(&self) -> Option<FileEvent> {
        self.event_rx.recv().ok()
    }

    /// Try to get the next event (non-blocking).
    pub fn try_next_event(&self) -> Option<FileEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Poll for changes once.
    pub fn poll(&self) -> std::io::Result<Vec<FileEvent>> {
        let paths: Vec<PathBuf> = self
            .watched_paths
            .lock()
            .map(|p| p.iter().cloned().collect())
            .unwrap_or_default();

        let mut events = Vec::new();

        for path in paths {
            let dir_events = self.poll_directory(&path)?;
            events.extend(dir_events);
        }

        Ok(events)
    }

    /// Start polling in a background thread.
    pub fn start_polling(&self) -> std::thread::JoinHandle<()> {
        let config = self.config.clone();
        let watched = Arc::clone(&self.watched_paths);
        let states = Arc::clone(&self.file_states);
        let tx = self.event_tx.clone();
        let running = Arc::clone(&self.running);

        *running.lock().unwrap() = true;

        std::thread::spawn(move || {
            while *running.lock().unwrap() {
                let paths: Vec<PathBuf> = watched
                    .lock()
                    .map(|p| p.iter().cloned().collect())
                    .unwrap_or_default();

                for path in paths {
                    if let Ok(events) = poll_directory_impl(&path, &states, &config) {
                        for event in events {
                            if tx.send(event).is_err() {
                                return;
                            }
                        }
                    }
                }

                std::thread::sleep(config.poll_interval);
            }
        })
    }

    /// Stop the polling thread.
    pub fn stop(&self) {
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        }
    }

    /// Scan a directory and record initial state.
    fn scan_directory(&self, path: &Path) -> std::io::Result<()> {
        let entries = if self.config.recursive {
            walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .map(|e| e.path().to_path_buf())
                .collect::<Vec<_>>()
        } else {
            std::fs::read_dir(path)?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .collect()
        };

        let mut states = self.file_states.lock().unwrap();

        for entry in entries {
            if !entry.is_file() {
                continue;
            }

            if !self.should_watch(&entry) {
                continue;
            }

            if let Ok(meta) = entry.metadata() {
                let mtime = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                states.insert(entry, mtime);
            }
        }

        Ok(())
    }

    /// Poll a directory for changes.
    fn poll_directory(&self, path: &Path) -> std::io::Result<Vec<FileEvent>> {
        poll_directory_impl(path, &self.file_states, &self.config)
    }

    /// Check if a file should be watched based on config.
    fn should_watch(&self, path: &Path) -> bool {
        should_watch_impl(path, &self.config)
    }
}

/// Check if a file should be watched.
fn should_watch_impl(path: &Path, config: &WatcherConfig) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Check hidden files
    if config.ignore_hidden && name.starts_with('.') {
        return false;
    }

    // Check extensions
    if !config.extensions.is_empty() {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !config.extensions.iter().any(|e| e.to_lowercase() == ext) {
            return false;
        }
    }

    // Check ignore patterns (simple glob matching)
    for pattern in &config.ignore_patterns {
        if matches_glob(name, pattern) {
            return false;
        }
    }

    true
}

/// Poll a directory for changes.
fn poll_directory_impl(
    path: &Path,
    states: &Arc<Mutex<std::collections::HashMap<PathBuf, u64>>>,
    config: &WatcherConfig,
) -> std::io::Result<Vec<FileEvent>> {
    let mut events = Vec::new();
    let mut current_files: HashSet<PathBuf> = HashSet::new();

    let entries = if config.recursive {
        walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>()
    } else {
        std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect()
    };

    let mut states_guard = states.lock().unwrap();

    for entry in entries {
        if !entry.is_file() {
            continue;
        }

        if !should_watch_impl(&entry, config) {
            continue;
        }

        current_files.insert(entry.clone());

        let mtime = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        match states_guard.get(&entry) {
            Some(&old_mtime) if old_mtime != mtime => {
                events.push(FileEvent::Modified(entry.clone()));
                states_guard.insert(entry, mtime);
            }
            None => {
                events.push(FileEvent::Created(entry.clone()));
                states_guard.insert(entry, mtime);
            }
            _ => {}
        }
    }

    // Check for deleted files
    let deleted: Vec<PathBuf> = states_guard
        .keys()
        .filter(|p| p.starts_with(path) && !current_files.contains(*p))
        .cloned()
        .collect();

    for path in deleted {
        states_guard.remove(&path);
        events.push(FileEvent::Deleted(path));
    }

    Ok(events)
}

/// Simple glob pattern matching.
fn matches_glob(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if let Some(suffix) = pattern.strip_prefix("*.") {
        return name.ends_with(&format!(".{}", suffix));
    }

    if let Some(prefix) = pattern.strip_suffix("*") {
        return name.starts_with(prefix);
    }

    if pattern.contains("/*") {
        // Skip path patterns for now
        return false;
    }

    name == pattern
}

/// Watch a directory and process files as they appear.
pub fn watch_and_process<F>(
    dir: impl AsRef<Path>,
    config: WatcherConfig,
    mut processor: F,
) -> std::io::Result<()>
where
    F: FnMut(&FileEvent) -> ToolOutput,
{
    let mut watcher = FileWatcher::new(config);
    watcher.watch(dir)?;

    let handle = watcher.start_polling();

    while let Some(event) = watcher.next_event() {
        let _output = processor(&event);
    }

    handle.join().ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_event() {
        let event = FileEvent::Created(PathBuf::from("/test/file.txt"));
        assert!(event.is_created());
        assert!(!event.is_modified());
        assert_eq!(event.path(), Path::new("/test/file.txt"));
    }

    #[test]
    fn test_watcher_poll() {
        let dir = tempdir().unwrap();

        let mut watcher = FileWatcher::default_watcher();
        watcher.watch(dir.path()).unwrap();

        // Create a new file
        let file = dir.path().join("test.txt");
        std::fs::write(&file, b"content").unwrap();

        // Poll should detect the new file
        let events = watcher.poll().unwrap();
        assert!(events.iter().any(|e| e.is_created()));
    }

    #[test]
    fn test_matches_glob() {
        assert!(matches_glob("test.txt", "*.txt"));
        assert!(!matches_glob("test.jpg", "*.txt"));
        assert!(matches_glob("anything", "*"));
        assert!(matches_glob("file.tmp", "*.tmp"));
    }
}
