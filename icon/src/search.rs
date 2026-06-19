use crate::types::IconMetadata;

/// Search result with relevance score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub icon: IconMetadata,
    pub score: f32,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    Exact,
    Prefix,
    Fuzzy,
    Semantic,
}

impl SearchResult {
    pub fn new(icon: IconMetadata, score: f32, match_type: MatchType) -> Self {
        Self {
            icon,
            score,
            match_type,
        }
    }
}

/// Calculate relevance score based on multiple factors
pub fn calculate_score(
    query: &str,
    icon_name: &str,
    match_type: MatchType,
    popularity: u32,
) -> f32 {
    let mut score = match match_type {
        MatchType::Exact => 100.0,
        MatchType::Prefix => 80.0,
        MatchType::Fuzzy => 50.0,
        MatchType::Semantic => 40.0,
    };

    // Boost score for shorter names (more specific)
    let length_factor = 1.0 / (icon_name.len() as f32).sqrt();
    score *= 1.0 + length_factor;

    // Boost score based on popularity
    let popularity_factor = (popularity as f32).log10().max(0.0);
    score *= 1.0 + (popularity_factor * 0.1);

    // Boost if query matches start of name
    if icon_name.starts_with(query) {
        score *= 1.5;
    }

    score
}

/// SIMD-accelerated fuzzy match using triple_accel (20-30x faster)
/// Falls back to nucleo for more sophisticated matching
pub fn fuzzy_match(query: &str, target: &str, threshold: f64) -> Option<f32> {
    use triple_accel::levenshtein::levenshtein_simd_k;

    let max_distance = ((1.0 - threshold) * query.len() as f64).ceil() as u32;

    // SIMD-accelerated Levenshtein distance
    match levenshtein_simd_k(query.as_bytes(), target.as_bytes(), max_distance) {
        Some(distance) => {
            let similarity = 1.0 - (distance as f32 / query.len().max(target.len()) as f32);
            if similarity >= threshold as f32 {
                Some(similarity)
            } else {
                None
            }
        }
        None => None,
    }
}

/// Advanced fuzzy matching using Nucleo (Helix editor's matcher)
/// Use for more sophisticated pattern matching
#[allow(dead_code)]
pub fn fuzzy_match_nucleo(query: &str, target: &str) -> Option<f32> {
    use nucleo_matcher::{Config, Matcher, Utf32Str};

    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut target_buf = Vec::new();
    let mut query_buf = Vec::new();
    let target_utf32 = Utf32Str::new(target, &mut target_buf);
    let query_utf32 = Utf32Str::new(query, &mut query_buf);

    matcher
        .fuzzy_match(target_utf32, query_utf32)
        .map(|score| score as f32 / 1000.0)
}
