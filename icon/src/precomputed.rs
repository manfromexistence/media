use crate::bloom::IconBloomFilters;
use crate::perfect_hash::{LowercaseCache, PerfectHashIndex};
/// Pre-computed indices for instant lookups
/// All expensive computations done once at startup
use crate::types::IconMetadata;
use std::collections::HashMap;

/// Pre-computed search index (built once, used forever)
pub struct PrecomputedIndex {
    /// Perfect hash for O(1) exact matches
    pub perfect_hash: PerfectHashIndex,

    /// Pre-computed lowercase names (zero allocation)
    pub lowercase_cache: LowercaseCache,

    /// Bloom filters for fast rejection
    pub bloom_filters: IconBloomFilters,

    /// Prefix index for fast prefix matching
    pub prefix_index: PrefixIndex,

    /// Metadata reference
    pub metadata: Vec<IconMetadata>,
}

impl PrecomputedIndex {
    /// Build all indices (done once at startup)
    pub fn build(metadata: Vec<IconMetadata>) -> Self {
        // Build perfect hash index
        let perfect_hash = PerfectHashIndex::build(&metadata);

        // Build lowercase cache
        let lowercase_cache = LowercaseCache::build(&metadata);

        // Build bloom filters
        let lowercase_names: Vec<String> = metadata.iter().map(|m| m.name.to_lowercase()).collect();
        let bloom_filters = IconBloomFilters::build(&lowercase_names);

        // Build prefix index
        let prefix_index = PrefixIndex::build(&lowercase_names);

        Self {
            perfect_hash,
            lowercase_cache,
            bloom_filters,
            prefix_index,
            metadata,
        }
    }
}

/// Prefix trie for fast prefix matching
pub struct PrefixIndex {
    /// Map of prefix -> list of icon indices
    /// Pre-computed for all 1-3 char prefixes
    prefix_map: HashMap<String, Vec<u32>>,
}

impl PrefixIndex {
    /// Build prefix index
    pub fn build(lowercase_names: &[String]) -> Self {
        let mut prefix_map: HashMap<String, Vec<u32>> = HashMap::new();

        for (idx, name) in lowercase_names.iter().enumerate() {
            // Index all prefixes up to 3 chars
            for len in 1..=3.min(name.len()) {
                let prefix = &name[..len];
                prefix_map
                    .entry(prefix.to_string())
                    .or_insert_with(Vec::new)
                    .push(idx as u32);
            }
        }

        Self { prefix_map }
    }

    /// Get candidates for prefix (O(1) lookup)
    #[inline(always)]
    pub fn get_candidates(&self, prefix: &str) -> Option<&[u32]> {
        if prefix.len() <= 3 {
            self.prefix_map.get(prefix).map(|v| v.as_slice())
        } else {
            // For longer prefixes, use first 3 chars
            self.prefix_map.get(&prefix[..3]).map(|v| v.as_slice())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_index() {
        let names = vec!["home".to_string(), "house".to_string(), "arrow".to_string()];

        let index = PrefixIndex::build(&names);

        let candidates = index.get_candidates("ho").unwrap();
        assert_eq!(candidates.len(), 2); // home, house

        let candidates = index.get_candidates("ar").unwrap();
        assert_eq!(candidates.len(), 1); // arrow
    }
}
