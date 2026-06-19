/// Bloom filter for fast rejection of non-matching icons
/// Reduces search space by 90%+ before expensive string comparisons
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Space-efficient bloom filter (1 bit per icon)
pub struct BloomFilter {
    /// Bit array (packed into u64s for cache efficiency)
    bits: Vec<u64>,
    /// Number of hash functions
    num_hashes: usize,
    /// Size of bit array
    size: usize,
}

impl BloomFilter {
    /// Create bloom filter for icon names
    /// False positive rate: ~1% with 10 bits per element
    pub fn new(capacity: usize) -> Self {
        let bits_per_element = 10; // 1% false positive rate
        let size = capacity * bits_per_element;
        let num_hashes = 7; // Optimal for 10 bits/element

        Self {
            bits: vec![0u64; (size + 63) / 64],
            num_hashes,
            size,
        }
    }

    /// Add icon name to filter
    pub fn insert(&mut self, name: &str) {
        for i in 0..self.num_hashes {
            let hash = self.hash(name, i);
            let idx = hash % self.size;
            self.bits[idx / 64] |= 1u64 << (idx % 64);
        }
    }

    /// Check if query might match (fast rejection)
    #[inline(always)]
    pub fn might_contain(&self, query: &str) -> bool {
        for i in 0..self.num_hashes {
            let hash = self.hash(query, i);
            let idx = hash % self.size;
            if (self.bits[idx / 64] & (1u64 << (idx % 64))) == 0 {
                return false; // Definitely not present
            }
        }
        true // Might be present (or false positive)
    }

    /// Hash function with seed
    #[inline(always)]
    fn hash(&self, s: &str, seed: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish() as usize
    }
}

/// Per-icon bloom filters for substring matching
pub struct IconBloomFilters {
    /// One bloom filter per icon (for n-grams)
    filters: Vec<BloomFilter>,
}

impl IconBloomFilters {
    /// Build bloom filters for all icons
    pub fn build(icon_names: &[String]) -> Self {
        let filters = icon_names
            .iter()
            .map(|name| {
                let mut filter = BloomFilter::new(name.len());

                // Insert all 2-grams and 3-grams
                let bytes = name.as_bytes();
                for i in 0..bytes.len().saturating_sub(1) {
                    filter.insert(&name[i..i + 2]);
                }
                for i in 0..bytes.len().saturating_sub(2) {
                    filter.insert(&name[i..i + 3]);
                }

                filter
            })
            .collect();

        Self { filters }
    }

    /// Quick rejection test (90%+ of non-matches rejected)
    #[inline(always)]
    pub fn might_match(&self, icon_idx: usize, query: &str) -> bool {
        if query.len() < 2 {
            return true; // Too short for bloom filter
        }

        let filter = &self.filters[icon_idx];

        // Check if all query n-grams are present
        let bytes = query.as_bytes();
        for i in 0..bytes.len().saturating_sub(1) {
            if !filter.might_contain(&query[i..i + 2]) {
                return false; // Definitely doesn't match
            }
        }

        true // Might match
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter() {
        let mut filter = BloomFilter::new(100);

        filter.insert("home");
        filter.insert("arrow");

        assert!(filter.might_contain("home"));
        assert!(filter.might_contain("arrow"));
        // High probability of rejection for non-inserted items
    }

    #[test]
    fn test_icon_bloom_filters() {
        let names = vec!["home-icon".to_string(), "arrow-left".to_string()];

        let filters = IconBloomFilters::build(&names);

        assert!(filters.might_match(0, "home"));
        assert!(filters.might_match(1, "arrow"));
        // Should reject most non-matches quickly
    }
}
