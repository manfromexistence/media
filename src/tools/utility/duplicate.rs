//! Duplicate file finder using content hashing.
//!
//! High-performance parallel duplicate detection using SHA-256.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::tools::utility::checksum::{ChecksumAlgorithm, calculate_checksum};
use crate::tools::{ToolOutput, ToolReceipt};

/// A group of duplicate files.
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    /// SHA-256 hash of the content.
    pub hash: String,
    /// Size of each file in bytes.
    pub size: u64,
    /// Paths to duplicate files.
    pub files: Vec<PathBuf>,
    /// Potential space savings if duplicates are removed.
    pub savings: u64,
}

/// Options for duplicate finding.
#[derive(Debug, Clone)]
pub struct DuplicateOptions {
    /// Minimum file size to consider.
    pub min_size: u64,
    /// Maximum file size to consider (0 = no limit).
    pub max_size: u64,
    /// File extensions to include (empty = all).
    pub extensions: Vec<String>,
    /// Directories to exclude.
    pub exclude_dirs: Vec<String>,
    /// Follow symbolic links.
    pub follow_symlinks: bool,
    /// Include hidden files.
    pub include_hidden: bool,
}

impl Default for DuplicateOptions {
    fn default() -> Self {
        Self {
            min_size: 1,
            max_size: 0,
            extensions: Vec::new(),
            exclude_dirs: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".cache".to_string(),
            ],
            follow_symlinks: false,
            include_hidden: false,
        }
    }
}

/// Find duplicate files in a directory.
pub fn find_duplicates(dir: impl AsRef<Path>, options: &DuplicateOptions) -> Vec<DuplicateGroup> {
    let dir = dir.as_ref();

    // Phase 1: Group files by size
    let size_groups = group_by_size(dir, options);

    // Phase 2: Hash files with same size (in parallel)
    let hash_groups = hash_candidates(size_groups, options);

    // Phase 3: Build duplicate groups
    let mut duplicates: Vec<DuplicateGroup> = hash_groups
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(hash, files)| {
            let size = files
                .first()
                .and_then(|p| p.metadata().ok())
                .map_or(0, |m| m.len());
            let savings = size * (files.len() as u64 - 1);

            DuplicateGroup {
                hash,
                size,
                files,
                savings,
            }
        })
        .collect();

    // Sort by potential savings (largest first)
    duplicates.sort_by(|a, b| b.savings.cmp(&a.savings));

    duplicates
}

/// Find duplicates and return as ToolOutput.
pub fn find_duplicates_tool(
    dir: impl AsRef<Path>,
    options: Option<DuplicateOptions>,
) -> ToolOutput {
    let dir = dir.as_ref();
    let options = options.unwrap_or_default();
    let duplicates = find_duplicates(dir, &options);

    let total_groups = duplicates.len();
    let total_files: usize = duplicates.iter().map(|g| g.files.len()).sum();
    let total_savings: u64 = duplicates.iter().map(|g| g.savings).sum();

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("duplicate_groups".to_string(), total_groups.to_string());
    metadata.insert("total_duplicate_files".to_string(), total_files.to_string());
    metadata.insert(
        "potential_savings_bytes".to_string(),
        total_savings.to_string(),
    );
    metadata.insert(
        "potential_savings_human".to_string(),
        format_size(total_savings),
    );

    // Add duplicate info to metadata
    for (i, group) in duplicates.iter().take(100).enumerate() {
        metadata.insert(format!("group_{}_hash", i), group.hash.clone());
        metadata.insert(
            format!("group_{}_files", i),
            group
                .files
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(";"),
        );
    }

    ToolOutput::success(format!(
        "Found {} duplicate groups ({} files), potential savings: {}",
        total_groups,
        total_files,
        format_size(total_savings)
    ))
    .with_paths(duplicates.iter().flat_map(|g| g.files.clone()).collect())
    .with_receipt(
        ToolReceipt::local("utility.find-duplicates").with_source(dir.display().to_string()),
    )
    .with_metadata("tool.expected_media_type", "directory")
    .with_metadata("tool.type_validation", "not-applicable")
    .with_metadata("tool.type_validation_scope", "directory-scan")
    .with_metadata_entries(metadata)
}

/// Group files by size (first pass filter).
fn group_by_size(dir: &Path, options: &DuplicateOptions) -> HashMap<u64, Vec<PathBuf>> {
    let mut size_groups: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    let walker = WalkDir::new(dir)
        .follow_links(options.follow_symlinks)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();

            // Skip hidden files if not included
            if !options.include_hidden && name.starts_with('.') && e.depth() > 0 {
                return false;
            }

            // Skip excluded directories
            if e.file_type().is_dir() {
                return !options.exclude_dirs.iter().any(|ex| name == *ex);
            }

            true
        });

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // Check extension filter
        if !options.extensions.is_empty() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();

            if !options.extensions.iter().any(|e| e.to_lowercase() == ext) {
                continue;
            }
        }

        // Get file size
        let size = match path.metadata() {
            Ok(m) => m.len(),
            Err(_) => continue,
        };

        // Check size limits
        if size < options.min_size {
            continue;
        }
        if options.max_size > 0 && size > options.max_size {
            continue;
        }

        size_groups
            .entry(size)
            .or_default()
            .push(path.to_path_buf());
    }

    // Only keep groups with potential duplicates
    size_groups.retain(|_, files| files.len() > 1);

    size_groups
}

/// Hash files with same size to find actual duplicates.
fn hash_candidates(
    size_groups: HashMap<u64, Vec<PathBuf>>,
    _options: &DuplicateOptions,
) -> HashMap<String, Vec<PathBuf>> {
    let hash_groups: Arc<Mutex<HashMap<String, Vec<PathBuf>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Flatten to list of files to hash
    let files: Vec<PathBuf> = size_groups.into_values().flatten().collect();

    // Hash files in parallel
    files.par_iter().for_each(|path| {
        if let Ok(hash) = hash_file(path) {
            let mut groups = hash_groups.lock().unwrap();
            groups.entry(hash).or_default().push(path.clone());
        }
    });

    Arc::try_unwrap(hash_groups).unwrap().into_inner().unwrap()
}

/// Calculate a real SHA-256 hash of a file.
fn hash_file(path: &Path) -> std::io::Result<String> {
    calculate_checksum(path, ChecksumAlgorithm::Sha256).map(|checksum| checksum.hash)
}

/// Format file size for human display.
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_duplicates() {
        let dir = tempdir().unwrap();

        // Create duplicate files
        let content = b"duplicate content here";

        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        let file3 = dir.path().join("unique.txt");

        std::fs::write(&file1, content).unwrap();
        std::fs::write(&file2, content).unwrap();
        std::fs::write(&file3, b"unique content").unwrap();

        let duplicates = find_duplicates(dir.path(), &DuplicateOptions::default());

        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].files.len(), 2);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
