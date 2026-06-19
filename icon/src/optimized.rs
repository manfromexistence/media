/// Const generic optimizations for common query lengths
/// Compiler generates specialized code for each length at compile time
use crate::types::IconMetadata;

/// Optimized exact match for short queries (compile-time specialized)
#[inline(always)]
pub fn exact_match_const<const N: usize>(query: &[u8; N], icon_name: &str) -> bool {
    let icon_bytes = icon_name.as_bytes();
    if icon_bytes.len() != N {
        return false;
    }

    // Compiler unrolls this loop at compile time
    for i in 0..N {
        if query[i] != icon_bytes[i] {
            return false;
        }
    }
    true
}

/// Fast path for common query lengths (2-10 chars)
/// Uses const generics to eliminate branches
#[inline]
pub fn fast_exact_match(query: &str, icon_name: &str) -> bool {
    let query_bytes = query.as_bytes();

    match query_bytes.len() {
        2 => {
            let arr: &[u8; 2] = query_bytes.try_into().unwrap();
            exact_match_const(arr, icon_name)
        }
        3 => {
            let arr: &[u8; 3] = query_bytes.try_into().unwrap();
            exact_match_const(arr, icon_name)
        }
        4 => {
            let arr: &[u8; 4] = query_bytes.try_into().unwrap();
            exact_match_const(arr, icon_name)
        }
        5 => {
            let arr: &[u8; 5] = query_bytes.try_into().unwrap();
            exact_match_const(arr, icon_name)
        }
        6 => {
            let arr: &[u8; 6] = query_bytes.try_into().unwrap();
            exact_match_const(arr, icon_name)
        }
        7 => {
            let arr: &[u8; 7] = query_bytes.try_into().unwrap();
            exact_match_const(arr, icon_name)
        }
        8 => {
            let arr: &[u8; 8] = query_bytes.try_into().unwrap();
            exact_match_const(arr, icon_name)
        }
        _ => query == icon_name,
    }
}

/// Vectorized comparison for 16-byte chunks (SSE/NEON)
#[cfg(target_feature = "sse2")]
#[inline]
pub fn simd_prefix_match(query: &[u8], icon: &[u8]) -> bool {
    use std::arch::x86_64::*;

    if query.len() > icon.len() {
        return false;
    }

    unsafe {
        let query_ptr = query.as_ptr();
        let icon_ptr = icon.as_ptr();

        // Load 16 bytes at a time
        let query_vec = _mm_loadu_si128(query_ptr as *const __m128i);
        let icon_vec = _mm_loadu_si128(icon_ptr as *const __m128i);

        // Compare all 16 bytes simultaneously
        let cmp = _mm_cmpeq_epi8(query_vec, icon_vec);
        let mask = _mm_movemask_epi8(cmp);

        // Check if first query.len() bytes match
        let query_mask = (1 << query.len()) - 1;
        (mask & query_mask) == query_mask
    }
}

/// Batch icon filtering with const generic chunk size
pub fn filter_icons_batch<const CHUNK: usize>(
    icons: &[IconMetadata],
    predicate: impl Fn(&IconMetadata) -> bool + Sync,
) -> Vec<&IconMetadata> {
    use rayon::prelude::*;

    icons
        .par_chunks(CHUNK)
        .flat_map(|chunk| {
            chunk
                .iter()
                .filter(|icon| predicate(icon))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_match() {
        assert!(fast_exact_match("home", "home"));
        assert!(!fast_exact_match("home", "house"));
    }
}
