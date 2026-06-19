//! xkcd comics provider.
//!
//! xkcd is a webcomic by Randall Munroe with:
//! - 3,000+ comics and growing
//! - Direct JSON API (no key required)
//! - High-resolution images
//! - CC BY-NC 2.5 License
//!
//! API: <https://xkcd.com/json.html>

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::Provider;
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// xkcd comics provider.
#[derive(Debug)]
pub struct XkcdProvider {
    client: HttpClient,
}

// Fields are read during serde deserialization
#[allow(dead_code)]
/// xkcd comic response.
#[derive(Debug, Deserialize)]
struct XkcdComic {
    num: u32,
    title: String,
    safe_title: String,
    img: String,
    alt: String,
    #[serde(default)]
    year: String,
    #[serde(default)]
    month: String,
    #[serde(default)]
    day: String,
}

impl XkcdProvider {
    /// Create a new xkcd provider.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let client = HttpClient::with_config(
            Self::RATE_LIMIT,
            config.retry_attempts,
            Duration::from_secs(config.timeout_secs),
        )
        .unwrap_or_default();

        Self { client }
    }

    /// Rate limit: Be respectful
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(10, 60);

    /// Get the latest comic number.
    async fn get_latest_comic_num(&self) -> Result<u32> {
        let url = "https://xkcd.com/info.0.json";
        let response = self.client.get(url).await?;
        let comic: XkcdComic = response.json_or_error().await?;
        Ok(comic.num)
    }

    /// Get a specific comic by number.
    async fn get_comic(&self, num: u32) -> Result<XkcdComic> {
        let url = format!("https://xkcd.com/{}/info.0.json", num);
        let response = self.client.get(&url).await?;
        response.json_or_error().await
    }

    /// Convert comic to media asset.
    fn comic_to_asset(&self, comic: XkcdComic) -> Option<MediaAsset> {
        let source_url = format!("https://xkcd.com/{}", comic.num);
        let mut builder = MediaAsset::builder()
            .id(format!("xkcd_{}", comic.num))
            .provider(self.name().to_string())
            .title(comic.safe_title.clone())
            .media_type(MediaType::Image)
            .direct_download_url(comic.img.clone())
            .preview_url(comic.img.clone())
            .source_url(source_url)
            .license(License::CcByNc)
            .author("Randall Munroe")
            .provider_metadata_entry("xkcd.comic_number", comic.num.to_string())
            .provider_metadata_entry("xkcd.original_title", comic.title.clone())
            .provider_metadata_entry("xkcd.alt_text", comic.alt.clone())
            .provider_metadata_entry("xkcd.license_url", "https://xkcd.com/license.html")
            .provider_metadata_entry("xkcd.license_scope", "non-commercial");

        if let Some(date) = xkcd_publication_date(&comic) {
            builder = builder.provider_metadata_entry("xkcd.publication_date", date);
        }

        builder.build_or_log()
    }
}

fn xkcd_publication_date(comic: &XkcdComic) -> Option<String> {
    let year = comic.year.parse::<u32>().ok()?;
    let month = comic.month.parse::<u32>().ok()?;
    let day = comic.day.parse::<u32>().ok()?;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    Some(format!("{year:04}-{month:02}-{day:02}"))
}

#[async_trait]
impl Provider for XkcdProvider {
    fn name(&self) -> &'static str {
        "xkcd"
    }

    fn display_name(&self) -> &'static str {
        "xkcd Comics"
    }

    fn supported_media_types(&self) -> &[MediaType] {
        &[MediaType::Image]
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn rate_limit(&self) -> RateLimitConfig {
        Self::RATE_LIMIT
    }

    fn is_available(&self) -> bool {
        true
    }

    fn base_url(&self) -> &'static str {
        "https://xkcd.com"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        // Get latest comic number
        let latest = self.get_latest_comic_num().await?;
        let count = query.count.min(10); // Limit to avoid too many requests

        // Search strategy: get random comics or recent ones
        let query_lower = query.query.to_lowercase();

        let comic_nums: Vec<u32> = if query_lower.contains("latest")
            || query_lower.contains("recent")
            || query_lower.contains("new")
        {
            // Get most recent comics
            (latest.saturating_sub(count as u32 - 1)..=latest)
                .rev()
                .collect()
        } else {
            // Get a mix of random comics using query as seed
            use std::collections::HashSet;
            let mut nums = HashSet::new();
            let mut rng_seed = query
                .query
                .bytes()
                .fold(0u32, |acc, b| acc.wrapping_add(b as u32));

            while nums.len() < count && nums.len() < latest as usize {
                rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
                let num = (rng_seed % latest) + 1;
                // Skip comic 404 (it doesn't exist - it's a joke)
                if num != 404 {
                    nums.insert(num);
                }
            }
            nums.into_iter().collect()
        };

        // Fetch comics CONCURRENTLY for speed
        use futures::future::join_all;
        let futures: Vec<_> = comic_nums
            .into_iter()
            .take(count)
            .map(|num| self.get_comic(num))
            .collect();

        let results = join_all(futures).await;
        let assets: Vec<_> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .filter_map(|comic| self.comic_to_asset(comic))
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: latest as usize,
            assets,
            providers_searched: vec![self.name().to_string()],
            provider_errors: vec![],
            duration_ms: 0,
            provider_timings: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let config = Config::default();
        let provider = XkcdProvider::new(&config);
        assert_eq!(provider.name(), "xkcd");
        assert_eq!(provider.display_name(), "xkcd Comics");
        assert!(provider.is_available());
        assert!(!provider.requires_api_key());
    }

    #[test]
    fn xkcd_asset_preserves_comic_provenance() {
        let config = Config::default();
        let provider = XkcdProvider::new(&config);
        let asset = provider
            .comic_to_asset(XkcdComic {
                num: 353,
                title: "Python".to_string(),
                safe_title: "Python".to_string(),
                img: "https://imgs.xkcd.com/comics/python.png".to_string(),
                alt: "I wrote 20 short programs in Python yesterday.".to_string(),
                year: "2007".to_string(),
                month: "12".to_string(),
                day: "5".to_string(),
            })
            .expect("fixture should build an xkcd asset");

        let provenance = asset.provenance();

        assert_eq!(provenance.source_url, "https://xkcd.com/353");
        assert_eq!(provenance.license, "CC-BY-NC");
        assert!(provenance.license_known);
        assert_eq!(
            provenance.provider_metadata.get("xkcd.comic_number"),
            Some(&"353".to_string())
        );
        assert_eq!(
            provenance.provider_metadata.get("xkcd.original_title"),
            Some(&"Python".to_string())
        );
        assert_eq!(
            provenance.provider_metadata.get("xkcd.alt_text"),
            Some(&"I wrote 20 short programs in Python yesterday.".to_string())
        );
        assert_eq!(
            provenance.provider_metadata.get("xkcd.publication_date"),
            Some(&"2007-12-05".to_string())
        );
        assert_eq!(
            provenance.provider_metadata.get("xkcd.license_url"),
            Some(&"https://xkcd.com/license.html".to_string())
        );
        assert!(provenance.type_validation.has_evidence());
        assert!(provenance.type_validation.is_valid());
    }
}
