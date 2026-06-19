//! Scryfall provider - Magic: The Gathering card images.
//!
//! Scryfall is the most comprehensive MTG card database:
//! - 80,000+ unique cards with multiple art versions
//! - High-resolution scans (PNG, border crop, art crop)
//! - All sets, promos, foils, special editions
//! - No API key required
//! - Excellent search syntax
//!
//! API: <https://scryfall.com/docs/api>

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

use crate::config::Config;
use crate::error::Result;
use crate::http::{HttpClient, ResponseExt};
use crate::providers::traits::Provider;
use crate::types::{License, MediaAsset, MediaType, RateLimitConfig, SearchQuery, SearchResult};

/// Scryfall MTG card provider.
#[derive(Debug)]
pub struct ScryfallProvider {
    client: HttpClient,
}

/// Card image URIs.
#[derive(Debug, Deserialize)]
struct ImageUris {
    small: Option<String>,
    normal: Option<String>,
    large: Option<String>,
    png: Option<String>,
    #[allow(dead_code)]
    art_crop: Option<String>,
    #[allow(dead_code)]
    border_crop: Option<String>,
}

/// Scryfall card response.
#[derive(Debug, Deserialize)]
struct ScryfallCard {
    id: String,
    name: String,
    #[serde(default)]
    set_name: String,
    #[serde(default)]
    artist: Option<String>,
    #[serde(default)]
    image_uris: Option<ImageUris>,
    #[serde(default)]
    card_faces: Option<Vec<CardFace>>,
    scryfall_uri: String,
}

/// Card face for double-faced cards.
#[derive(Debug, Deserialize)]
struct CardFace {
    #[allow(dead_code)]
    name: String,
    #[serde(default)]
    image_uris: Option<ImageUris>,
}

/// Search response.
#[derive(Debug, Deserialize)]
struct SearchResponse {
    total_cards: usize,
    #[allow(dead_code)]
    has_more: bool,
    data: Vec<ScryfallCard>,
}

impl ScryfallProvider {
    /// Create a new Scryfall provider.
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

    /// Rate limit: 10 requests/second
    const RATE_LIMIT: RateLimitConfig = RateLimitConfig::new(10, 1);

    /// Get the best image URL from a card.
    fn get_best_image(card: &ScryfallCard) -> Option<String> {
        // Try main image URIs first
        if let Some(ref uris) = card.image_uris {
            return uris
                .large
                .clone()
                .or_else(|| uris.png.clone())
                .or_else(|| uris.normal.clone());
        }

        // For double-faced cards, get the front face
        if let Some(ref faces) = card.card_faces {
            if let Some(face) = faces.first() {
                if let Some(ref uris) = face.image_uris {
                    return uris
                        .large
                        .clone()
                        .or_else(|| uris.png.clone())
                        .or_else(|| uris.normal.clone());
                }
            }
        }

        None
    }

    /// Get preview (smaller) image URL.
    fn get_preview_image(card: &ScryfallCard) -> Option<String> {
        if let Some(ref uris) = card.image_uris {
            return uris.normal.clone().or_else(|| uris.small.clone());
        }

        if let Some(ref faces) = card.card_faces {
            if let Some(face) = faces.first() {
                if let Some(ref uris) = face.image_uris {
                    return uris.normal.clone().or_else(|| uris.small.clone());
                }
            }
        }

        None
    }

    /// Convert card to media asset.
    fn card_to_asset(&self, card: ScryfallCard) -> Option<MediaAsset> {
        let download_url = Self::get_best_image(&card)?;
        let preview_url = Self::get_preview_image(&card).unwrap_or_else(|| download_url.clone());

        let title = if card.set_name.is_empty() {
            card.name.clone()
        } else {
            format!("{} ({})", card.name, card.set_name)
        };

        let mut builder = MediaAsset::builder()
            .id(format!("scryfall_{}", card.id))
            .provider(self.name().to_string())
            .title(title)
            .media_type(MediaType::Image)
            .direct_download_url(download_url)
            .preview_url(preview_url)
            .source_url(card.scryfall_uri)
            .license(License::Other("Wizards of the Coast".to_string()));

        if let Some(artist) = card.artist {
            builder = builder.author(artist);
        }

        builder.build_or_log()
    }
}

#[async_trait]
impl Provider for ScryfallProvider {
    fn name(&self) -> &'static str {
        "scryfall"
    }

    fn display_name(&self) -> &'static str {
        "Scryfall"
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
        "https://api.scryfall.com"
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        // Build search URL - Scryfall has excellent search syntax
        let search_query = if query.query.contains(':') {
            // User is using Scryfall syntax already
            query.query.clone()
        } else {
            // Simple name search
            format!("name:{}", query.query)
        };

        // URL encode the search query
        let encoded_query: String = search_query
            .chars()
            .flat_map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | ':' => vec![c],
                ' ' => vec!['+'],
                _ => format!("%{:02X}", c as u8).chars().collect(),
            })
            .collect();

        let url = format!(
            "{}/cards/search?q={}&unique=art&order=released&dir=desc",
            self.base_url(),
            encoded_query
        );

        let response = self.client.get(&url).await?;

        // Handle 404 for no results
        if response.status().as_u16() == 404 {
            return Ok(SearchResult {
                query: query.query.clone(),
                media_type: query.media_type,
                total_count: 0,
                assets: vec![],
                providers_searched: vec![self.name().to_string()],
                provider_errors: vec![],
                duration_ms: 0,
                provider_timings: Default::default(),
            });
        }

        let search_result: SearchResponse = response.json_or_error().await?;

        let assets: Vec<MediaAsset> = search_result
            .data
            .into_iter()
            .take(query.count)
            .filter_map(|card| self.card_to_asset(card))
            .collect();

        Ok(SearchResult {
            query: query.query.clone(),
            media_type: query.media_type,
            total_count: search_result.total_cards,
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
        let provider = ScryfallProvider::new(&config);
        assert_eq!(provider.name(), "scryfall");
        assert_eq!(provider.display_name(), "Scryfall");
        assert!(provider.is_available());
        assert!(!provider.requires_api_key());
    }
}
