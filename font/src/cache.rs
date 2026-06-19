//! Cache manager for dx-font
//!
//! This module provides caching functionality for API responses and font metadata.
//! Cache entries are stored as JSON files with TTL (time-to-live) support.

use crate::error::{FontError, FontResult};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

/// A cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// The cached data
    pub data: T,
    /// Unix timestamp when the entry was cached
    pub cached_at: u64,
    /// TTL in seconds
    pub ttl_secs: u64,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry with the current timestamp
    pub fn new(data: T, ttl: Duration) -> Self {
        let cached_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            data,
            cached_at,
            ttl_secs: ttl.as_secs(),
        }
    }

    /// Check if this cache entry is still valid
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now < self.cached_at + self.ttl_secs
    }

    /// Get the age of this cache entry in seconds
    pub fn age_secs(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.cached_at)
    }

    /// Get the remaining TTL in seconds (0 if expired)
    pub fn remaining_ttl_secs(&self) -> u64 {
        let expires_at = self.cached_at + self.ttl_secs;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        expires_at.saturating_sub(now)
    }
}

/// Cache manager for storing and retrieving cached data
#[derive(Debug, Clone)]
pub struct CacheManager {
    /// Directory where cache files are stored
    cache_dir: PathBuf,
    /// Default TTL for cache entries
    ttl: Duration,
}

impl CacheManager {
    /// Create a new cache manager
    ///
    /// # Arguments
    /// * `cache_dir` - Directory to store cache files
    /// * `ttl` - Default time-to-live for cache entries
    ///
    /// # Errors
    /// Returns an error if the cache directory cannot be created
    pub fn new(cache_dir: PathBuf, ttl: Duration) -> FontResult<Self> {
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).map_err(|e| {
                FontError::cache_with_source(
                    format!("Failed to create cache directory: {}", cache_dir.display()),
                    e,
                )
            })?;
        }

        Ok(Self { cache_dir, ttl })
    }

    /// Create a cache manager with the default cache directory
    ///
    /// Uses `dirs::cache_dir()/dx-font` as the cache directory
    pub fn with_default_dir(ttl: Duration) -> FontResult<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dx-font");
        Self::new(cache_dir, ttl)
    }

    /// Get the path to a cache file for a given key
    fn cache_path(&self, key: &str) -> PathBuf {
        // Sanitize the key to be a valid filename
        let safe_key: String = key
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.cache_dir.join(format!("{}.json", safe_key))
    }

    /// Get cached data if valid, None if stale or missing
    ///
    /// # Arguments
    /// * `key` - Cache key to look up
    ///
    /// # Returns
    /// * `Ok(Some(data))` - Valid cached data
    /// * `Ok(None)` - Cache miss or stale entry
    /// * `Err(_)` - Cache read error (corrupted cache is handled gracefully)
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> FontResult<Option<T>> {
        let path = self.cache_path(key);

        if !path.exists() {
            debug!("Cache miss for key '{}': file not found", key);
            return Ok(None);
        }

        // Read the cache file
        let content = match tokio::fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(e) => {
                warn!("Failed to read cache file for key '{}': {}", key, e);
                // Try to remove corrupted cache file
                let _ = tokio::fs::remove_file(&path).await;
                return Ok(None);
            }
        };

        // Parse the cache entry
        let entry: CacheEntry<T> = match serde_json::from_str(&content) {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Cache corruption detected for key '{}': {}", key, e);
                // Remove corrupted cache file
                let _ = tokio::fs::remove_file(&path).await;
                return Ok(None);
            }
        };

        // Check if entry is still valid
        if entry.is_valid() {
            debug!(
                "Cache hit for key '{}' (age: {}s, remaining TTL: {}s)",
                key,
                entry.age_secs(),
                entry.remaining_ttl_secs()
            );
            Ok(Some(entry.data))
        } else {
            debug!(
                "Cache stale for key '{}' (age: {}s, TTL was: {}s)",
                key,
                entry.age_secs(),
                entry.ttl_secs
            );
            Ok(None)
        }
    }

    /// Store data in cache with current timestamp
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `data` - Data to cache
    pub async fn set<T: Serialize>(&self, key: &str, data: &T) -> FontResult<()> {
        self.set_with_ttl(key, data, self.ttl).await
    }

    /// Store data in cache with a custom TTL
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `data` - Data to cache
    /// * `ttl` - Custom TTL for this entry
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: &str,
        data: &T,
        ttl: Duration,
    ) -> FontResult<()> {
        let path = self.cache_path(key);
        let entry = CacheEntry::new(data, ttl);

        let content = serde_json::to_string_pretty(&entry).map_err(|e| {
            FontError::cache(format!(
                "Failed to serialize cache entry for key '{}': {}",
                key, e
            ))
        })?;

        tokio::fs::write(&path, content).await.map_err(|e| {
            FontError::cache_with_source(format!("Failed to write cache file for key '{}'", key), e)
        })?;

        debug!("Cached data for key '{}' with TTL {}s", key, ttl.as_secs());
        Ok(())
    }

    /// Invalidate a specific cache entry
    ///
    /// # Arguments
    /// * `key` - Cache key to invalidate
    pub async fn invalidate(&self, key: &str) -> FontResult<()> {
        let path = self.cache_path(key);

        if path.exists() {
            tokio::fs::remove_file(&path).await.map_err(|e| {
                FontError::cache_with_source(
                    format!("Failed to invalidate cache for key '{}'", key),
                    e,
                )
            })?;
            debug!("Invalidated cache for key '{}'", key);
        }

        Ok(())
    }

    /// Clear all cached data
    pub async fn clear(&self) -> FontResult<()> {
        let mut entries = tokio::fs::read_dir(&self.cache_dir)
            .await
            .map_err(|e| FontError::cache_with_source("Failed to read cache directory", e))?;

        let mut count = 0;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| FontError::cache_with_source("Failed to read cache directory entry", e))?
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Err(e) = tokio::fs::remove_file(&path).await {
                    warn!("Failed to remove cache file {}: {}", path.display(), e);
                } else {
                    count += 1;
                }
            }
        }

        debug!("Cleared {} cache entries", count);
        Ok(())
    }

    /// Check if cache entry exists and is valid
    ///
    /// This is a synchronous check that only verifies the file exists
    /// and the entry hasn't expired based on file metadata.
    pub fn is_valid(&self, key: &str) -> bool {
        let path = self.cache_path(key);

        if !path.exists() {
            return false;
        }

        // Read and check validity synchronously
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<CacheEntry<serde_json::Value>>(&content) {
                Ok(entry) => entry.is_valid(),
                Err(_) => false,
            },
            Err(_) => false,
        }
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Get the default TTL
    pub fn ttl(&self) -> Duration {
        self.ttl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_cache() -> (CacheManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache =
            CacheManager::new(temp_dir.path().to_path_buf(), Duration::from_secs(3600)).unwrap();
        (cache, temp_dir)
    }

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let (cache, _temp) = create_test_cache().await;

        let data = vec!["font1", "font2", "font3"];
        cache.set("test_key", &data).await.unwrap();

        let retrieved: Option<Vec<String>> = cache.get("test_key").await.unwrap();
        assert_eq!(
            retrieved,
            Some(data.iter().map(|s| s.to_string()).collect())
        );
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let (cache, _temp) = create_test_cache().await;

        let result: Option<Vec<String>> = cache.get("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let (cache, _temp) = create_test_cache().await;

        cache.set("test_key", &"test_data").await.unwrap();
        assert!(cache.is_valid("test_key"));

        cache.invalidate("test_key").await.unwrap();
        assert!(!cache.is_valid("test_key"));
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let (cache, _temp) = create_test_cache().await;

        cache.set("key1", &"data1").await.unwrap();
        cache.set("key2", &"data2").await.unwrap();

        assert!(cache.is_valid("key1"));
        assert!(cache.is_valid("key2"));

        cache.clear().await.unwrap();

        assert!(!cache.is_valid("key1"));
        assert!(!cache.is_valid("key2"));
    }

    #[tokio::test]
    async fn test_cache_ttl_expiry() {
        let temp_dir = TempDir::new().unwrap();
        let cache =
            CacheManager::new(temp_dir.path().to_path_buf(), Duration::from_secs(1)).unwrap();

        cache.set("test_key", &"test_data").await.unwrap();
        assert!(cache.is_valid("test_key"));

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Entry should be stale now
        let result: Option<String> = cache.get("test_key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_corruption_handling() {
        let (cache, temp_dir) = create_test_cache().await;

        // Write corrupted data directly to cache file
        let path = temp_dir.path().join("corrupted_key.json");
        tokio::fs::write(&path, "not valid json").await.unwrap();

        // Should handle corruption gracefully and return None
        let result: Option<String> = cache.get("corrupted_key").await.unwrap();
        assert!(result.is_none());

        // Corrupted file should be removed
        assert!(!path.exists());
    }

    #[test]
    fn test_cache_entry_validity() {
        let entry = CacheEntry::new("test", Duration::from_secs(3600));
        assert!(entry.is_valid());
        assert!(entry.age_secs() < 2); // Should be very recent
        assert!(entry.remaining_ttl_secs() > 3598); // Should have most of TTL remaining
    }

    #[test]
    fn test_cache_path_sanitization() {
        let temp_dir = TempDir::new().unwrap();
        let cache =
            CacheManager::new(temp_dir.path().to_path_buf(), Duration::from_secs(3600)).unwrap();

        // Keys with special characters should be sanitized
        let path = cache.cache_path("provider/fonts?query=test");
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(!filename.contains('/'));
        assert!(!filename.contains('?'));
        assert!(filename.ends_with(".json"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    // Feature: dx-font-production-ready, Property 2: Cache Serialization Round-Trip
    // **Validates: Requirements 3.6**
    //
    // For any valid Vec<Font> data, serializing to the cache and deserializing back
    // SHALL produce an equivalent data structure.

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn cache_roundtrip_strings(
            data in prop::collection::vec("[a-zA-Z0-9]{1,20}", 0..10)
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let cache = CacheManager::new(
                    temp_dir.path().to_path_buf(),
                    Duration::from_secs(3600)
                ).unwrap();

                cache.set("test_key", &data).await.unwrap();
                let retrieved: Option<Vec<String>> = cache.get("test_key").await.unwrap();

                prop_assert_eq!(retrieved, Some(data));
                Ok(())
            })?;
        }

        #[test]
        fn cache_roundtrip_numbers(
            data in prop::collection::vec(any::<i64>(), 0..10)
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let cache = CacheManager::new(
                    temp_dir.path().to_path_buf(),
                    Duration::from_secs(3600)
                ).unwrap();

                cache.set("test_key", &data).await.unwrap();
                let retrieved: Option<Vec<i64>> = cache.get("test_key").await.unwrap();

                prop_assert_eq!(retrieved, Some(data));
                Ok(())
            })?;
        }

        #[test]
        fn cache_roundtrip_nested_structure(
            keys in prop::collection::vec("[a-zA-Z]{1,10}", 1..5),
            values in prop::collection::vec("[a-zA-Z0-9]{1,20}", 1..5)
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let cache = CacheManager::new(
                    temp_dir.path().to_path_buf(),
                    Duration::from_secs(3600)
                ).unwrap();

                // Create a map from keys to values
                let data: std::collections::HashMap<String, String> = keys
                    .into_iter()
                    .zip(values.into_iter().cycle())
                    .collect();

                cache.set("test_key", &data).await.unwrap();
                let retrieved: Option<std::collections::HashMap<String, String>> =
                    cache.get("test_key").await.unwrap();

                prop_assert_eq!(retrieved, Some(data));
                Ok(())
            })?;
        }
    }

    // Feature: dx-font-production-ready, Property 3: Cache TTL Enforcement
    // **Validates: Requirements 3.3, 3.4, 3.5**
    //
    // For any cache entry with TTL t seconds, the entry SHALL be considered valid
    // for requests made before t seconds have elapsed, and invalid for requests
    // made after t seconds have elapsed.

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn cache_entry_valid_before_ttl(ttl_secs in 10u64..3600u64) {
            let entry = CacheEntry::new("test_data", Duration::from_secs(ttl_secs));

            // Entry should be valid immediately after creation
            prop_assert!(entry.is_valid());

            // Age should be very small (< 1 second)
            prop_assert!(entry.age_secs() < 2);

            // Remaining TTL should be close to original TTL
            prop_assert!(entry.remaining_ttl_secs() >= ttl_secs - 2);
        }

        #[test]
        fn cache_entry_expired_after_ttl(
            cached_at_offset in 100u64..10000u64,
            ttl_secs in 1u64..100u64
        ) {
            // Create an entry that was cached in the past (simulating expiry)
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Set cached_at to be far enough in the past that TTL has expired
            let cached_at = now.saturating_sub(cached_at_offset);

            let entry = CacheEntry {
                data: "test_data",
                cached_at,
                ttl_secs,
            };

            // If cached_at + ttl_secs < now, entry should be invalid
            if cached_at + ttl_secs < now {
                prop_assert!(!entry.is_valid());
                prop_assert_eq!(entry.remaining_ttl_secs(), 0);
            }
        }

        #[test]
        fn cache_entry_age_increases_with_time(ttl_secs in 10u64..3600u64) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Create entry with a past timestamp
            let past_offset = 100u64;
            let entry = CacheEntry {
                data: "test_data",
                cached_at: now.saturating_sub(past_offset),
                ttl_secs,
            };

            // Age should be approximately the offset
            let age = entry.age_secs();
            prop_assert!(age >= past_offset - 1 && age <= past_offset + 1);
        }
    }
}
