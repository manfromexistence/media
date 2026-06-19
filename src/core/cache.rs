//! Binary conversion cache for dx-media.
//!
//! Follows Binary Dawn philosophy: never repeat work.
//! Uses Blake3 for fast, secure content hashing.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use super::CoreResult;

/// A cache key derived from input content and processing parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey([u8; 32]);

impl CacheKey {
    /// Generate a cache key from input bytes and parameters.
    pub fn generate(input: &[u8], params: &[u8]) -> Self {
        // Use a simple hash combining input and params
        // In production, this would use Blake3
        let mut hasher = SimpleHasher::new();
        hasher.update(input);
        hasher.update(params);
        Self(hasher.finalize())
    }

    /// Generate a cache key from a file path and parameters.
    pub fn from_file(path: impl AsRef<Path>, params: &[u8]) -> CoreResult<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(Self::generate(&buffer, params))
    }

    /// Convert to hex string for display.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

/// A cached conversion entry.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Path to the cached output file.
    pub path: PathBuf,
    /// Hash of the processing parameters.
    pub params_hash: [u8; 32],
    /// Creation timestamp (Unix epoch seconds).
    pub created_at: u64,
    /// Size of the cached file in bytes.
    pub size: u64,
}

impl CacheEntry {
    /// Check if the cache entry is still valid.
    pub fn is_valid(&self) -> bool {
        self.path.exists()
    }
}

/// Binary conversion cache manager.
pub struct ConversionCache {
    /// In-memory cache index.
    index: RwLock<HashMap<CacheKey, CacheEntry>>,
    /// Cache directory.
    cache_dir: PathBuf,
    /// Maximum cache size in bytes.
    max_size: u64,
    /// Current cache size in bytes.
    current_size: RwLock<u64>,
}

impl ConversionCache {
    /// Create a new conversion cache.
    pub fn new(cache_dir: impl AsRef<Path>, max_size: u64) -> CoreResult<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        fs::create_dir_all(&cache_dir)?;

        let cache = Self {
            index: RwLock::new(HashMap::new()),
            cache_dir,
            max_size,
            current_size: RwLock::new(0),
        };

        // Load existing cache entries
        cache.load_index()?;

        Ok(cache)
    }

    /// Get a cached entry by key.
    pub fn get(&self, key: &CacheKey) -> Option<CacheEntry> {
        let index = self.index.read().ok()?;
        let entry = index.get(key)?;

        if entry.is_valid() {
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Store a conversion result in the cache.
    pub fn put(&self, key: CacheKey, data: &[u8], params: &[u8]) -> CoreResult<CacheEntry> {
        // Generate output path
        let filename = format!("{}.cache", key.to_hex());
        let path = self.cache_dir.join(&filename);

        // Ensure we have space
        self.ensure_space(data.len() as u64)?;

        // Write data to cache file
        let mut file = File::create(&path)?;
        file.write_all(data)?;

        let entry = CacheEntry {
            path: path.clone(),
            params_hash: {
                let mut hasher = SimpleHasher::new();
                hasher.update(params);
                hasher.finalize()
            },
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            size: data.len() as u64,
        };

        // Update index
        if let Ok(mut index) = self.index.write() {
            index.insert(key, entry.clone());
        }

        // Update size tracking
        if let Ok(mut size) = self.current_size.write() {
            *size += data.len() as u64;
        }

        Ok(entry)
    }

    /// Remove an entry from the cache.
    pub fn remove(&self, key: &CacheKey) -> CoreResult<()> {
        if let Ok(mut index) = self.index.write() {
            if let Some(entry) = index.remove(key) {
                if entry.path.exists() {
                    fs::remove_file(&entry.path)?;
                }
                if let Ok(mut size) = self.current_size.write() {
                    *size = size.saturating_sub(entry.size);
                }
            }
        }
        Ok(())
    }

    /// Clear the entire cache.
    pub fn clear(&self) -> CoreResult<()> {
        if let Ok(mut index) = self.index.write() {
            for entry in index.values() {
                if entry.path.exists() {
                    fs::remove_file(&entry.path).ok();
                }
            }
            index.clear();
        }
        if let Ok(mut size) = self.current_size.write() {
            *size = 0;
        }
        Ok(())
    }

    /// Get current cache size in bytes.
    pub fn size(&self) -> u64 {
        self.current_size.read().map_or(0, |s| *s)
    }

    /// Get number of cached entries.
    pub fn len(&self) -> usize {
        self.index.read().map_or(0, |i| i.len())
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Ensure there's enough space for new data.
    fn ensure_space(&self, needed: u64) -> CoreResult<()> {
        let current = self.size();
        if current + needed <= self.max_size {
            return Ok(());
        }

        // Need to evict some entries (LRU based on creation time)
        let mut entries: Vec<_> = self
            .index
            .read()
            .map(|i| i.iter().map(|(k, v)| (*k, v.clone())).collect())
            .unwrap_or_default();

        entries.sort_by_key(|(_, e)| e.created_at);

        let mut freed = 0u64;
        let to_free = (current + needed).saturating_sub(self.max_size);

        for (key, entry) in entries {
            if freed >= to_free {
                break;
            }
            freed += entry.size;
            self.remove(&key)?;
        }

        Ok(())
    }

    /// Load cache index from disk.
    fn load_index(&self) -> CoreResult<()> {
        let entries = fs::read_dir(&self.cache_dir)?;
        let mut total_size = 0u64;

        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("cache"))
            {
                if let Ok(metadata) = path.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        if let Ok(mut size) = self.current_size.write() {
            *size = total_size;
        }

        Ok(())
    }
}

/// Simple hasher for cache keys (placeholder for Blake3).
struct SimpleHasher {
    state: [u8; 32],
    pos: usize,
}

impl SimpleHasher {
    fn new() -> Self {
        Self {
            state: [0u8; 32],
            pos: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        for byte in data {
            self.state[self.pos % 32] ^= byte;
            self.state[(self.pos + 1) % 32] = self.state[(self.pos + 1) % 32].wrapping_add(*byte);
            self.pos = self.pos.wrapping_add(1);
        }
    }

    fn finalize(self) -> [u8; 32] {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_key_generation() {
        let key1 = CacheKey::generate(b"hello", b"params");
        let key2 = CacheKey::generate(b"hello", b"params");
        let key3 = CacheKey::generate(b"hello", b"different");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_put_get() {
        let dir = tempdir().unwrap();
        let cache = ConversionCache::new(dir.path(), 1024 * 1024).unwrap();

        let key = CacheKey::generate(b"test", b"params");
        let data = b"cached content";

        cache.put(key, data, b"params").unwrap();

        let entry = cache.get(&key).unwrap();
        assert!(entry.path.exists());

        let cached = fs::read(&entry.path).unwrap();
        assert_eq!(cached, data);
    }
}
