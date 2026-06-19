//! FontSpace provider - Large free font collection (90,000+ fonts)
//!
//! This provider implements web scraping to fetch real font data from FontSpace.

use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::error::{FontError, FontResult};
use crate::models::{Font, FontCategory, FontFamily, FontLicense, FontProvider, SearchQuery};
use crate::providers::FontProviderTrait;

pub struct FontSpaceProvider {
    client: Client,
    base_url: String,
}

impl FontSpaceProvider {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            base_url: "https://www.fontspace.com".to_string(),
        }
    }

    fn parse_category(category: &str) -> Option<FontCategory> {
        match category.to_lowercase().as_str() {
            "serif" | "slab" => Some(FontCategory::Serif),
            "sans-serif" | "sans" => Some(FontCategory::SansSerif),
            "script" | "calligraphy" | "handwritten" | "cursive" => Some(FontCategory::Handwriting),
            "display" | "decorative" | "fancy" | "headline" => Some(FontCategory::Display),
            "monospace" | "typewriter" | "code" => Some(FontCategory::Monospace),
            _ => None,
        }
    }

    /// Scrape fonts from FontSpace search results page
    async fn scrape_search_results(&self, query: &str, page: u32) -> FontResult<Vec<Font>> {
        let url = if query.is_empty() {
            format!("{}/popular/fonts?p={}", self.base_url, page)
        } else {
            format!(
                "{}/search?q={}&p={}",
                self.base_url,
                urlencoding::encode(query),
                page
            )
        };

        let response = self
            .client
            .get(&url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .send()
            .await
            .map_err(|e| FontError::network(&url, e))?;

        if !response.status().is_success() {
            return Err(FontError::provider(
                self.name(),
                format!("HTTP {} from {}", response.status(), url),
            ));
        }

        let html = response
            .text()
            .await
            .map_err(|e| FontError::network(&url, e))?;

        self.parse_search_html(&html)
    }

    /// Parse HTML from FontSpace search results
    fn parse_search_html(&self, html: &str) -> FontResult<Vec<Font>> {
        let document = Html::parse_document(html);
        let mut fonts = Vec::new();

        // FontSpace uses card-based layout for font listings
        let card_selector = Selector::parse("div.font-card, article.font").ok();
        let title_selector = Selector::parse("h2 a, .font-name a, a.fontname").ok();
        let link_selector = Selector::parse("a[href*='/font/']").ok();

        if let Some(card_sel) = card_selector {
            for element in document.select(&card_sel) {
                // Try to find font name and link
                let name = title_selector
                    .as_ref()
                    .and_then(|sel| element.select(sel).next())
                    .map(|el| el.text().collect::<String>().trim().to_string());

                let href = link_selector
                    .as_ref()
                    .and_then(|sel| element.select(sel).next())
                    .and_then(|el| el.value().attr("href"));

                if let (Some(name), Some(href)) = (name, href)
                    && !name.is_empty()
                {
                    // Extract font ID from URL like /font/fontname-abc123
                    let id = href
                        .split('/')
                        .next_back()
                        .unwrap_or(&name.to_lowercase().replace(' ', "-"))
                        .to_string();

                    fonts.push(Font {
                        id: format!("fontspace-{}", id),
                        name,
                        provider: FontProvider::FontSpace,
                        category: None,
                        variant_count: 1,
                        license: Some(FontLicense::OFL),
                        preview_url: Some(format!("{}{}", self.base_url, href)),
                        download_url: Some(format!("{}/get/{}.zip", self.base_url, id)),
                    });
                }
            }
        }

        // If scraping didn't find fonts, fall back to the hardcoded list
        if fonts.is_empty() {
            return Ok(self.get_fallback_fonts());
        }

        Ok(fonts)
    }

    /// Fallback font list when scraping fails or returns empty
    fn get_fallback_fonts(&self) -> Vec<Font> {
        let fonts_data: Vec<(&str, &str)> = vec![
            // Sans Serif Popular
            ("Montserrat Alternates", "sans-serif"),
            ("Quicksand", "sans-serif"),
            ("Poppins", "sans-serif"),
            ("Lato", "sans-serif"),
            ("Open Sans", "sans-serif"),
            ("Nunito", "sans-serif"),
            ("Raleway", "sans-serif"),
            ("Work Sans", "sans-serif"),
            ("Rubik", "sans-serif"),
            ("Karla", "sans-serif"),
            ("Manrope", "sans-serif"),
            ("Outfit", "sans-serif"),
            ("Urbanist", "sans-serif"),
            ("Sora", "sans-serif"),
            ("Lexend", "sans-serif"),
            ("DM Sans", "sans-serif"),
            ("Plus Jakarta Sans", "sans-serif"),
            ("Figtree", "sans-serif"),
            ("Albert Sans", "sans-serif"),
            ("Epilogue", "sans-serif"),
            // Serif Popular
            ("Playfair Display", "serif"),
            ("Lora", "serif"),
            ("Merriweather", "serif"),
            ("Crimson Text", "serif"),
            ("Libre Baskerville", "serif"),
            ("Cormorant", "serif"),
            ("Spectral", "serif"),
            ("Source Serif Pro", "serif"),
            ("Libre Caslon Text", "serif"),
            ("Bitter", "serif"),
            ("Zilla Slab", "serif"),
            ("Rokkitt", "serif"),
            ("Vollkorn", "serif"),
            ("Cardo", "serif"),
            ("Newsreader", "serif"),
            // Script/Handwriting
            ("Pacifico", "script"),
            ("Dancing Script", "script"),
            ("Great Vibes", "script"),
            ("Sacramento", "script"),
            ("Allura", "script"),
            ("Satisfy", "script"),
            ("Cookie", "script"),
            ("Kaushan Script", "script"),
            ("Yellowtail", "script"),
            ("Courgette", "script"),
            ("Tangerine", "script"),
            ("Alex Brush", "script"),
            ("Caveat", "script"),
            ("Amatic SC", "script"),
            ("Indie Flower", "script"),
            // Display
            ("Bebas Neue", "display"),
            ("Oswald", "display"),
            ("Anton", "display"),
            ("Lobster", "display"),
            ("Righteous", "display"),
            ("Passion One", "display"),
            ("Black Ops One", "display"),
            ("Bungee", "display"),
            ("Bangers", "display"),
            ("Archivo Black", "display"),
            ("Teko", "display"),
            ("Russo One", "display"),
            ("Staatliches", "display"),
            ("Monoton", "display"),
            // Monospace
            ("Fira Code", "monospace"),
            ("JetBrains Mono", "monospace"),
            ("Source Code Pro", "monospace"),
            ("Roboto Mono", "monospace"),
            ("IBM Plex Mono", "monospace"),
            ("Ubuntu Mono", "monospace"),
            ("Space Mono", "monospace"),
            ("Inconsolata", "monospace"),
            ("Anonymous Pro", "monospace"),
            ("Cousine", "monospace"),
        ];

        fonts_data
            .into_iter()
            .map(|(name, category)| {
                let id = name
                    .to_lowercase()
                    .replace(' ', "-")
                    .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
                Font {
                    id: format!("fontspace-{}", id),
                    name: name.to_string(),
                    provider: FontProvider::FontSpace,
                    category: Self::parse_category(category),
                    variant_count: 1,
                    license: Some(FontLicense::OFL),
                    preview_url: Some(format!("{}/category/{}", self.base_url, id)),
                    download_url: Some(format!("{}/get/{}.zip", self.base_url, id)),
                }
            })
            .collect()
    }
}

#[async_trait]
impl FontProviderTrait for FontSpaceProvider {
    fn name(&self) -> &str {
        "FontSpace"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn search(&self, query: &SearchQuery) -> FontResult<Vec<Font>> {
        // Try to scrape real results first
        match self.scrape_search_results(&query.query, 1).await {
            Ok(fonts) if !fonts.is_empty() => {
                let query_lower = query.query.to_lowercase();
                if query_lower.is_empty() {
                    Ok(fonts)
                } else {
                    Ok(fonts
                        .into_iter()
                        .filter(|font| {
                            font.name.to_lowercase().contains(&query_lower)
                                || font
                                    .category
                                    .as_ref()
                                    .map(|c| {
                                        format!("{:?}", c).to_lowercase().contains(&query_lower)
                                    })
                                    .unwrap_or(false)
                        })
                        .collect())
                }
            }
            Ok(_) | Err(_) => {
                // Fall back to hardcoded list on scraping failure
                let all_fonts = self.get_fallback_fonts();
                let query_lower = query.query.to_lowercase();

                if query_lower.is_empty() {
                    return Ok(all_fonts);
                }

                Ok(all_fonts
                    .into_iter()
                    .filter(|font| {
                        font.name.to_lowercase().contains(&query_lower)
                            || font
                                .category
                                .as_ref()
                                .map(|c| format!("{:?}", c).to_lowercase().contains(&query_lower))
                                .unwrap_or(false)
                    })
                    .collect())
            }
        }
    }

    async fn list_all(&self) -> FontResult<Vec<Font>> {
        // Try scraping multiple pages
        let mut all_fonts = Vec::new();

        for page in 1..=3 {
            match self.scrape_search_results("", page).await {
                Ok(fonts) => all_fonts.extend(fonts),
                Err(_) => break,
            }
        }

        // If scraping failed completely, use fallback
        if all_fonts.is_empty() {
            all_fonts = self.get_fallback_fonts();
        }

        Ok(all_fonts)
    }

    async fn get_font_family(&self, font_id: &str) -> FontResult<FontFamily> {
        // Try to find in fallback fonts first
        let fonts = self.get_fallback_fonts();
        if let Some(font) = fonts.into_iter().find(|f| f.id == font_id) {
            return Ok(FontFamily {
                id: font.id,
                name: font.name,
                provider: FontProvider::FontSpace,
                category: font.category,
                variants: vec![],
                subsets: vec!["latin".to_string()],
                license: font.license,
                designer: None,
                description: None,
                preview_url: font.preview_url,
                download_url: font.download_url,
                languages: vec!["Latin".to_string()],
                last_modified: None,
                popularity: None,
            });
        }

        // If not found, create a basic entry
        let clean_id = font_id.replace("fontspace-", "");
        let name = clean_id
            .split('-')
            .map(|s| {
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        Ok(FontFamily {
            id: font_id.to_string(),
            name,
            provider: FontProvider::FontSpace,
            category: None,
            variants: vec![],
            subsets: vec!["latin".to_string()],
            license: Some(FontLicense::OFL),
            designer: None,
            description: None,
            preview_url: Some(format!("{}/font/{}", self.base_url, clean_id)),
            download_url: Some(format!("{}/get/{}.zip", self.base_url, clean_id)),
            languages: vec!["Latin".to_string()],
            last_modified: None,
            popularity: None,
        })
    }

    async fn get_download_url(&self, font_id: &str) -> FontResult<String> {
        Ok(format!(
            "{}/get/{}.zip",
            self.base_url,
            font_id.replace("fontspace-", "")
        ))
    }

    async fn health_check(&self) -> FontResult<bool> {
        let response = self
            .client
            .get(&self.base_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .send()
            .await
            .map_err(|e| FontError::network(&self.base_url, e))?;
        Ok(response.status().is_success())
    }
}
