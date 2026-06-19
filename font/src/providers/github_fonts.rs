//! GitHub Fonts provider implementation
//!
//! Direct access to fonts from major GitHub repositories.
//! Includes Adobe Fonts, Noto Fonts, popular coding fonts, etc.

use super::FontProviderTrait;
use crate::error::{FontError, FontResult};
use crate::models::{
    Font, FontCategory, FontFamily, FontLicense, FontProvider, FontStyle, FontVariant, FontWeight,
    SearchQuery,
};
use async_trait::async_trait;
use reqwest::Client;

/// GitHub Fonts provider
pub struct GitHubFontsProvider {
    client: Client,
}

impl GitHubFontsProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl FontProviderTrait for GitHubFontsProvider {
    fn name(&self) -> &str {
        "GitHub Fonts"
    }

    fn base_url(&self) -> &str {
        "https://github.com"
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
        let github_fonts = get_github_fonts();

        let fonts: Vec<Font> = github_fonts
            .into_iter()
            .map(|(id, name, category, repo, license)| Font {
                id: id.to_string(),
                name: name.to_string(),
                provider: FontProvider::GitHub,
                category: Some(category),
                variant_count: 4,
                license: Some(license),
                preview_url: Some(format!("https://github.com/{}", repo)),
                download_url: Some(format!("https://github.com/{}/releases/latest", repo)),
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
            provider: FontProvider::GitHub,
            category: font.category,
            variants: vec![FontVariant {
                weight: FontWeight::Regular,
                style: FontStyle::Normal,
                file_url: font.download_url.clone(),
                file_format: "ttf".to_string(),
            }],
            license: font.license,
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
        let fonts = self.list_all().await?;
        let font = fonts.into_iter().find(|f| f.id == font_id).ok_or_else(|| {
            FontError::provider(self.name(), format!("Font not found: {}", font_id))
        })?;

        font.download_url.ok_or_else(|| {
            FontError::provider(
                self.name(),
                format!("No download URL for font: {}", font_id),
            )
        })
    }

    async fn health_check(&self) -> FontResult<bool> {
        let response = self
            .client
            .head("https://github.com")
            .send()
            .await
            .map_err(|e| FontError::network("https://github.com", e))?;
        Ok(response.status().is_success())
    }
}

/// Get pre-defined GitHub fonts from major repositories
fn get_github_fonts() -> Vec<(
    &'static str,
    &'static str,
    FontCategory,
    &'static str,
    FontLicense,
)> {
    vec![
        // Adobe Fonts
        (
            "source-sans-3",
            "Source Sans 3",
            FontCategory::SansSerif,
            "adobe-fonts/source-sans",
            FontLicense::OFL,
        ),
        (
            "source-serif-4",
            "Source Serif 4",
            FontCategory::Serif,
            "adobe-fonts/source-serif",
            FontLicense::OFL,
        ),
        (
            "source-code-pro",
            "Source Code Pro",
            FontCategory::Monospace,
            "adobe-fonts/source-code-pro",
            FontLicense::OFL,
        ),
        (
            "source-han-sans",
            "Source Han Sans",
            FontCategory::SansSerif,
            "adobe-fonts/source-han-sans",
            FontLicense::OFL,
        ),
        (
            "source-han-serif",
            "Source Han Serif",
            FontCategory::Serif,
            "adobe-fonts/source-han-serif",
            FontLicense::OFL,
        ),
        (
            "source-han-mono",
            "Source Han Mono",
            FontCategory::Monospace,
            "adobe-fonts/source-han-mono",
            FontLicense::OFL,
        ),
        // IBM
        (
            "ibm-plex-sans",
            "IBM Plex Sans",
            FontCategory::SansSerif,
            "IBM/plex",
            FontLicense::OFL,
        ),
        (
            "ibm-plex-serif",
            "IBM Plex Serif",
            FontCategory::Serif,
            "IBM/plex",
            FontLicense::OFL,
        ),
        (
            "ibm-plex-mono",
            "IBM Plex Mono",
            FontCategory::Monospace,
            "IBM/plex",
            FontLicense::OFL,
        ),
        (
            "ibm-plex-sans-arabic",
            "IBM Plex Sans Arabic",
            FontCategory::SansSerif,
            "IBM/plex",
            FontLicense::OFL,
        ),
        (
            "ibm-plex-sans-devanagari",
            "IBM Plex Sans Devanagari",
            FontCategory::SansSerif,
            "IBM/plex",
            FontLicense::OFL,
        ),
        (
            "ibm-plex-sans-hebrew",
            "IBM Plex Sans Hebrew",
            FontCategory::SansSerif,
            "IBM/plex",
            FontLicense::OFL,
        ),
        (
            "ibm-plex-sans-jp",
            "IBM Plex Sans JP",
            FontCategory::SansSerif,
            "IBM/plex",
            FontLicense::OFL,
        ),
        (
            "ibm-plex-sans-kr",
            "IBM Plex Sans KR",
            FontCategory::SansSerif,
            "IBM/plex",
            FontLicense::OFL,
        ),
        (
            "ibm-plex-sans-thai",
            "IBM Plex Sans Thai",
            FontCategory::SansSerif,
            "IBM/plex",
            FontLicense::OFL,
        ),
        // Mozilla
        (
            "fira-sans",
            "Fira Sans",
            FontCategory::SansSerif,
            "mozilla/Fira",
            FontLicense::OFL,
        ),
        (
            "fira-mono",
            "Fira Mono",
            FontCategory::Monospace,
            "mozilla/Fira",
            FontLicense::OFL,
        ),
        (
            "fira-code",
            "Fira Code",
            FontCategory::Monospace,
            "tonsky/FiraCode",
            FontLicense::OFL,
        ),
        // JetBrains
        (
            "jetbrains-mono",
            "JetBrains Mono",
            FontCategory::Monospace,
            "JetBrains/JetBrainsMono",
            FontLicense::OFL,
        ),
        // Microsoft
        (
            "cascadia-code",
            "Cascadia Code",
            FontCategory::Monospace,
            "microsoft/cascadia-code",
            FontLicense::OFL,
        ),
        (
            "cascadia-mono",
            "Cascadia Mono",
            FontCategory::Monospace,
            "microsoft/cascadia-code",
            FontLicense::OFL,
        ),
        // GitHub
        (
            "monaspace-neon",
            "Monaspace Neon",
            FontCategory::Monospace,
            "githubnext/monaspace",
            FontLicense::OFL,
        ),
        (
            "monaspace-argon",
            "Monaspace Argon",
            FontCategory::Monospace,
            "githubnext/monaspace",
            FontLicense::OFL,
        ),
        (
            "monaspace-xenon",
            "Monaspace Xenon",
            FontCategory::Monospace,
            "githubnext/monaspace",
            FontLicense::OFL,
        ),
        (
            "monaspace-radon",
            "Monaspace Radon",
            FontCategory::Monospace,
            "githubnext/monaspace",
            FontLicense::OFL,
        ),
        (
            "monaspace-krypton",
            "Monaspace Krypton",
            FontCategory::Monospace,
            "githubnext/monaspace",
            FontLicense::OFL,
        ),
        // Inter
        (
            "inter",
            "Inter",
            FontCategory::SansSerif,
            "rsms/inter",
            FontLicense::OFL,
        ),
        (
            "inter-display",
            "Inter Display",
            FontCategory::Display,
            "rsms/inter",
            FontLicense::OFL,
        ),
        // Red Hat
        (
            "red-hat-display",
            "Red Hat Display",
            FontCategory::Display,
            "RedHatOfficial/RedHatFont",
            FontLicense::OFL,
        ),
        (
            "red-hat-text",
            "Red Hat Text",
            FontCategory::SansSerif,
            "RedHatOfficial/RedHatFont",
            FontLicense::OFL,
        ),
        (
            "red-hat-mono",
            "Red Hat Mono",
            FontCategory::Monospace,
            "RedHatOfficial/RedHatFont",
            FontLicense::OFL,
        ),
        (
            "overpass",
            "Overpass",
            FontCategory::SansSerif,
            "RedHatOfficial/Overpass",
            FontLicense::OFL,
        ),
        (
            "overpass-mono",
            "Overpass Mono",
            FontCategory::Monospace,
            "RedHatOfficial/Overpass",
            FontLicense::OFL,
        ),
        // Intel
        (
            "intel-one-mono",
            "Intel One Mono",
            FontCategory::Monospace,
            "intel/intel-one-mono",
            FontLicense::OFL,
        ),
        // Other Popular
        (
            "recursive",
            "Recursive",
            FontCategory::SansSerif,
            "arrowtype/recursive",
            FontLicense::OFL,
        ),
        (
            "manrope",
            "Manrope",
            FontCategory::SansSerif,
            "sharanda/manrope",
            FontLicense::OFL,
        ),
        (
            "public-sans",
            "Public Sans",
            FontCategory::SansSerif,
            "uswds/public-sans",
            FontLicense::OFL,
        ),
        (
            "work-sans",
            "Work Sans",
            FontCategory::SansSerif,
            "weiweihuanghuang/Work-Sans",
            FontLicense::OFL,
        ),
        (
            "lexend",
            "Lexend",
            FontCategory::SansSerif,
            "googlefonts/lexend",
            FontLicense::OFL,
        ),
        (
            "atkinson-hyperlegible",
            "Atkinson Hyperlegible",
            FontCategory::SansSerif,
            "googlefonts/atkinson-hyperlegible",
            FontLicense::OFL,
        ),
        // Coding fonts
        (
            "hack",
            "Hack",
            FontCategory::Monospace,
            "source-foundry/Hack",
            FontLicense::MIT,
        ),
        (
            "victor-mono",
            "Victor Mono",
            FontCategory::Monospace,
            "rubjo/victor-mono",
            FontLicense::OFL,
        ),
        (
            "iosevka",
            "Iosevka",
            FontCategory::Monospace,
            "be5invis/Iosevka",
            FontLicense::OFL,
        ),
        (
            "mononoki",
            "Mononoki",
            FontCategory::Monospace,
            "madmalik/mononoki",
            FontLicense::OFL,
        ),
        (
            "fantasque-sans-mono",
            "Fantasque Sans Mono",
            FontCategory::Monospace,
            "belluzj/fantasque-sans",
            FontLicense::OFL,
        ),
        (
            "monoid",
            "Monoid",
            FontCategory::Monospace,
            "larsenwork/monoid",
            FontLicense::MIT,
        ),
        (
            "hasklig",
            "Hasklig",
            FontCategory::Monospace,
            "i-tu/Hasklig",
            FontLicense::OFL,
        ),
        (
            "comic-mono",
            "Comic Mono",
            FontCategory::Monospace,
            "dtinth/comic-mono-font",
            FontLicense::MIT,
        ),
        (
            "anonymous-pro",
            "Anonymous Pro",
            FontCategory::Monospace,
            "googlefonts/anonymouspro",
            FontLicense::OFL,
        ),
        (
            "inconsolata",
            "Inconsolata",
            FontCategory::Monospace,
            "googlefonts/Inconsolata",
            FontLicense::OFL,
        ),
        (
            "ubuntu-mono",
            "Ubuntu Mono",
            FontCategory::Monospace,
            "googlefonts/ubuntu",
            FontLicense::OFL,
        ),
        (
            "space-mono",
            "Space Mono",
            FontCategory::Monospace,
            "googlefonts/spacemono",
            FontLicense::OFL,
        ),
        (
            "dm-mono",
            "DM Mono",
            FontCategory::Monospace,
            "googlefonts/dm-mono",
            FontLicense::OFL,
        ),
        (
            "cousine",
            "Cousine",
            FontCategory::Monospace,
            "nicememes/Cousine",
            FontLicense::Apache2,
        ),
        // Liberation Fonts
        (
            "liberation-sans",
            "Liberation Sans",
            FontCategory::SansSerif,
            "liberationfonts/liberation-fonts",
            FontLicense::OFL,
        ),
        (
            "liberation-serif",
            "Liberation Serif",
            FontCategory::Serif,
            "liberationfonts/liberation-fonts",
            FontLicense::OFL,
        ),
        (
            "liberation-mono",
            "Liberation Mono",
            FontCategory::Monospace,
            "liberationfonts/liberation-fonts",
            FontLicense::OFL,
        ),
        // Display fonts
        (
            "lobster",
            "Lobster",
            FontCategory::Display,
            "nicememes/lobster",
            FontLicense::OFL,
        ),
        (
            "pacifico",
            "Pacifico",
            FontCategory::Handwriting,
            "googlefonts/Pacifico",
            FontLicense::OFL,
        ),
        (
            "dancing-script",
            "Dancing Script",
            FontCategory::Handwriting,
            "nicememes/dancing-script",
            FontLicense::OFL,
        ),
        (
            "caveat",
            "Caveat",
            FontCategory::Handwriting,
            "googlefonts/caveat",
            FontLicense::OFL,
        ),
        (
            "permanent-marker",
            "Permanent Marker",
            FontCategory::Display,
            "nicememes/permanent-marker",
            FontLicense::Apache2,
        ),
        // Korean
        (
            "pretendard",
            "Pretendard",
            FontCategory::SansSerif,
            "orioncactus/pretendard",
            FontLicense::OFL,
        ),
        (
            "spoqa-han-sans",
            "Spoqa Han Sans",
            FontCategory::SansSerif,
            "nicememes/spoqa-han-sans",
            FontLicense::OFL,
        ),
        // Chinese
        (
            "lxgw-wenkai",
            "LXGW WenKai",
            FontCategory::Serif,
            "lxgw/LxgwWenKai",
            FontLicense::OFL,
        ),
        (
            "sarasa-gothic",
            "Sarasa Gothic",
            FontCategory::SansSerif,
            "be5invis/Sarasa-Gothic",
            FontLicense::OFL,
        ),
    ]
}
