//! Scraping target definitions and types.

use serde::{Deserialize, Serialize};

/// Method for scraping a website.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrapingMethod {
    /// Standard HTML parsing with CSS selectors
    Html,
    /// Sitemap-based discovery
    Sitemap,
    /// Tumblr blog scraping
    Tumblr,
    /// Direct file listing
    Direct,
    /// JSON API (pseudo-scraping)
    JsonApi,
    /// RSS/Atom feed parsing
    Feed,
}

/// Category of media content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrapingCategory {
    /// Stock photos and images
    Images,
    /// Video footage
    Videos,
    /// Audio, music, and sound effects
    Audio,
    /// 3D models and assets
    Models3D,
    /// Textures and materials
    Textures,
    /// Vector graphics and illustrations
    Vectors,
    /// Documents and datasets
    Documents,
    /// Game assets and sprites
    GameAssets,
    /// Patterns and backgrounds
    Patterns,
    /// Maps and geographic data
    Maps,
}

/// A pre-configured scraping target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingTarget {
    /// Unique identifier for the target.
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Base URL of the website.
    pub base_url: &'static str,
    /// Search URL pattern (use {query} for search term, {page} for pagination).
    pub search_url: Option<&'static str>,
    /// CSS selector for media items container.
    pub container_selector: Option<&'static str>,
    /// CSS selector for individual media items.
    pub item_selector: &'static str,
    /// CSS selector for the image/media URL within an item.
    pub media_selector: &'static str,
    /// CSS selector for the title/alt text.
    pub title_selector: Option<&'static str>,
    /// CSS selector for download link (if different from media).
    pub download_selector: Option<&'static str>,
    /// CSS selector for pagination/next page.
    pub pagination_selector: Option<&'static str>,
    /// Scraping method to use.
    pub method: ScrapingMethod,
    /// Category of content.
    pub category: ScrapingCategory,
    /// Estimated number of assets available.
    pub estimated_assets: &'static str,
    /// License type (CC0, Free, etc.).
    pub license: &'static str,
    /// Recommended delay between requests (milliseconds).
    pub rate_limit_ms: u64,
    /// Whether the site requires JavaScript rendering.
    pub requires_js: bool,
    /// Additional notes or requirements.
    pub notes: Option<&'static str>,
}

impl ScrapingTarget {
    /// Create a new scraping target with minimal required fields.
    pub const fn new(
        id: &'static str,
        name: &'static str,
        base_url: &'static str,
        item_selector: &'static str,
        media_selector: &'static str,
        category: ScrapingCategory,
        license: &'static str,
        estimated_assets: &'static str,
    ) -> Self {
        Self {
            id,
            name,
            base_url,
            search_url: None,
            container_selector: None,
            item_selector,
            media_selector,
            title_selector: None,
            download_selector: None,
            pagination_selector: None,
            method: ScrapingMethod::Html,
            category,
            estimated_assets,
            license,
            rate_limit_ms: 1000,
            requires_js: false,
            notes: None,
        }
    }

    /// Set the search URL pattern.
    pub const fn with_search_url(mut self, url: &'static str) -> Self {
        self.search_url = Some(url);
        self
    }

    /// Set the container selector.
    pub const fn with_container(mut self, selector: &'static str) -> Self {
        self.container_selector = Some(selector);
        self
    }

    /// Set the title selector.
    pub const fn with_title_selector(mut self, selector: &'static str) -> Self {
        self.title_selector = Some(selector);
        self
    }

    /// Set the download selector.
    pub const fn with_download_selector(mut self, selector: &'static str) -> Self {
        self.download_selector = Some(selector);
        self
    }

    /// Set the pagination selector.
    pub const fn with_pagination(mut self, selector: &'static str) -> Self {
        self.pagination_selector = Some(selector);
        self
    }

    /// Set the scraping method.
    pub const fn with_method(mut self, method: ScrapingMethod) -> Self {
        self.method = method;
        self
    }

    /// Set the rate limit.
    pub const fn with_rate_limit(mut self, ms: u64) -> Self {
        self.rate_limit_ms = ms;
        self
    }

    /// Mark as requiring JavaScript.
    pub const fn requires_javascript(mut self) -> Self {
        self.requires_js = true;
        self
    }

    /// Add notes.
    pub const fn with_notes(mut self, notes: &'static str) -> Self {
        self.notes = Some(notes);
        self
    }
}
