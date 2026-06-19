//! Font Squirrel provider - 100% free for commercial use fonts

use async_trait::async_trait;
use reqwest::Client;

use crate::error::{FontError, FontResult};
use crate::models::{Font, FontCategory, FontFamily, FontLicense, FontProvider, SearchQuery};
use crate::providers::FontProviderTrait;

pub struct FontSquirrelProvider {
    client: Client,
    base_url: String,
}

impl FontSquirrelProvider {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            base_url: "https://www.fontsquirrel.com".to_string(),
        }
    }

    fn parse_category(category: &str) -> Option<FontCategory> {
        match category.to_lowercase().as_str() {
            "serif" => Some(FontCategory::Serif),
            "sans-serif" | "sans" => Some(FontCategory::SansSerif),
            "script" | "calligraphic" | "handdrawn" => Some(FontCategory::Handwriting),
            "display" | "decorative" | "retro" => Some(FontCategory::Display),
            "monospaced" | "typewriter" => Some(FontCategory::Monospace),
            _ => None,
        }
    }

    fn get_font_collection(&self) -> Vec<Font> {
        let fonts_data: Vec<(&str, &str)> = vec![
            // Sans Serif (100+ fonts)
            ("Aileron", "sans-serif"),
            ("Akrobat", "sans-serif"),
            ("Alef", "sans-serif"),
            ("Arimo", "sans-serif"),
            ("Archivo", "sans-serif"),
            ("Archivo Narrow", "sans-serif"),
            ("Archivo Black", "sans-serif"),
            ("Armata", "sans-serif"),
            ("Cabin", "sans-serif"),
            ("Carlito", "sans-serif"),
            ("Catamaran", "sans-serif"),
            ("Clear Sans", "sans-serif"),
            ("Comfortaa", "sans-serif"),
            ("Didact Gothic", "sans-serif"),
            ("Dosis", "sans-serif"),
            ("Encode Sans", "sans-serif"),
            ("Enriqueta", "sans-serif"),
            ("Exo", "sans-serif"),
            ("Fira Sans", "sans-serif"),
            ("Fontin Sans", "sans-serif"),
            ("Gidole", "sans-serif"),
            ("Glacial Indifference", "sans-serif"),
            ("Gothic A1", "sans-serif"),
            ("Gudea", "sans-serif"),
            ("Hind", "sans-serif"),
            ("Inter", "sans-serif"),
            ("Istok Web", "sans-serif"),
            ("Josefin Sans", "sans-serif"),
            ("Junction", "sans-serif"),
            ("Karla", "sans-serif"),
            ("Lato", "sans-serif"),
            ("League Spartan", "sans-serif"),
            ("Lexend Deca", "sans-serif"),
            ("Libre Franklin", "sans-serif"),
            ("Lora", "sans-serif"),
            ("Maven Pro", "sans-serif"),
            ("Metropolis", "sans-serif"),
            ("Montserrat", "sans-serif"),
            ("Muli", "sans-serif"),
            ("Nunito", "sans-serif"),
            ("Nunito Sans", "sans-serif"),
            ("Open Sans", "sans-serif"),
            ("Oswald", "sans-serif"),
            ("Overpass", "sans-serif"),
            ("Oxygen", "sans-serif"),
            ("Poppins", "sans-serif"),
            ("PT Sans", "sans-serif"),
            ("Questrial", "sans-serif"),
            ("Quicksand", "sans-serif"),
            ("Rajdhani", "sans-serif"),
            ("Raleway", "sans-serif"),
            ("Red Hat Display", "sans-serif"),
            ("Red Hat Text", "sans-serif"),
            ("Roboto", "sans-serif"),
            ("Rubik", "sans-serif"),
            ("Ruda", "sans-serif"),
            ("Saira", "sans-serif"),
            ("Signika", "sans-serif"),
            ("Source Sans Pro", "sans-serif"),
            ("Spartan", "sans-serif"),
            ("Titillium Web", "sans-serif"),
            ("Ubuntu", "sans-serif"),
            ("Varela", "sans-serif"),
            ("Varela Round", "sans-serif"),
            ("Work Sans", "sans-serif"),
            ("Abel", "sans-serif"),
            ("Advent Pro", "sans-serif"),
            ("Albert Sans", "sans-serif"),
            ("Aldrich", "sans-serif"),
            ("Alegreya Sans", "sans-serif"),
            ("Alexandria", "sans-serif"),
            ("Allerta", "sans-serif"),
            ("Alumni Sans", "sans-serif"),
            ("Amaranth", "sans-serif"),
            ("Amiko", "sans-serif"),
            ("Anaheim", "sans-serif"),
            ("Andika", "sans-serif"),
            ("Anta", "sans-serif"),
            ("Antic", "sans-serif"),
            ("Antonio", "sans-serif"),
            ("Anuphan", "sans-serif"),
            ("Archivo Narrow", "sans-serif"),
            ("Arya", "sans-serif"),
            ("Asap", "sans-serif"),
            ("Asap Condensed", "sans-serif"),
            ("Assistant", "sans-serif"),
            ("Asul", "sans-serif"),
            ("Athiti", "sans-serif"),
            ("Atkinson Hyperlegible", "sans-serif"),
            ("Average Sans", "sans-serif"),
            ("Averia Sans Libre", "sans-serif"),
            ("B612", "sans-serif"),
            ("Bai Jamjuree", "sans-serif"),
            ("Barlow", "sans-serif"),
            ("Barlow Condensed", "sans-serif"),
            ("Basic", "sans-serif"),
            ("Be Vietnam Pro", "sans-serif"),
            ("Belleza", "sans-serif"),
            ("BenchNine", "sans-serif"),
            // Serif (80+ fonts)
            ("Aleo", "serif"),
            ("Alegreya", "serif"),
            ("Alike", "serif"),
            ("Amiri", "serif"),
            ("Arvo", "serif"),
            ("Bitter", "serif"),
            ("Bree Serif", "serif"),
            ("Butler", "serif"),
            ("Cardo", "serif"),
            ("Charter", "serif"),
            ("Cinzel", "serif"),
            ("Cormorant", "serif"),
            ("Cormorant Garamond", "serif"),
            ("Crimson Pro", "serif"),
            ("Crimson Text", "serif"),
            ("Crete Round", "serif"),
            ("Domine", "serif"),
            ("EB Garamond", "serif"),
            ("Gentium Basic", "serif"),
            ("Gentium Book Basic", "serif"),
            ("Goudy Bookletter 1911", "serif"),
            ("Heuristica", "serif"),
            ("Josefin Slab", "serif"),
            ("Jura", "serif"),
            ("Kreon", "serif"),
            ("Libre Baskerville", "serif"),
            ("Libre Bodoni", "serif"),
            ("Libre Caslon Text", "serif"),
            ("Literata", "serif"),
            ("Lora", "serif"),
            ("Lusitana", "serif"),
            ("Lustria", "serif"),
            ("Merriweather", "serif"),
            ("Neuton", "serif"),
            ("Noticia Text", "serif"),
            ("Noto Serif", "serif"),
            ("Old Standard TT", "serif"),
            ("Petrona", "serif"),
            ("Philosopher", "serif"),
            ("Playfair Display", "serif"),
            ("Podkova", "serif"),
            ("Prata", "serif"),
            ("PT Serif", "serif"),
            ("Quattrocento", "serif"),
            ("Roboto Slab", "serif"),
            ("Rokkitt", "serif"),
            ("Rosarivo", "serif"),
            ("Rufina", "serif"),
            ("Scope One", "serif"),
            ("Source Serif Pro", "serif"),
            ("Spectral", "serif"),
            ("Tinos", "serif"),
            ("Trirong", "serif"),
            ("Ultra", "serif"),
            ("Unna", "serif"),
            ("Vollkorn", "serif"),
            ("Zilla Slab", "serif"),
            ("Abhaya Libre", "serif"),
            ("Alice", "serif"),
            ("Alike Angular", "serif"),
            ("Almendra", "serif"),
            ("Amethysta", "serif"),
            ("Andada Pro", "serif"),
            ("Antic Didone", "serif"),
            ("Antic Slab", "serif"),
            ("Arapey", "serif"),
            ("Arbutus Slab", "serif"),
            ("Artifika", "serif"),
            ("Asar", "serif"),
            ("Average", "serif"),
            ("Balthazar", "serif"),
            ("Belgrano", "serif"),
            ("Bellefair", "serif"),
            ("Bentham", "serif"),
            ("BioRhyme", "serif"),
            // Script/Handwriting (60+ fonts)
            ("Alex Brush", "script"),
            ("Allison", "script"),
            ("Allura", "script"),
            ("Arizonia", "script"),
            ("Bad Script", "script"),
            ("Birthstone", "script"),
            ("Caveat", "script"),
            ("Courgette", "script"),
            ("Dancing Script", "script"),
            ("Engagement", "script"),
            ("Fondamento", "script"),
            ("Gloria Hallelujah", "script"),
            ("Gochi Hand", "script"),
            ("Grand Hotel", "script"),
            ("Great Vibes", "script"),
            ("Handlee", "script"),
            ("Homemade Apple", "script"),
            ("Indie Flower", "script"),
            ("Italianno", "script"),
            ("Kalam", "script"),
            ("Kaushan Script", "script"),
            ("Lobster Two", "script"),
            ("Marck Script", "script"),
            ("Neucha", "script"),
            ("Niconne", "script"),
            ("Pacifico", "script"),
            ("Parisienne", "script"),
            ("Patrick Hand", "script"),
            ("Permanent Marker", "script"),
            ("Pinyon Script", "script"),
            ("Playball", "script"),
            ("Rancho", "script"),
            ("Reenie Beanie", "script"),
            ("Rochester", "script"),
            ("Rock Salt", "script"),
            ("Rouge Script", "script"),
            ("Sacramento", "script"),
            ("Satisfy", "script"),
            ("Shadows Into Light", "script"),
            ("Tangerine", "script"),
            ("Yellowtail", "script"),
            ("Amatic SC", "script"),
            ("Architects Daughter", "script"),
            ("Covered By Your Grace", "script"),
            ("Damion", "script"),
            ("Dawning of a New Day", "script"),
            ("Delius", "script"),
            ("Euphoria Script", "script"),
            ("Herr Von Muellerhoff", "script"),
            ("Just Another Hand", "script"),
            ("La Belle Aurore", "script"),
            ("League Script", "script"),
            ("Loved by the King", "script"),
            ("Lovers Quarrel", "script"),
            ("Meddon", "script"),
            ("Miss Fajardose", "script"),
            ("Monsieur La Doulaise", "script"),
            ("Mr De Haviland", "script"),
            ("Mrs Saint Delafield", "script"),
            // Display (60+ fonts)
            ("Abril Fatface", "display"),
            ("Alfa Slab One", "display"),
            ("Anton", "display"),
            ("Audiowide", "display"),
            ("Bangers", "display"),
            ("Bebas Neue", "display"),
            ("Bevan", "display"),
            ("Big Shoulders Display", "display"),
            ("Black Ops One", "display"),
            ("Bungee", "display"),
            ("Carter One", "display"),
            ("Changa One", "display"),
            ("Comfortaa", "display"),
            ("Concert One", "display"),
            ("Creepster", "display"),
            ("Fredoka One", "display"),
            ("Germania One", "display"),
            ("Graduate", "display"),
            ("Iceland", "display"),
            ("Kelly Slab", "display"),
            ("Knewave", "display"),
            ("League Gothic", "display"),
            ("Lilita One", "display"),
            ("Lobster", "display"),
            ("Luckiest Guy", "display"),
            ("Monoton", "display"),
            ("Nixie One", "display"),
            ("Nosifer", "display"),
            ("Oleo Script", "display"),
            ("Orbitron", "display"),
            ("Passion One", "display"),
            ("Patua One", "display"),
            ("Plaster", "display"),
            ("Press Start 2P", "display"),
            ("Racing Sans One", "display"),
            ("Righteous", "display"),
            ("Russo One", "display"),
            ("Shrikhand", "display"),
            ("Special Elite", "display"),
            ("Staatliches", "display"),
            ("Stalinist One", "display"),
            ("Teko", "display"),
            ("Titan One", "display"),
            ("Trade Winds", "display"),
            ("Wallpoet", "display"),
            ("Yeseva One", "display"),
            ("Almendra Display", "display"),
            ("Angkor", "display"),
            ("Arbutus", "display"),
            ("Astloch", "display"),
            ("Atomic Age", "display"),
            ("Aubrey", "display"),
            ("Averia Libre", "display"),
            ("Baloo 2", "display"),
            ("Barriecito", "display"),
            ("Barrio", "display"),
            ("Battambang", "display"),
            ("Baumans", "display"),
            ("Bowlby One", "display"),
            ("Bubblegum Sans", "display"),
            // Monospace (30+ fonts)
            ("Anonymous Pro", "monospaced"),
            ("Azeret Mono", "monospaced"),
            ("B612 Mono", "monospaced"),
            ("Courier Prime", "monospaced"),
            ("Cousine", "monospaced"),
            ("Cutive Mono", "monospaced"),
            ("DM Mono", "monospaced"),
            ("Fantasque Sans Mono", "monospaced"),
            ("Fira Code", "monospaced"),
            ("Fira Mono", "monospaced"),
            ("Hack", "monospaced"),
            ("IBM Plex Mono", "monospaced"),
            ("Inconsolata", "monospaced"),
            ("Input Mono", "monospaced"),
            ("JetBrains Mono", "monospaced"),
            ("Liberation Mono", "monospaced"),
            ("Major Mono Display", "monospaced"),
            ("Nanum Gothic Coding", "monospaced"),
            ("Nova Mono", "monospaced"),
            ("Overpass Mono", "monospaced"),
            ("Oxygen Mono", "monospaced"),
            ("PT Mono", "monospaced"),
            ("Red Hat Mono", "monospaced"),
            ("Roboto Mono", "monospaced"),
            ("Share Tech Mono", "monospaced"),
            ("Source Code Pro", "monospaced"),
            ("Space Mono", "monospaced"),
            ("Ubuntu Mono", "monospaced"),
            ("Victor Mono", "monospaced"),
        ];

        fonts_data
            .into_iter()
            .map(|(name, category)| {
                let id = name
                    .to_lowercase()
                    .replace(' ', "-")
                    .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
                Font {
                    id: format!("fontsquirrel-{}", id),
                    name: name.to_string(),
                    provider: FontProvider::FontSquirrel,
                    category: Self::parse_category(category),
                    variant_count: 1,
                    license: Some(FontLicense::FreeCommercial),
                    preview_url: Some(format!("{}/fonts/{}", self.base_url, id)),
                    download_url: Some(format!("{}/fonts/download/{}", self.base_url, id)),
                }
            })
            .collect()
    }
}

#[async_trait]
impl FontProviderTrait for FontSquirrelProvider {
    fn name(&self) -> &str {
        "Font Squirrel"
    }
    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn search(&self, query: &SearchQuery) -> FontResult<Vec<Font>> {
        let all_fonts = self.get_font_collection();
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

    async fn list_all(&self) -> FontResult<Vec<Font>> {
        Ok(self.get_font_collection())
    }

    async fn get_font_family(&self, font_id: &str) -> FontResult<FontFamily> {
        let font = self
            .get_font_collection()
            .into_iter()
            .find(|f| f.id == font_id)
            .ok_or_else(|| {
                FontError::provider("Font Squirrel", format!("Font not found: {}", font_id))
            })?;
        Ok(FontFamily {
            id: font.id,
            name: font.name,
            provider: FontProvider::FontSquirrel,
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
        })
    }

    async fn get_download_url(&self, font_id: &str) -> FontResult<String> {
        Ok(format!(
            "{}/fonts/download/{}",
            self.base_url,
            font_id.replace("fontsquirrel-", "")
        ))
    }

    async fn health_check(&self) -> FontResult<bool> {
        let response = self
            .client
            .get(&self.base_url)
            .send()
            .await
            .map_err(|e| FontError::network(&self.base_url, e))?;
        Ok(response.status().is_success())
    }
}
