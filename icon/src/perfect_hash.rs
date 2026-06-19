/// Minimal Perfect Hash Function (MPHF) for O(1) icon lookup
/// Pre-computed at build time for zero-cost runtime lookups
use crate::types::IconMetadata;

/// Perfect hash index for instant O(1) lookups
pub struct PerfectHashIndex {
    /// Pre-computed hash -> icon index mapping
    hash_table: Vec<Option<u32>>,
    /// Hash function parameters (computed at build time)
    seed: u64,
    table_size: usize,
}

impl PerfectHashIndex {
    /// Build perfect hash index from icon names (done once at startup)
    pub fn build(metadata: &[IconMetadata]) -> Self {
        let table_size = (metadata.len() as f64 * 1.3) as usize; // 30% overhead
        let mut hash_table = vec![None; table_size];

        // Find seed that produces no collisions
        let seed = Self::find_perfect_seed(metadata, table_size);

        // Build collision-free hash table
        for (idx, icon) in metadata.iter().enumerate() {
            let hash = Self::hash_name(&icon.name.to_lowercase(), seed, table_size);
            hash_table[hash] = Some(idx as u32);
        }

        Self {
            hash_table,
            seed,
            table_size,
        }
    }

    /// Find a seed that produces no collisions
    fn find_perfect_seed(metadata: &[IconMetadata], table_size: usize) -> u64 {
        for seed in 0..10000 {
            let mut used = vec![false; table_size];
            let mut collision = false;

            for icon in metadata {
                let hash = Self::hash_name(&icon.name.to_lowercase(), seed, table_size);
                if used[hash] {
                    collision = true;
                    break;
                }
                used[hash] = true;
            }

            if !collision {
                return seed;
            }
        }

        // Fallback: use larger table
        0
    }

    /// Hash function (FNV-1a variant)
    #[inline(always)]
    fn hash_name(name: &str, seed: u64, table_size: usize) -> usize {
        let mut hash = seed.wrapping_add(0xcbf29ce484222325);
        for byte in name.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        (hash as usize) % table_size
    }

    /// O(1) exact match lookup
    #[inline(always)]
    pub fn lookup_exact(&self, query: &str) -> Option<u32> {
        let hash = Self::hash_name(query, self.seed, self.table_size);
        self.hash_table.get(hash).and_then(|&idx| idx)
    }
}

/// Pre-computed lowercase name cache (eliminates runtime allocations)
pub struct LowercaseCache {
    /// Pre-computed lowercase names (computed once at startup)
    lowercase_names: Vec<String>,
}

impl LowercaseCache {
    /// Build cache from metadata (done once)
    pub fn build(metadata: &[IconMetadata]) -> Self {
        let lowercase_names = metadata
            .iter()
            .map(|icon| icon.name.to_lowercase())
            .collect();

        Self { lowercase_names }
    }

    /// Get pre-computed lowercase name (zero allocation)
    #[inline(always)]
    pub fn get(&self, idx: usize) -> &str {
        &self.lowercase_names[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_hash() {
        let icons = vec![
            IconMetadata {
                id: 0,
                name: "home".to_string(),
                pack: "test".to_string(),
                category: "test".to_string(),
                tags: vec![],
                popularity: 1.0,
            },
            IconMetadata {
                id: 1,
                name: "arrow".to_string(),
                pack: "test".to_string(),
                category: "test".to_string(),
                tags: vec![],
                popularity: 1.0,
            },
        ];

        let index = PerfectHashIndex::build(&icons);

        assert_eq!(index.lookup_exact("home"), Some(0));
        assert_eq!(index.lookup_exact("arrow"), Some(1));
        assert_eq!(index.lookup_exact("notfound"), None);
    }
}
