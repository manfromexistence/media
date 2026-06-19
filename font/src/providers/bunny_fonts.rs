//! Bunny Fonts provider implementation
//!
//! Bunny Fonts is a privacy-friendly alternative to Google Fonts with 1,478+ fonts.
//! API: <https://fonts.bunny.net>

use super::FontProviderTrait;
use crate::error::{FontError, FontResult};
use crate::models::{
    Font, FontCategory, FontFamily, FontLicense, FontProvider, FontStyle, FontVariant, FontWeight,
    SearchQuery,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

/// Bunny Fonts API response structure
#[derive(Debug, Deserialize)]
pub struct BunnyFontsResponse(pub HashMap<String, BunnyFont>);

/// Individual Bunny Font entry
#[derive(Debug, Deserialize)]
pub struct BunnyFont {
    #[serde(rename = "familyName")]
    pub family_name: String,
    pub category: String,
    pub styles: HashMap<String, BunnyFontStyle>,
    pub subsets: Option<Vec<String>>,
}

/// Font style information
#[derive(Debug, Deserialize)]
pub struct BunnyFontStyle {
    pub weight: Option<String>,
    pub style: Option<String>,
}

/// Bunny Fonts provider
pub struct BunnyFontsProvider {
    client: Client,
    api_url: String,
}

impl BunnyFontsProvider {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            api_url: "https://fonts.bunny.net/list".to_string(),
        }
    }

    fn parse_category(category: &str) -> Option<FontCategory> {
        match category.to_lowercase().as_str() {
            "serif" => Some(FontCategory::Serif),
            "sans-serif" => Some(FontCategory::SansSerif),
            "display" => Some(FontCategory::Display),
            "handwriting" => Some(FontCategory::Handwriting),
            "monospace" => Some(FontCategory::Monospace),
            _ => None,
        }
    }

    fn parse_weight(weight: &str) -> FontWeight {
        match weight {
            "100" => FontWeight::Thin,
            "200" => FontWeight::ExtraLight,
            "300" => FontWeight::Light,
            "400" | "regular" => FontWeight::Regular,
            "500" => FontWeight::Medium,
            "600" => FontWeight::SemiBold,
            "700" | "bold" => FontWeight::Bold,
            "800" => FontWeight::ExtraBold,
            "900" => FontWeight::Black,
            _ => FontWeight::Regular,
        }
    }
}

#[async_trait]
impl FontProviderTrait for BunnyFontsProvider {
    fn name(&self) -> &str {
        "Bunny Fonts"
    }

    fn base_url(&self) -> &str {
        "https://fonts.bunny.net"
    }

    async fn search(&self, query: &SearchQuery) -> FontResult<Vec<Font>> {
        let fonts = self.list_all().await?;

        let query_lower = query.query.to_lowercase();
        let filtered: Vec<Font> = fonts
            .into_iter()
            .filter(|f| f.name.to_lowercase().contains(&query_lower))
            .collect();

        Ok(filtered)
    }

    async fn list_all(&self) -> FontResult<Vec<Font>> {
        let response = self
            .client
            .get(&self.api_url)
            .send()
            .await
            .map_err(|e| FontError::network(&self.api_url, e))?;

        if !response.status().is_success() {
            return Err(FontError::provider(
                self.name(),
                format!("API returned status {}", response.status()),
            ));
        }

        let bunny_response: BunnyFontsResponse = response.json().await.map_err(|e| {
            FontError::parse(
                self.name(),
                format!("Failed to parse font list JSON: {}", e),
            )
        })?;

        let fonts: Vec<Font> = bunny_response
            .0
            .into_iter()
            .map(|(id, font)| Font {
                id: id.clone(),
                name: font.family_name.clone(),
                provider: FontProvider::BunnyFonts,
                category: Self::parse_category(&font.category),
                variant_count: font.styles.len(),
                license: Some(FontLicense::OFL),
                preview_url: Some(format!("https://fonts.bunny.net/family/{}", id)),
                download_url: Some(format!(
                    "https://fonts.bunny.net/css?family={}",
                    font.family_name.replace(' ', "+")
                )),
            })
            .collect();

        Ok(fonts)
    }

    async fn get_font_family(&self, font_id: &str) -> FontResult<FontFamily> {
        let response = self
            .client
            .get(&self.api_url)
            .send()
            .await
            .map_err(|e| FontError::network(&self.api_url, e))?;

        if !response.status().is_success() {
            return Err(FontError::provider(
                self.name(),
                format!("API returned status {}", response.status()),
            ));
        }

        let bunny_response: BunnyFontsResponse = response.json().await.map_err(|e| {
            FontError::parse(
                self.name(),
                format!("Failed to parse font list JSON: {}", e),
            )
        })?;

        let font = bunny_response.0.get(font_id).ok_or_else(|| {
            FontError::provider(self.name(), format!("Font not found: {}", font_id))
        })?;

        let variants: Vec<FontVariant> = font
            .styles
            .values()
            .map(|style| {
                let weight = style
                    .weight
                    .as_ref()
                    .map(|w| Self::parse_weight(w))
                    .unwrap_or(FontWeight::Regular);
                let is_italic = style.style.as_ref().map(|s| s == "italic").unwrap_or(false);

                FontVariant {
                    weight,
                    style: if is_italic {
                        FontStyle::Italic
                    } else {
                        FontStyle::Normal
                    },
                    file_url: None, // Bunny Fonts uses CSS delivery
                    file_format: "woff2".to_string(),
                }
            })
            .collect();

        Ok(FontFamily {
            id: font_id.to_string(),
            name: font.family_name.clone(),
            provider: FontProvider::BunnyFonts,
            category: Self::parse_category(&font.category),
            variants,
            license: Some(FontLicense::OFL),
            designer: None,
            description: None,
            preview_url: Some(format!("https://fonts.bunny.net/family/{}", font_id)),
            download_url: Some(format!(
                "https://fonts.bunny.net/css?family={}",
                font.family_name.replace(' ', "+")
            )),
            languages: vec!["Latin".to_string()],
            subsets: font.subsets.clone().unwrap_or_default(),
            popularity: None,
            last_modified: None,
        })
    }

    async fn get_download_url(&self, font_id: &str) -> FontResult<String> {
        let response = self
            .client
            .get(&self.api_url)
            .send()
            .await
            .map_err(|e| FontError::network(&self.api_url, e))?;

        if !response.status().is_success() {
            return Err(FontError::provider(
                self.name(),
                format!("API returned status {}", response.status()),
            ));
        }

        let bunny_response: BunnyFontsResponse = response.json().await.map_err(|e| {
            FontError::parse(
                self.name(),
                format!("Failed to parse font list JSON: {}", e),
            )
        })?;

        let font = bunny_response.0.get(font_id).ok_or_else(|| {
            FontError::provider(self.name(), format!("Font not found: {}", font_id))
        })?;

        Ok(format!(
            "https://fonts.bunny.net/css?family={}",
            font.family_name.replace(' ', "+")
        ))
    }

    async fn health_check(&self) -> FontResult<bool> {
        let response = self
            .client
            .head(&self.api_url)
            .send()
            .await
            .map_err(|e| FontError::network(&self.api_url, e))?;
        Ok(response.status().is_success())
    }
}
