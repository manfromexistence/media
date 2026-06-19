use aho_corasick::AhoCorasick;
use daachorse::DoubleArrayAhoCorasick;

/// Multi-pattern search using Aho-Corasick for autocomplete
/// Searches multiple queries simultaneously in O(N+M) time
pub struct MultiPatternMatcher {
    ac: AhoCorasick,
}

impl MultiPatternMatcher {
    /// Build matcher from icon names
    pub fn new(patterns: &[String]) -> Self {
        let ac = AhoCorasick::new(patterns).unwrap();
        Self { ac }
    }

    /// Search for all patterns in text simultaneously
    pub fn find_all(&self, text: &str) -> Vec<usize> {
        self.ac
            .find_iter(text)
            .map(|mat| mat.pattern().as_usize())
            .collect()
    }
}

/// Double-array Aho-Corasick (fastest implementation)
/// Use for production - 2-3x faster than standard AC
pub struct FastMultiPatternMatcher {
    dac: DoubleArrayAhoCorasick<u32>,
}

impl FastMultiPatternMatcher {
    /// Build matcher from icon names
    pub fn new(patterns: &[String]) -> Self {
        let dac = DoubleArrayAhoCorasick::new(patterns).unwrap();
        Self { dac }
    }

    /// Search for all patterns in text simultaneously
    pub fn find_all(&self, text: &str) -> Vec<u32> {
        self.dac.find_iter(text).map(|mat| mat.value()).collect()
    }

    /// Find first match (best for autocomplete)
    pub fn find_first(&self, text: &str) -> Option<u32> {
        self.dac.find_iter(text).next().map(|mat| mat.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_pattern() {
        let patterns = vec![
            "home".to_string(),
            "arrow".to_string(),
            "search".to_string(),
        ];
        let matcher = FastMultiPatternMatcher::new(&patterns);

        let results = matcher.find_all("home-arrow-search");
        assert_eq!(results.len(), 3);
    }
}
