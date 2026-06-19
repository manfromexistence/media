//! Zero-copy processing pipeline for media transformations.
//!
//! Provides a builder-style API for chaining media operations
//! with automatic caching and progress tracking.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{
    CoreConfig, CoreError, CoreResult,
    buffer::MediaBuffer,
    cache::{CacheKey, ConversionCache},
    progress::ProgressTracker,
};

/// A stage in the processing pipeline.
#[derive(Debug, Clone)]
pub struct PipelineStage {
    /// Name of the stage.
    pub name: String,
    /// Description of what this stage does.
    pub description: String,
    /// Estimated weight (for progress calculation).
    pub weight: f32,
}

impl PipelineStage {
    /// Create a new pipeline stage.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weight: 1.0,
        }
    }

    /// Set the weight of this stage.
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }
}

/// Zero-copy media processing pipeline.
pub struct MediaPipeline {
    /// Input buffer.
    input: Option<MediaBuffer>,
    /// Input path (if loaded from file).
    input_path: Option<PathBuf>,
    /// Output path.
    output_path: Option<PathBuf>,
    /// Pipeline stages.
    stages: Vec<PipelineStage>,
    /// Cache instance.
    cache: Option<Arc<ConversionCache>>,
    /// Progress tracker.
    progress: Option<Arc<ProgressTracker>>,
    /// Configuration.
    config: CoreConfig,
    /// Parameters for cache key generation.
    params: Vec<u8>,
}

impl MediaPipeline {
    /// Create a new pipeline with default configuration.
    pub fn new() -> Self {
        Self {
            input: None,
            input_path: None,
            output_path: None,
            stages: Vec::new(),
            cache: None,
            progress: None,
            config: CoreConfig::default(),
            params: Vec::new(),
        }
    }

    /// Create a new pipeline with custom configuration.
    pub fn with_config(config: CoreConfig) -> Self {
        Self {
            input: None,
            input_path: None,
            output_path: None,
            stages: Vec::new(),
            cache: None,
            progress: None,
            config,
            params: Vec::new(),
        }
    }

    /// Load input from a file path.
    pub fn input(mut self, path: impl AsRef<Path>) -> CoreResult<Self> {
        let path = path.as_ref();
        self.input = Some(MediaBuffer::from_file(path, self.config.mmap_threshold)?);
        self.input_path = Some(path.to_path_buf());
        Ok(self)
    }

    /// Load input from bytes.
    pub fn input_bytes(mut self, data: Vec<u8>) -> Self {
        self.input = Some(MediaBuffer::from_bytes(data));
        self
    }

    /// Set output path.
    pub fn output(mut self, path: impl AsRef<Path>) -> Self {
        self.output_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Enable caching with the given cache instance.
    pub fn with_cache(mut self, cache: Arc<ConversionCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Set progress callback.
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(f32, &str) + Send + Sync + 'static,
    {
        self.progress = Some(Arc::new(ProgressTracker::new(callback)));
        self
    }

    /// Add a pipeline stage.
    pub fn stage(mut self, stage: PipelineStage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Add processing parameters (for cache key generation).
    pub fn param(mut self, key: &str, value: &str) -> Self {
        self.params.extend_from_slice(key.as_bytes());
        self.params.push(b'=');
        self.params.extend_from_slice(value.as_bytes());
        self.params.push(b';');
        self
    }

    /// Get the input bytes (zero-copy).
    pub fn input_bytes_ref(&self) -> CoreResult<&[u8]> {
        self.input
            .as_ref()
            .map(|b| b.as_bytes())
            .ok_or_else(|| CoreError::InvalidInput("No input loaded".into()))
    }

    /// Check if a cached result exists.
    pub fn check_cache(&self) -> Option<PathBuf> {
        let cache = self.cache.as_ref()?;
        let input = self.input.as_ref()?;

        let key = CacheKey::generate(input.as_bytes(), &self.params);
        cache.get(&key).map(|e| e.path)
    }

    /// Store result in cache.
    pub fn store_in_cache(&self, data: &[u8]) -> CoreResult<()> {
        if let Some(cache) = &self.cache {
            if let Some(input) = &self.input {
                let key = CacheKey::generate(input.as_bytes(), &self.params);
                cache.put(key, data, &self.params)?;
            }
        }
        Ok(())
    }

    /// Report progress.
    pub fn report_progress(&self, progress: f32, message: &str) {
        if let Some(tracker) = &self.progress {
            tracker.report(progress, message);
        }
    }

    /// Execute the pipeline with a processing function.
    pub fn execute<F>(self, processor: F) -> CoreResult<Vec<u8>>
    where
        F: FnOnce(&[u8]) -> CoreResult<Vec<u8>>,
    {
        // Check cache first
        if let Some(cached_path) = self.check_cache() {
            self.report_progress(1.0, "Using cached result");
            return std::fs::read(cached_path).map_err(CoreError::from);
        }

        // Get input
        let input = self.input_bytes_ref()?;

        // Report start
        self.report_progress(0.0, "Starting processing");

        // Process
        let result = processor(input)?;

        // Store in cache
        self.store_in_cache(&result)?;

        // Write to output if specified
        if let Some(output_path) = &self.output_path {
            std::fs::write(output_path, &result)?;
        }

        // Report completion
        self.report_progress(1.0, "Processing complete");

        Ok(result)
    }

    /// Get the output path.
    pub fn output_path(&self) -> Option<&Path> {
        self.output_path.as_deref()
    }

    /// Get the input path.
    pub fn input_path(&self) -> Option<&Path> {
        self.input_path.as_deref()
    }
}

impl Default for MediaPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_pipeline_basic() {
        let input_data = b"test input data";

        let result = MediaPipeline::new()
            .input_bytes(input_data.to_vec())
            .param("operation", "test")
            .execute(|data| {
                // Simple processing: uppercase ASCII
                Ok(data.iter().map(|b| b.to_ascii_uppercase()).collect())
            })
            .unwrap();

        assert_eq!(result, b"TEST INPUT DATA");
    }

    #[test]
    fn test_pipeline_with_file() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), b"file content").unwrap();

        let result = MediaPipeline::new()
            .input(temp.path())
            .unwrap()
            .execute(|data| Ok(data.to_vec()))
            .unwrap();

        assert_eq!(result, b"file content");
    }
}
