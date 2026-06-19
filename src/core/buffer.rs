//! Memory buffer utilities for zero-copy media processing.
//!
//! Provides efficient memory-mapped and owned buffers for handling
//! large media files without excessive memory copies.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
#[cfg(not(feature = "mmap"))]
use std::sync::Arc;

use super::CoreResult;

/// A media buffer that can be either owned or memory-mapped.
#[derive(Debug)]
pub enum MediaBuffer {
    /// Owned byte vector (for small files or in-memory processing).
    Owned(Vec<u8>),
    /// Memory-mapped file (for large files, zero-copy).
    Mapped(MappedBuffer),
}

impl MediaBuffer {
    /// Create a new buffer by reading a file.
    /// Uses memory mapping for files larger than the threshold.
    pub fn from_file(path: impl AsRef<Path>, mmap_threshold: u64) -> CoreResult<Self> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();

        if file_size > mmap_threshold {
            Ok(Self::Mapped(MappedBuffer::new(path)?))
        } else {
            let mut file = File::open(path)?;
            let mut buffer = Vec::with_capacity(file_size as usize);
            file.read_to_end(&mut buffer)?;
            Ok(Self::Owned(buffer))
        }
    }

    /// Create an owned buffer from bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::Owned(bytes)
    }

    /// Get the buffer contents as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Owned(vec) => vec,
            Self::Mapped(mapped) => mapped.as_bytes(),
        }
    }

    /// Get the length of the buffer.
    pub fn len(&self) -> usize {
        match self {
            Self::Owned(vec) => vec.len(),
            Self::Mapped(mapped) => mapped.len(),
        }
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Write the buffer contents to a file.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> CoreResult<()> {
        let mut file = File::create(path)?;
        file.write_all(self.as_bytes())?;
        Ok(())
    }
}

/// Memory-mapped file buffer for zero-copy access.
///
/// When the `mmap` feature is enabled, this uses actual memory mapping via `memmap2`.
/// Otherwise, it falls back to reading the file into memory.
#[derive(Debug)]
pub struct MappedBuffer {
    /// The buffer data - either memory-mapped or owned.
    inner: MappedBufferInner,
    /// Original file path (for reference).
    #[allow(dead_code)]
    path: std::path::PathBuf,
}

#[derive(Debug)]
enum MappedBufferInner {
    /// Memory-mapped region (when mmap feature is enabled).
    #[cfg(feature = "mmap")]
    Mmap(memmap2::Mmap),
    /// Fallback: owned data in memory.
    #[cfg(not(feature = "mmap"))]
    Owned(Arc<Vec<u8>>),
}

impl MappedBuffer {
    /// Create a new memory-mapped buffer from a file.
    ///
    /// When the `mmap` feature is enabled, this uses actual memory mapping.
    /// Otherwise, it reads the file into memory.
    pub fn new(path: impl AsRef<Path>) -> CoreResult<Self> {
        let path = path.as_ref();

        #[cfg(feature = "mmap")]
        {
            let file = File::open(path)?;
            // SAFETY: We're mapping a file that we've just opened for reading.
            // The file should not be modified while mapped.
            let mmap = unsafe { memmap2::Mmap::map(&file)? };
            Ok(Self {
                inner: MappedBufferInner::Mmap(mmap),
                path: path.to_path_buf(),
            })
        }

        #[cfg(not(feature = "mmap"))]
        {
            let mut file = File::open(path)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            Ok(Self {
                inner: MappedBufferInner::Owned(Arc::new(data)),
                path: path.to_path_buf(),
            })
        }
    }

    /// Get the mapped data as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        match &self.inner {
            #[cfg(feature = "mmap")]
            MappedBufferInner::Mmap(mmap) => mmap,
            #[cfg(not(feature = "mmap"))]
            MappedBufferInner::Owned(data) => data,
        }
    }

    /// Get the length of the mapped data.
    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if this buffer is using actual memory mapping.
    #[must_use]
    pub fn is_memory_mapped(&self) -> bool {
        #[cfg(feature = "mmap")]
        {
            matches!(self.inner, MappedBufferInner::Mmap(_))
        }
        #[cfg(not(feature = "mmap"))]
        {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_buffer_from_bytes() {
        let data = vec![1, 2, 3, 4, 5];
        let buffer = MediaBuffer::from_bytes(data.clone());
        assert_eq!(buffer.as_bytes(), &data);
        assert_eq!(buffer.len(), 5);
    }

    #[test]
    fn test_buffer_write_to_file() {
        let data = vec![1, 2, 3, 4, 5];
        let buffer = MediaBuffer::from_bytes(data.clone());

        let temp = NamedTempFile::new().unwrap();
        buffer.write_to_file(temp.path()).unwrap();

        let read_back = std::fs::read(temp.path()).unwrap();
        assert_eq!(read_back, data);
    }

    #[test]
    fn test_mapped_buffer_from_file() {
        let temp = NamedTempFile::new().unwrap();
        let data = b"Hello, memory mapping!";
        std::fs::write(temp.path(), data).unwrap();

        let buffer = MappedBuffer::new(temp.path()).unwrap();
        assert_eq!(buffer.as_bytes(), data);
        assert_eq!(buffer.len(), data.len());
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_media_buffer_from_file_small() {
        let temp = NamedTempFile::new().unwrap();
        let data = b"Small file content";
        std::fs::write(temp.path(), data).unwrap();

        // Use a high threshold so it reads into memory
        let buffer = MediaBuffer::from_file(temp.path(), 1024 * 1024).unwrap();
        assert!(matches!(buffer, MediaBuffer::Owned(_)));
        assert_eq!(buffer.as_bytes(), data);
    }

    #[test]
    fn test_media_buffer_from_file_large() {
        let temp = NamedTempFile::new().unwrap();
        let data = b"Large file content that exceeds threshold";
        std::fs::write(temp.path(), data).unwrap();

        // Use a low threshold so it uses memory mapping
        let buffer = MediaBuffer::from_file(temp.path(), 10).unwrap();
        assert!(matches!(buffer, MediaBuffer::Mapped(_)));
        assert_eq!(buffer.as_bytes(), data);
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn test_mapped_buffer_is_memory_mapped() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), b"test data").unwrap();

        let buffer = MappedBuffer::new(temp.path()).unwrap();
        assert!(buffer.is_memory_mapped());
    }

    #[cfg(not(feature = "mmap"))]
    #[test]
    fn test_mapped_buffer_fallback() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), b"test data").unwrap();

        let buffer = MappedBuffer::new(temp.path()).unwrap();
        assert!(!buffer.is_memory_mapped());
    }
}
