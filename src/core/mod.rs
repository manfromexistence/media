//! Core infrastructure for dx-media processing.
//!
//! This module provides the foundational components for zero-copy,
//! high-performance media processing following Binary Dawn principles.

mod buffer;
mod cache;
mod pipeline;
mod progress;

pub use buffer::{MappedBuffer, MediaBuffer};
pub use cache::{CacheEntry, CacheKey, ConversionCache};
pub use pipeline::{MediaPipeline, PipelineStage};
pub use progress::{ProgressCallback, ProgressTracker};

use std::path::PathBuf;
use thiserror::Error;

/// Core media processing errors.
#[derive(Error, Debug)]
pub enum CoreError {
    /// I/O error from file system operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The requested format is not supported for this operation.
    #[error("Format not supported: {0}")]
    UnsupportedFormat(String),

    /// Media conversion operation failed.
    #[error("Conversion failed: {0}")]
    ConversionFailed(String),

    /// Invalid input provided to the operation.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Error accessing or writing to the cache.
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Error in the processing pipeline.
    #[error("Pipeline error: {0}")]
    PipelineError(String),
}

/// Result type for core operations.
pub type CoreResult<T> = Result<T, CoreError>;

/// Configuration for the media processing core.
#[derive(Debug, Clone)]
pub struct CoreConfig {
    /// Directory for cache storage.
    pub cache_dir: PathBuf,
    /// Enable conversion caching.
    pub cache_enabled: bool,
    /// Maximum cache size in bytes.
    pub max_cache_size: u64,
    /// Number of parallel workers.
    pub parallel_workers: usize,
    /// Enable memory mapping for large files.
    pub use_mmap: bool,
    /// Threshold for memory mapping (files larger than this use mmap).
    pub mmap_threshold: u64,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            cache_dir: directories::BaseDirs::new()
                .map(|d| d.cache_dir().join("dx-media"))
                .unwrap_or_else(|| PathBuf::from(".dx-media-cache")),
            cache_enabled: true,
            max_cache_size: 1024 * 1024 * 1024, // 1 GB
            parallel_workers: rayon::current_num_threads(),
            use_mmap: true,
            mmap_threshold: 10 * 1024 * 1024, // 10 MB
        }
    }
}
