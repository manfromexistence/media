//! File manager for organizing downloaded media assets.

use std::path::{Path, PathBuf};

use crate::error::{DxError, Result};
use crate::types::MediaAsset;

/// File manager for organizing and naming downloaded files.
#[derive(Debug, Clone)]
pub struct FileManager {
    base_dir: PathBuf,
    organize_by_provider: bool,
    organize_by_type: bool,
}

impl FileManager {
    /// Create a new file manager with the given base directory.
    #[must_use]
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            organize_by_provider: false,
            organize_by_type: false,
        }
    }

    /// Enable organization by provider name (e.g., downloads/openverse/).
    #[must_use]
    pub fn organize_by_provider(mut self, enable: bool) -> Self {
        self.organize_by_provider = enable;
        self
    }

    /// Enable organization by media type (e.g., downloads/images/).
    #[must_use]
    pub fn organize_by_type(mut self, enable: bool) -> Self {
        self.organize_by_type = enable;
        self
    }

    /// Get the target directory for an asset based on organization settings.
    #[must_use]
    pub fn target_dir(&self, asset: &MediaAsset) -> PathBuf {
        let mut dir = self.base_dir.clone();

        if self.organize_by_provider {
            dir = dir.join(&asset.provider);
        }

        if self.organize_by_type {
            dir = dir.join(asset.media_type.as_plural_str());
        }

        dir
    }

    /// Get the full target path for an asset.
    #[must_use]
    pub fn target_path(&self, asset: &MediaAsset, filename: &str) -> PathBuf {
        self.target_dir(asset).join(filename)
    }

    /// Ensure the target directory exists.
    pub async fn ensure_dir(&self, asset: &MediaAsset) -> Result<PathBuf> {
        let dir = self.target_dir(asset);

        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| DxError::FileIo {
                path: dir.clone(),
                message: format!("Failed to create directory: {}", e),
                source: Some(e),
            })?;

        Ok(dir)
    }

    /// Check if a file already exists.
    pub async fn file_exists(&self, path: &Path) -> bool {
        tokio::fs::metadata(path).await.is_ok()
    }

    /// Get file size if file exists.
    pub async fn file_size(&self, path: &Path) -> Option<u64> {
        tokio::fs::metadata(path).await.ok().map(|m| m.len())
    }

    /// Delete a file.
    pub async fn delete_file(&self, path: &Path) -> Result<()> {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| DxError::FileIo {
                path: path.to_path_buf(),
                message: format!("Failed to delete file: {}", e),
                source: Some(e),
            })
    }

    /// Rename/move a file.
    pub async fn rename_file(&self, from: &Path, to: &Path) -> Result<()> {
        // Ensure target directory exists
        if let Some(parent) = to.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| DxError::FileIo {
                    path: parent.to_path_buf(),
                    message: format!("Failed to create directory: {}", e),
                    source: Some(e),
                })?;
        }

        tokio::fs::rename(from, to)
            .await
            .map_err(|e| DxError::FileIo {
                path: from.to_path_buf(),
                message: format!("Failed to rename file: {}", e),
                source: Some(e),
            })
    }

    /// Get the base directory.
    #[must_use]
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// List files in a directory.
    pub async fn list_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(dir)
            .await
            .map_err(|e| DxError::FileIo {
                path: dir.to_path_buf(),
                message: format!("Failed to read directory: {}", e),
                source: Some(e),
            })?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            }
        }

        Ok(files)
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new("./downloads")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MediaType;

    fn test_asset() -> MediaAsset {
        MediaAsset::builder()
            .id("123")
            .provider("unsplash")
            .media_type(MediaType::Image)
            .title("Test")
            .download_url("https://example.com/test.jpg")
            .source_url("https://unsplash.com/photos/123")
            .build()
            .expect("test asset should build")
    }

    #[test]
    fn test_target_dir_default() {
        let fm = FileManager::new("/downloads");
        let asset = test_asset();

        assert_eq!(fm.target_dir(&asset), PathBuf::from("/downloads"));
    }

    #[test]
    fn test_target_dir_by_provider() {
        let fm = FileManager::new("/downloads").organize_by_provider(true);
        let asset = test_asset();

        assert_eq!(fm.target_dir(&asset), PathBuf::from("/downloads/unsplash"));
    }

    #[test]
    fn test_target_dir_by_type() {
        let fm = FileManager::new("/downloads").organize_by_type(true);
        let asset = test_asset();

        assert_eq!(fm.target_dir(&asset), PathBuf::from("/downloads/images"));
    }

    #[test]
    fn test_target_dir_both() {
        let fm = FileManager::new("/downloads")
            .organize_by_provider(true)
            .organize_by_type(true);
        let asset = test_asset();

        assert_eq!(
            fm.target_dir(&asset),
            PathBuf::from("/downloads/unsplash/images")
        );
    }
}
