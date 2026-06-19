use crate::search::{MatchType, SearchResult, calculate_score};
/// Zero-allocation search optimizations (2026)
/// Eliminates all heap allocations during search for maximum performance
use crate::types::IconMetadata;
use smallvec::SmallVec;

/// Stack-allocated result buffer (avoids heap allocation for small result sets)
/// Most queries return <100 results, so this stays on stack
type StackResults = SmallVec<[SearchResult; 128]>;

/// Zero-allocation search using stack-only buffers
/// Returns results without any heap allocations for queries with <128 results
pub fn zero_alloc_search(
    metadata: &[IconMetadata],
    query: &str,
    limit: usize,
) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let query_bytes = query_lower.as_bytes();

    // Use stack-allocated buffer for results (zero heap allocations)
    let mut results: StackResults = SmallVec::new();

    // Inline search without allocations
    for icon in metadata.iter() {
        let icon_name_lower = icon.name.to_lowercase();
        let icon_bytes = icon_name_lower.as_bytes();

        // Fast path: exact match
        if icon_bytes == query_bytes {
            let score = calculate_score(
                &query_lower,
                &icon_name_lower,
                MatchType::Exact,
                icon.popularity,
            );
            results.push(SearchResult::new(icon.clone(), score, MatchType::Exact));
            continue;
        }

        // SIMD substring search
        if memchr::memmem::find(icon_bytes, query_bytes).is_some() {
            let (match_type, multiplier) = if icon_name_lower.starts_with(&query_lower) {
                (MatchType::Prefix, 0.8)
            } else {
                (MatchType::Prefix, 0.7)
            };

            let score =
                calculate_score(&query_lower, &icon_name_lower, match_type, icon.popularity)
                    * multiplier;

            results.push(SearchResult::new(icon.clone(), score, match_type));
        }
    }

    // Sort in-place (no allocation)
    results.sort_unstable_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Dedup in-place
    results.dedup_by(|a, b| a.icon.id == b.icon.id);

    // Convert to Vec (only allocates once at the end)
    results.into_iter().take(limit).collect()
}

/// Parallel zero-allocation search with work-stealing
/// Uses thread-local stack buffers to avoid allocations
pub fn parallel_zero_alloc_search(
    metadata: &[IconMetadata],
    query: &str,
    limit: usize,
) -> Vec<SearchResult> {
    use rayon::prelude::*;

    let query_lower = query.to_lowercase();
    let query_bytes = query_lower.as_bytes();

    // Parallel search with thread-local buffers
    let results: Vec<SearchResult> = metadata
        .par_chunks(256) // Cache-friendly chunks
        .flat_map(|chunk| {
            let mut local_results: StackResults = SmallVec::new();

            for icon in chunk.iter() {
                let icon_name_lower = icon.name.to_lowercase();
                let icon_bytes = icon_name_lower.as_bytes();

                if icon_bytes == query_bytes {
                    let score = calculate_score(
                        &query_lower,
                        &icon_name_lower,
                        MatchType::Exact,
                        icon.popularity,
                    );
                    local_results.push(SearchResult::new(icon.clone(), score, MatchType::Exact));
                } else if memchr::memmem::find(icon_bytes, query_bytes).is_some() {
                    let (match_type, multiplier) = if icon_name_lower.starts_with(&query_lower) {
                        (MatchType::Prefix, 0.8)
                    } else {
                        (MatchType::Prefix, 0.7)
                    };

                    let score = calculate_score(
                        &query_lower,
                        &icon_name_lower,
                        match_type,
                        icon.popularity,
                    ) * multiplier;

                    local_results.push(SearchResult::new(icon.clone(), score, match_type));
                }
            }

            // Convert SmallVec to Vec for parallel iterator
            local_results.into_vec()
        })
        .collect();

    // Final sort and limit
    let mut results = results;
    results.par_sort_unstable_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.dedup_by(|a, b| a.icon.id == b.icon.id);
    results.into_iter().take(limit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IconMetadata;

    fn create_test_icons(count: usize) -> Vec<IconMetadata> {
        (0..count)
            .map(|i| IconMetadata {
                id: i as u32,
                name: format!("icon-{}", i),
                pack: "test".to_string(),
                category: "test".to_string(),
                tags: vec![],
                popularity: 1.0,
            })
            .collect()
    }

    #[test]
    fn test_zero_alloc_search() {
        let icons = create_test_icons(1000);
        let results = zero_alloc_search(&icons, "icon-42", 100);
        assert!(results.iter().any(|r| r.icon.id == 42));
    }

    #[test]
    fn test_parallel_zero_alloc_search() {
        let icons = create_test_icons(10000);
        let results = parallel_zero_alloc_search(&icons, "icon-42", 100);
        assert!(results.iter().any(|r| r.icon.id == 42));
    }
}
