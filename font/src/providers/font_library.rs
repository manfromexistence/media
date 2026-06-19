//! Font Library provider implementation
//!
//! Open Font Library provides 650+ open-source fonts.
//! API: <https://fontlibrary.org/en/catalogue>

use super::FontProviderTrait;
use crate::error::{FontError, FontResult};
use crate::models::{
    Font, FontCategory, FontFamily, FontLicense, FontProvider, FontStyle, FontVariant, FontWeight,
    SearchQuery,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

/// Font Library API font response
#[derive(Debug, Deserialize)]
pub struct FontLibraryFont {
    pub id: String,
    #[serde(rename = "family_name")]
    pub family_name: String,
    pub category: Option<String>,
    pub designer: Option<String>,
    pub license: Option<String>,
    pub styles: Option<Vec<FontLibraryStyle>>,
}

#[derive(Debug, Deserialize)]
pub struct FontLibraryStyle {
    pub style: Option<String>,
    pub weight: Option<String>,
}

/// Font Library provider
pub struct FontLibraryProvider {
    client: Client,
}

impl FontLibraryProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl FontProviderTrait for FontLibraryProvider {
    fn name(&self) -> &str {
        "Font Library"
    }

    fn base_url(&self) -> &str {
        "https://fontlibrary.org"
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
        // Font Library doesn't have a public JSON API, so we'll use pre-defined popular fonts
        // In production, this would scrape the catalogue or use their API if available
        let popular_fonts = get_font_library_fonts();

        let fonts: Vec<Font> = popular_fonts
            .into_iter()
            .map(|(id, name, category)| Font {
                id: id.to_string(),
                name: name.to_string(),
                provider: FontProvider::FontLibrary,
                category: Some(category),
                variant_count: 4,
                license: Some(FontLicense::OFL),
                preview_url: Some(format!("https://fontlibrary.org/en/font/{}", id)),
                download_url: Some(format!(
                    "https://fontlibrary.org/assets/downloads/{}/{}-Regular.ttf",
                    id,
                    name.replace(' ', "")
                )),
            })
            .collect();

        Ok(fonts)
    }

    async fn get_font_family(&self, font_id: &str) -> FontResult<FontFamily> {
        let fonts = self.list_all().await?;
        let font = fonts.into_iter().find(|f| f.id == font_id).ok_or_else(|| {
            FontError::provider(self.name(), format!("Font not found: {}", font_id))
        })?;

        Ok(FontFamily {
            id: font.id,
            name: font.name.clone(),
            provider: FontProvider::FontLibrary,
            category: font.category,
            variants: vec![FontVariant {
                weight: FontWeight::Regular,
                style: FontStyle::Normal,
                file_url: font.download_url.clone(),
                file_format: "ttf".to_string(),
            }],
            license: Some(FontLicense::OFL),
            designer: None,
            description: None,
            preview_url: font.preview_url,
            download_url: font.download_url,
            languages: vec!["Latin".to_string()],
            subsets: vec!["latin".to_string()],
            popularity: None,
            last_modified: None,
        })
    }

    async fn get_download_url(&self, font_id: &str) -> FontResult<String> {
        Ok(format!(
            "https://fontlibrary.org/assets/downloads/{}/",
            font_id
        ))
    }

    async fn health_check(&self) -> FontResult<bool> {
        let response = self
            .client
            .head("https://fontlibrary.org")
            .send()
            .await
            .map_err(|e| FontError::network("https://fontlibrary.org", e))?;
        Ok(response.status().is_success())
    }
}

/// Get pre-defined Font Library fonts
fn get_font_library_fonts() -> Vec<(&'static str, &'static str, FontCategory)> {
    vec![
        ("linux-libertine", "Linux Libertine", FontCategory::Serif),
        ("linux-biolinum", "Linux Biolinum", FontCategory::SansSerif),
        ("dejavu-sans", "DejaVu Sans", FontCategory::SansSerif),
        ("dejavu-serif", "DejaVu Serif", FontCategory::Serif),
        (
            "dejavu-sans-mono",
            "DejaVu Sans Mono",
            FontCategory::Monospace,
        ),
        ("ubuntu", "Ubuntu", FontCategory::SansSerif),
        ("ubuntu-mono", "Ubuntu Mono", FontCategory::Monospace),
        ("cantarell", "Cantarell", FontCategory::SansSerif),
        ("droid-sans", "Droid Sans", FontCategory::SansSerif),
        ("droid-serif", "Droid Serif", FontCategory::Serif),
        ("gentium-basic", "Gentium Basic", FontCategory::Serif),
        ("inconsolata", "Inconsolata", FontCategory::Monospace),
        (
            "liberation-sans",
            "Liberation Sans",
            FontCategory::SansSerif,
        ),
        ("liberation-serif", "Liberation Serif", FontCategory::Serif),
        (
            "liberation-mono",
            "Liberation Mono",
            FontCategory::Monospace,
        ),
        ("open-sans", "Open Sans", FontCategory::SansSerif),
        (
            "source-sans-pro",
            "Source Sans Pro",
            FontCategory::SansSerif,
        ),
        ("source-serif-pro", "Source Serif Pro", FontCategory::Serif),
        (
            "source-code-pro",
            "Source Code Pro",
            FontCategory::Monospace,
        ),
        ("noto-sans", "Noto Sans", FontCategory::SansSerif),
        ("noto-serif", "Noto Serif", FontCategory::Serif),
        ("crimson-text", "Crimson Text", FontCategory::Serif),
        ("eb-garamond", "EB Garamond", FontCategory::Serif),
        ("lato", "Lato", FontCategory::SansSerif),
        ("merriweather", "Merriweather", FontCategory::Serif),
        ("montserrat", "Montserrat", FontCategory::SansSerif),
        ("oswald", "Oswald", FontCategory::SansSerif),
        ("playfair-display", "Playfair Display", FontCategory::Serif),
        ("poppins", "Poppins", FontCategory::SansSerif),
        ("raleway", "Raleway", FontCategory::SansSerif),
        ("roboto", "Roboto", FontCategory::SansSerif),
        ("quicksand", "Quicksand", FontCategory::SansSerif),
        ("nunito", "Nunito", FontCategory::SansSerif),
        ("oxygen", "Oxygen", FontCategory::SansSerif),
        ("karla", "Karla", FontCategory::SansSerif),
        ("work-sans", "Work Sans", FontCategory::SansSerif),
        ("josefin-sans", "Josefin Sans", FontCategory::SansSerif),
        ("rubik", "Rubik", FontCategory::SansSerif),
        ("cabin", "Cabin", FontCategory::SansSerif),
        ("bitter", "Bitter", FontCategory::Serif),
        ("arvo", "Arvo", FontCategory::Serif),
        ("vollkorn", "Vollkorn", FontCategory::Serif),
        (
            "libre-baskerville",
            "Libre Baskerville",
            FontCategory::Serif,
        ),
        ("fira-sans", "Fira Sans", FontCategory::SansSerif),
        ("fira-mono", "Fira Mono", FontCategory::Monospace),
        ("pt-sans", "PT Sans", FontCategory::SansSerif),
        ("pt-serif", "PT Serif", FontCategory::Serif),
        ("pt-mono", "PT Mono", FontCategory::Monospace),
        ("exo", "Exo", FontCategory::SansSerif),
        ("exo-2", "Exo 2", FontCategory::SansSerif),
    ]
}
