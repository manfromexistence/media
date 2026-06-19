//! Binary artifact cache for dx-media.
//!
//! High-performance caching layer that stores processed media
//! artifacts using content-addressable storage.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Configuration for the binary cache.
#[derive(Debug, Clone)]
pub struct BinaryCacheConfig {
    /// Root directory for cache storage.
    pub cache_dir: PathBuf,
    /// Maximum cache size in bytes.
    pub max_size: u64,
    /// Time-to-live for cache entries.
    pub ttl: Duration,
    /// Enable content-based deduplication.
    pub dedup_enabled: bool,
    /// Subdirectory depth for cache organization.
    pub dir_depth: usize,
}

impl Default for BinaryCacheConfig {
    fn default() -> Self {
        Self {
            cache_dir: directories::BaseDirs::new()
                .map(|d| d.cache_dir().join("dx-media").join("binary"))
                .unwrap_or_else(|| PathBuf::from(".dx-cache/binary")),
            max_size: 2 * 1024 * 1024 * 1024,           // 2 GB
            ttl: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            dedup_enabled: true,
            dir_depth: 2,
        }
    }
}

/// Cache entry metadata.
/// Fields are used for cache management and may be accessed in future features.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheMetadata {
    /// Content hash (key).
    pub hash: String,
    /// Original filename.
    pub original_name: String,
    /// Size in bytes.
    pub size: u64,
    /// Creation timestamp.
    pub created_at: u64,
    /// Last access timestamp.
    pub accessed_at: u64,
    /// Processing parameters used.
    pub params: HashMap<String, String>,
}

/// High-performance binary cache.
pub struct BinaryCache {
    /// Configuration.
    config: BinaryCacheConfig,
    /// In-memory index.
    index: Arc<RwLock<HashMap<String, CacheMetadata>>>,
    /// Current total size.
    current_size: Arc<RwLock<u64>>,
}

impl BinaryCache {
    /// Create a new binary cache.
    pub fn new(config: BinaryCacheConfig) -> std::io::Result<Self> {
        fs::create_dir_all(&config.cache_dir)?;

        let cache = Self {
            config,
            index: Arc::new(RwLock::new(HashMap::new())),
            current_size: Arc::new(RwLock::new(0)),
        };

        cache.rebuild_index()?;
        Ok(cache)
    }

    /// Create with default configuration.
    pub fn default_cache() -> std::io::Result<Self> {
        Self::new(BinaryCacheConfig::default())
    }

    /// Generate a cache key from content and parameters.
    pub fn generate_key(content: &[u8], params: &HashMap<String, String>) -> String {
        // Simple hash combining content and params
        let mut hasher = SimpleHasher::new();
        hasher.update(content);

        // Sort params for consistent hashing
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);
        for (k, v) in sorted_params {
            hasher.update(k.as_bytes());
            hasher.update(v.as_bytes());
        }

        hasher.finalize_hex()
    }

    /// Get the cache path for a key.
    fn cache_path(&self, key: &str) -> PathBuf {
        let mut path = self.config.cache_dir.clone();

        // Create subdirectory structure based on hash prefix
        for i in 0..self.config.dir_depth.min(key.len() / 2) {
            path.push(&key[i * 2..i * 2 + 2]);
        }

        path.push(format!("{}.bin", key));
        path
    }

    /// Check if a key exists in the cache.
    pub fn contains(&self, key: &str) -> bool {
        if let Ok(index) = self.index.read() {
            if let Some(meta) = index.get(key) {
                // Check TTL
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if now - meta.created_at < self.config.ttl.as_secs() {
                    return self.cache_path(key).exists();
                }
            }
        }
        false
    }

    /// Get data from cache.
    pub fn get(&self, key: &str) -> std::io::Result<Option<Vec<u8>>> {
        if !self.contains(key) {
            return Ok(None);
        }

        let path = self.cache_path(key);
        if !path.exists() {
            return Ok(None);
        }

        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        // Update access time
        if let Ok(mut index) = self.index.write() {
            if let Some(meta) = index.get_mut(key) {
                meta.accessed_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
            }
        }

        Ok(Some(data))
    }

    /// Store data in cache.
    pub fn put(
        &self,
        key: &str,
        data: &[u8],
        original_name: &str,
        params: HashMap<String, String>,
    ) -> std::io::Result<()> {
        // Ensure we have space
        self.ensure_space(data.len() as u64)?;

        let path = self.cache_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write data
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(data)?;
        writer.flush()?;

        // Update index
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let meta = CacheMetadata {
            hash: key.to_string(),
            original_name: original_name.to_string(),
            size: data.len() as u64,
            created_at: now,
            accessed_at: now,
            params,
        };

        if let Ok(mut index) = self.index.write() {
            index.insert(key.to_string(), meta);
        }

        if let Ok(mut size) = self.current_size.write() {
            *size += data.len() as u64;
        }

        Ok(())
    }

    /// Remove an entry from cache.
    pub fn remove(&self, key: &str) -> std::io::Result<()> {
        let path = self.cache_path(key);

        if let Ok(mut index) = self.index.write() {
            if let Some(meta) = index.remove(key) {
                if let Ok(mut size) = self.current_size.write() {
                    *size = size.saturating_sub(meta.size);
                }
            }
        }

        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    /// Clear the entire cache.
    pub fn clear(&self) -> std::io::Result<()> {
        if let Ok(mut index) = self.index.write() {
            index.clear();
        }

        if let Ok(mut size) = self.current_size.write() {
            *size = 0;
        }

        // Remove all files
        if self.config.cache_dir.exists() {
            fs::remove_dir_all(&self.config.cache_dir)?;
            fs::create_dir_all(&self.config.cache_dir)?;
        }

        Ok(())
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let entry_count = self.index.read().map_or(0, |i| i.len());
        let total_size = self.current_size.read().map_or(0, |s| *s);

        CacheStats {
            entry_count,
            total_size,
            max_size: self.config.max_size,
            usage_percent: (total_size as f64 / self.config.max_size as f64 * 100.0) as u8,
        }
    }

    /// Ensure there's enough space for new data.
    fn ensure_space(&self, needed: u64) -> std::io::Result<()> {
        let current = self.current_size.read().map_or(0, |s| *s);

        if current + needed <= self.config.max_size {
            return Ok(());
        }

        // Need to evict entries (LRU based on access time)
        let mut entries: Vec<_> = self
            .index
            .read()
            .map(|i| i.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        entries.sort_by_key(|(_, e)| e.accessed_at);

        let mut freed = 0u64;
        let to_free = (current + needed).saturating_sub(self.config.max_size);

        for (key, entry) in entries {
            if freed >= to_free {
                break;
            }
            freed += entry.size;
            self.remove(&key)?;
        }

        Ok(())
    }

    /// Rebuild index from disk.
    fn rebuild_index(&self) -> std::io::Result<()> {
        let mut total_size = 0u64;

        if self.config.cache_dir.exists() {
            for entry in walkdir::WalkDir::new(&self.config.cache_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry
                    .path()
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("bin"))
                {
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                }
            }
        }

        if let Ok(mut size) = self.current_size.write() {
            *size = total_size;
        }

        Ok(())
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached entries.
    pub entry_count: usize,
    /// Total size in bytes.
    pub total_size: u64,
    /// Maximum allowed size.
    pub max_size: u64,
    /// Usage percentage.
    pub usage_percent: u8,
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
            self.state[(self.pos + 7) % 32] = self.state[(self.pos + 7) % 32]
                .wrapping_mul(31)
                .wrapping_add(*byte);
            self.pos = self.pos.wrapping_add(1);
        }
    }

    fn finalize_hex(self) -> String {
        self.state.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_binary_cache() {
        let dir = tempdir().unwrap();
        let config = BinaryCacheConfig {
            cache_dir: dir.path().to_path_buf(),
            ..Default::default()
        };

        let cache = BinaryCache::new(config).unwrap();

        let key = "test_key_123";
        let data = b"test data content";
        let params = HashMap::new();

        // Put and get
        cache.put(key, data, "test.txt", params.clone()).unwrap();
        let retrieved = cache.get(key).unwrap().unwrap();
        assert_eq!(retrieved, data);

        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.entry_count, 1);
        assert_eq!(stats.total_size, data.len() as u64);

        // Remove
        cache.remove(key).unwrap();
        assert!(cache.get(key).unwrap().is_none());
    }

    #[test]
    fn test_generate_key() {
        let content = b"test content";
        let mut params = HashMap::new();
        params.insert("width".to_string(), "100".to_string());

        let key1 = BinaryCache::generate_key(content, &params);
        let key2 = BinaryCache::generate_key(content, &params);
        assert_eq!(key1, key2);

        params.insert("height".to_string(), "200".to_string());
        let key3 = BinaryCache::generate_key(content, &params);
        assert_ne!(key1, key3);
    }
}
