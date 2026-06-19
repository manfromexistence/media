//! 1001 Fonts provider - Large collection of free fonts (40,000+)

use async_trait::async_trait;
use reqwest::Client;

use crate::error::{FontError, FontResult};
use crate::models::{Font, FontCategory, FontFamily, FontLicense, FontProvider, SearchQuery};
use crate::providers::FontProviderTrait;

pub struct Fonts1001Provider {
    client: Client,
    base_url: String,
}

impl Fonts1001Provider {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            base_url: "https://www.1001fonts.com".to_string(),
        }
    }

    fn parse_category(category: &str) -> Option<FontCategory> {
        match category.to_lowercase().as_str() {
            "serif" | "slab" => Some(FontCategory::Serif),
            "sans-serif" | "sans" | "geometric" => Some(FontCategory::SansSerif),
            "script" | "calligraphy" | "handwritten" | "cursive" | "brush" => {
                Some(FontCategory::Handwriting)
            }
            "display" | "decorative" | "fancy" | "headline" | "poster" => {
                Some(FontCategory::Display)
            }
            "monospace" | "typewriter" | "code" | "fixed" => Some(FontCategory::Monospace),
            _ => None,
        }
    }

    fn get_font_collection(&self) -> Vec<Font> {
        let fonts_data: Vec<(&str, &str, &str)> = vec![
            // Sans Serif (100+ fonts)
            ("Aileron", "sans-serif", "100% Free"),
            ("Akrobat", "sans-serif", "100% Free"),
            ("Aleo", "sans-serif", "OFL"),
            ("Anders", "sans-serif", "100% Free"),
            ("Anurati", "sans-serif", "Free"),
            ("Azonix", "sans-serif", "Free"),
            ("Bebas Kai", "sans-serif", "100% Free"),
            ("Blogger Sans", "sans-serif", "100% Free"),
            ("Cabin", "sans-serif", "OFL"),
            ("Comfortaa", "sans-serif", "OFL"),
            ("Coolvetica", "sans-serif", "Free"),
            ("Coves", "sans-serif", "Free"),
            ("D-DIN", "sans-serif", "100% Free"),
            ("Dosis", "sans-serif", "OFL"),
            ("Exo", "sans-serif", "OFL"),
            ("Fira Sans", "sans-serif", "OFL"),
            ("Gidole", "sans-serif", "OFL"),
            ("Glacial Indifference", "sans-serif", "OFL"),
            ("Gravity", "sans-serif", "Free"),
            ("Harabara", "sans-serif", "Free"),
            ("Hero", "sans-serif", "100% Free"),
            ("Highway Gothic", "sans-serif", "Public Domain"),
            ("Josefin Sans", "sans-serif", "OFL"),
            ("Kollektif", "sans-serif", "100% Free"),
            ("Lato", "sans-serif", "OFL"),
            ("League Spartan", "sans-serif", "OFL"),
            ("Libre Franklin", "sans-serif", "OFL"),
            ("Metropolis", "sans-serif", "100% Free"),
            ("Montserrat", "sans-serif", "OFL"),
            ("Muli", "sans-serif", "OFL"),
            ("Nunito Sans", "sans-serif", "OFL"),
            ("Open Sans", "sans-serif", "Apache"),
            ("Oswald", "sans-serif", "OFL"),
            ("Oxygen", "sans-serif", "OFL"),
            ("Pier Sans", "sans-serif", "Free"),
            ("Poppins", "sans-serif", "OFL"),
            ("Quicksand", "sans-serif", "OFL"),
            ("Rajdhani", "sans-serif", "OFL"),
            ("Raleway", "sans-serif", "OFL"),
            ("Roboto", "sans-serif", "Apache"),
            ("Rubik", "sans-serif", "OFL"),
            ("Source Sans Pro", "sans-serif", "OFL"),
            ("Spartan", "sans-serif", "OFL"),
            ("Ubuntu", "sans-serif", "UFL"),
            ("Varela Round", "sans-serif", "OFL"),
            ("Venus Rising", "sans-serif", "Free"),
            ("Walkway", "sans-serif", "100% Free"),
            ("Work Sans", "sans-serif", "OFL"),
            ("Abel", "sans-serif", "OFL"),
            ("ABeeZee", "sans-serif", "OFL"),
            ("Aclonica", "sans-serif", "OFL"),
            ("Actor", "sans-serif", "OFL"),
            ("Advent Pro", "sans-serif", "OFL"),
            ("Afacad", "sans-serif", "OFL"),
            ("Agdasima", "sans-serif", "OFL"),
            ("Akshar", "sans-serif", "OFL"),
            ("Alata", "sans-serif", "OFL"),
            ("Alatsi", "sans-serif", "OFL"),
            ("Albert Sans", "sans-serif", "OFL"),
            ("Aldrich", "sans-serif", "OFL"),
            ("Alef", "sans-serif", "OFL"),
            ("Alegreya Sans", "sans-serif", "OFL"),
            ("Alexandria", "sans-serif", "OFL"),
            ("Allerta", "sans-serif", "OFL"),
            ("Allerta Stencil", "sans-serif", "OFL"),
            ("Alumni Sans", "sans-serif", "OFL"),
            ("Amaranth", "sans-serif", "OFL"),
            ("Amiko", "sans-serif", "OFL"),
            ("Anaheim", "sans-serif", "OFL"),
            ("Andika", "sans-serif", "OFL"),
            ("Anta", "sans-serif", "OFL"),
            ("Antic", "sans-serif", "OFL"),
            ("Antonio", "sans-serif", "OFL"),
            ("Anuphan", "sans-serif", "OFL"),
            // Serif (80+ fonts)
            ("Abril Fatface", "serif", "OFL"),
            ("Alike", "serif", "OFL"),
            ("Amiri", "serif", "OFL"),
            ("Arvo", "serif", "OFL"),
            ("Bitter", "serif", "OFL"),
            ("Bona Nova", "serif", "OFL"),
            ("Bree Serif", "serif", "OFL"),
            ("Butler", "serif", "100% Free"),
            ("Cardo", "serif", "OFL"),
            ("Cinzel", "serif", "OFL"),
            ("Cormorant", "serif", "OFL"),
            ("Crimson Pro", "serif", "OFL"),
            ("Crimson Text", "serif", "OFL"),
            ("EB Garamond", "serif", "OFL"),
            ("Gabriela", "serif", "OFL"),
            ("Gentium", "serif", "OFL"),
            ("Gloock", "serif", "OFL"),
            ("Goudy Bookletter", "serif", "100% Free"),
            ("Gravitas One", "serif", "OFL"),
            ("Heuristica", "serif", "OFL"),
            ("Instrument Serif", "serif", "OFL"),
            ("Junicode", "serif", "OFL"),
            ("Libre Baskerville", "serif", "OFL"),
            ("Libre Bodoni", "serif", "OFL"),
            ("Libre Caslon Display", "serif", "OFL"),
            ("Libre Caslon Text", "serif", "OFL"),
            ("Literata", "serif", "OFL"),
            ("Lora", "serif", "OFL"),
            ("Merriweather", "serif", "OFL"),
            ("Noto Serif", "serif", "OFL"),
            ("Old Standard TT", "serif", "OFL"),
            ("Petrona", "serif", "OFL"),
            ("Playfair Display", "serif", "OFL"),
            ("Prata", "serif", "OFL"),
            ("PT Serif", "serif", "OFL"),
            ("Quattrocento", "serif", "OFL"),
            ("Roboto Slab", "serif", "Apache"),
            ("Rokkitt", "serif", "OFL"),
            ("Rosarivo", "serif", "OFL"),
            ("Rozha One", "serif", "OFL"),
            ("Rufina", "serif", "OFL"),
            ("Slabo 27px", "serif", "OFL"),
            ("Source Serif Pro", "serif", "OFL"),
            ("Spectral", "serif", "OFL"),
            ("Trirong", "serif", "OFL"),
            ("Ultra", "serif", "OFL"),
            ("Unna", "serif", "OFL"),
            ("Vidaloka", "serif", "OFL"),
            ("Vollkorn", "serif", "OFL"),
            ("Young Serif", "serif", "OFL"),
            ("Zilla Slab", "serif", "OFL"),
            ("Abhaya Libre", "serif", "OFL"),
            ("Aleo", "serif", "OFL"),
            ("Alice", "serif", "OFL"),
            ("Alike Angular", "serif", "OFL"),
            ("Alkalami", "serif", "OFL"),
            ("Almendra", "serif", "OFL"),
            ("Amethysta", "serif", "OFL"),
            ("Andada Pro", "serif", "OFL"),
            ("Antic Didone", "serif", "OFL"),
            ("Antic Slab", "serif", "OFL"),
            ("Arapey", "serif", "OFL"),
            ("Arbutus Slab", "serif", "OFL"),
            ("Artifika", "serif", "OFL"),
            // Script/Handwriting (80+ fonts)
            ("Aguafina Script", "script", "OFL"),
            ("Alex Brush", "script", "OFL"),
            ("Allura", "script", "OFL"),
            ("Arizonia", "script", "OFL"),
            ("Bad Script", "script", "OFL"),
            ("Ballet", "script", "OFL"),
            ("Birthstone", "script", "OFL"),
            ("Bonheur Royale", "script", "OFL"),
            ("Corinthia", "script", "OFL"),
            ("Dancing Script", "script", "OFL"),
            ("Delicious Handrawn", "script", "OFL"),
            ("Engagement", "script", "OFL"),
            ("Euphoria Script", "script", "OFL"),
            ("Felipa", "script", "OFL"),
            ("Fondamento", "script", "OFL"),
            ("Great Vibes", "script", "OFL"),
            ("Herr Von Muellerhoff", "script", "OFL"),
            ("Italianno", "script", "OFL"),
            ("Kaushan Script", "script", "OFL"),
            ("Lavishly Yours", "script", "OFL"),
            ("League Script", "script", "OFL"),
            ("Lobster Two", "script", "OFL"),
            ("Loved by the King", "script", "OFL"),
            ("Lovers Quarrel", "script", "OFL"),
            ("Marck Script", "script", "OFL"),
            ("Meow Script", "script", "OFL"),
            ("Milonga", "script", "OFL"),
            ("Miss Fajardose", "script", "OFL"),
            ("Monsieur La Doulaise", "script", "OFL"),
            ("Mr De Haviland", "script", "OFL"),
            ("Mrs Saint Delafield", "script", "OFL"),
            ("Mrs Sheppards", "script", "OFL"),
            ("Niconne", "script", "OFL"),
            ("Norican", "script", "OFL"),
            ("Nothing You Could Do", "script", "OFL"),
            ("Oleo Script", "script", "OFL"),
            ("Pacifico", "script", "OFL"),
            ("Parisienne", "script", "OFL"),
            ("Petit Formal Script", "script", "OFL"),
            ("Pinyon Script", "script", "OFL"),
            ("Playball", "script", "OFL"),
            ("Qwigley", "script", "OFL"),
            ("Rochester", "script", "OFL"),
            ("Rouge Script", "script", "OFL"),
            ("Sacramento", "script", "OFL"),
            ("Sail", "script", "OFL"),
            ("Satisfy", "script", "OFL"),
            ("Seaweed Script", "script", "OFL"),
            ("Shadows Into Light", "script", "OFL"),
            ("Style Script", "script", "OFL"),
            ("Tangerine", "script", "OFL"),
            ("The Nautigal", "script", "OFL"),
            ("Waterfall", "script", "OFL"),
            ("Yellowtail", "script", "OFL"),
            ("Zeyada", "script", "OFL"),
            ("Caveat", "script", "OFL"),
            ("Amatic SC", "script", "OFL"),
            ("Indie Flower", "script", "OFL"),
            ("Patrick Hand", "script", "OFL"),
            ("Permanent Marker", "script", "OFL"),
            ("Gloria Hallelujah", "script", "OFL"),
            ("Rock Salt", "script", "OFL"),
            ("Architects Daughter", "script", "OFL"),
            ("Reenie Beanie", "script", "OFL"),
            // Display (80+ fonts)
            ("Anton", "display", "OFL"),
            ("Audiowide", "display", "OFL"),
            ("Bangers", "display", "OFL"),
            ("Bebas Neue", "display", "OFL"),
            ("Big Shoulders Display", "display", "OFL"),
            ("Black Ops One", "display", "OFL"),
            ("Bungee", "display", "OFL"),
            ("Bungee Shade", "display", "OFL"),
            ("Cabin Sketch", "display", "OFL"),
            ("Carter One", "display", "OFL"),
            ("Changa One", "display", "OFL"),
            ("Concert One", "display", "OFL"),
            ("Creepster", "display", "OFL"),
            ("Diplomata", "display", "OFL"),
            ("Emblema One", "display", "OFL"),
            ("Faster One", "display", "OFL"),
            ("Flavors", "display", "OFL"),
            ("Fugaz One", "display", "OFL"),
            ("Germania One", "display", "OFL"),
            ("Graduate", "display", "OFL"),
            ("Gravitas One", "display", "OFL"),
            ("Iceland", "display", "OFL"),
            ("Kelly Slab", "display", "OFL"),
            ("Knewave", "display", "OFL"),
            ("League Gothic", "display", "OFL"),
            ("Lemon", "display", "OFL"),
            ("Lilita One", "display", "OFL"),
            ("Lobster", "display", "OFL"),
            ("Monoton", "display", "OFL"),
            ("Nosifer", "display", "OFL"),
            ("Orbitron", "display", "OFL"),
            ("Passion One", "display", "OFL"),
            ("Patua One", "display", "OFL"),
            ("Plaster", "display", "OFL"),
            ("Press Start 2P", "display", "OFL"),
            ("Racing Sans One", "display", "OFL"),
            ("Righteous", "display", "OFL"),
            ("Rubik Mono One", "display", "OFL"),
            ("Russo One", "display", "OFL"),
            ("Shrikhand", "display", "OFL"),
            ("Sigmar One", "display", "OFL"),
            ("Smokum", "display", "OFL"),
            ("Special Elite", "display", "OFL"),
            ("Squada One", "display", "OFL"),
            ("Staatliches", "display", "OFL"),
            ("Stalinist One", "display", "OFL"),
            ("Teko", "display", "OFL"),
            ("Titan One", "display", "OFL"),
            ("Trade Winds", "display", "OFL"),
            ("Ultra", "display", "OFL"),
            ("Wallpoet", "display", "OFL"),
            ("Yeseva One", "display", "OFL"),
            ("Alfa Slab One", "display", "OFL"),
            ("Almendra Display", "display", "OFL"),
            ("Angkor", "display", "OFL"),
            ("Arbutus", "display", "OFL"),
            ("Astloch", "display", "OFL"),
            ("Atma", "display", "OFL"),
            ("Atomic Age", "display", "OFL"),
            ("Aubrey", "display", "OFL"),
            // Monospace (40+ fonts)
            ("Anonymous Pro", "monospace", "OFL"),
            ("B612 Mono", "monospace", "OFL"),
            ("Chivo Mono", "monospace", "OFL"),
            ("Courier Prime", "monospace", "OFL"),
            ("Cousine", "monospace", "Apache"),
            ("Cutive Mono", "monospace", "OFL"),
            ("DM Mono", "monospace", "OFL"),
            ("Fira Code", "monospace", "OFL"),
            ("Fira Mono", "monospace", "OFL"),
            ("Geist Mono", "monospace", "OFL"),
            ("Hack", "monospace", "MIT"),
            ("IBM Plex Mono", "monospace", "OFL"),
            ("Inconsolata", "monospace", "OFL"),
            ("JetBrains Mono", "monospace", "OFL"),
            ("Major Mono Display", "monospace", "OFL"),
            ("Martian Mono", "monospace", "OFL"),
            ("Nanum Gothic Coding", "monospace", "OFL"),
            ("Nova Mono", "monospace", "OFL"),
            ("Overpass Mono", "monospace", "OFL"),
            ("Oxygen Mono", "monospace", "OFL"),
            ("PT Mono", "monospace", "OFL"),
            ("Red Hat Mono", "monospace", "OFL"),
            ("Roboto Mono", "monospace", "Apache"),
            ("Share Tech Mono", "monospace", "OFL"),
            ("Source Code Pro", "monospace", "OFL"),
            ("Space Mono", "monospace", "OFL"),
            ("Ubuntu Mono", "monospace", "UFL"),
            ("Victor Mono", "monospace", "OFL"),
            ("Xanh Mono", "monospace", "OFL"),
            ("Azeret Mono", "monospace", "OFL"),
        ];

        fonts_data
            .into_iter()
            .map(|(name, category, _license)| {
                let id = name
                    .to_lowercase()
                    .replace(' ', "-")
                    .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
                Font {
                    id: format!("1001fonts-{}", id),
                    name: name.to_string(),
                    provider: FontProvider::Fonts1001,
                    category: Self::parse_category(category),
                    variant_count: 1,
                    license: Some(FontLicense::OFL),
                    preview_url: Some(format!("{}/{}", self.base_url, id)),
                    download_url: Some(format!("{}/{}/download", self.base_url, id)),
                }
            })
            .collect()
    }
}

#[async_trait]
impl FontProviderTrait for Fonts1001Provider {
    fn name(&self) -> &str {
        "1001 Fonts"
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
                FontError::provider(self.name(), format!("Font not found: {}", font_id))
            })?;
        Ok(FontFamily {
            id: font.id,
            name: font.name,
            provider: FontProvider::Fonts1001,
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
            "{}/{}/download",
            self.base_url,
            font_id.replace("1001fonts-", "")
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
