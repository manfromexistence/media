//! CDN URL generation for font previews
//!
//! This module provides CDN URLs for previewing fonts in real-world usage.
//! Supports multiple CDN providers:
//! - jsDelivr (for Fontsource fonts)
//! - Bunny Fonts CDN
//! - Google Fonts CDN
//! - unpkg CDN
//! - GitHub Raw (for GitHub-hosted fonts)

use serde::{Deserialize, Serialize};

/// CDN provider for font delivery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CdnProvider {
    JsDelivr,
    BunnyFonts,
    GoogleFonts,
    Unpkg,
    GitHubRaw,
    Cdnjs,
}

impl CdnProvider {
    pub fn base_url(&self) -> &str {
        match self {
            CdnProvider::JsDelivr => "https://cdn.jsdelivr.net/npm/@fontsource",
            CdnProvider::BunnyFonts => "https://fonts.bunny.net/css",
            CdnProvider::GoogleFonts => "https://fonts.googleapis.com/css2",
            CdnProvider::Unpkg => "https://unpkg.com/@fontsource",
            CdnProvider::GitHubRaw => "https://raw.githubusercontent.com/google/fonts/main/ofl",
            CdnProvider::Cdnjs => "https://cdnjs.cloudflare.com/ajax/libs",
        }
    }
}

/// Font CDN URLs for preview and usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontCdnUrls {
    /// CSS URL to include the font in a webpage
    pub css_url: Option<String>,
    /// Direct woff2 file URL
    pub woff2_url: Option<String>,
    /// Direct woff file URL
    pub woff_url: Option<String>,
    /// Direct TTF file URL
    pub ttf_url: Option<String>,
    /// HTML preview snippet
    pub preview_html: Option<String>,
    /// CDN provider used
    pub cdn_provider: CdnProvider,
}

/// Generate CDN URLs for a font
pub struct CdnUrlGenerator;

impl CdnUrlGenerator {
    /// Generate CDN URLs for a font from Google Fonts / Fontsource
    pub fn for_google_font(font_id: &str, font_name: &str) -> FontCdnUrls {
        let font_slug = font_id.to_lowercase().replace(' ', "-");
        let font_family = font_name.replace(' ', "+");

        FontCdnUrls {
            css_url: Some(format!(
                "https://fonts.googleapis.com/css2?family={}:wght@100;200;300;400;500;600;700;800;900&display=swap",
                font_family
            )),
            woff2_url: Some(format!(
                "https://cdn.jsdelivr.net/npm/@fontsource/{}/files/{}-latin-400-normal.woff2",
                font_slug, font_slug
            )),
            woff_url: Some(format!(
                "https://cdn.jsdelivr.net/npm/@fontsource/{}/files/{}-latin-400-normal.woff",
                font_slug, font_slug
            )),
            ttf_url: Some(format!(
                "https://raw.githubusercontent.com/google/fonts/main/ofl/{}/{}-Regular.ttf",
                font_slug.replace("-", ""),
                font_name.replace(' ', "")
            )),
            preview_html: Some(Self::generate_preview_html(font_name, &font_family)),
            cdn_provider: CdnProvider::GoogleFonts,
        }
    }

    /// Generate CDN URLs for a Bunny Fonts font
    pub fn for_bunny_font(_font_id: &str, font_name: &str) -> FontCdnUrls {
        let font_family = font_name.replace(' ', "+");

        FontCdnUrls {
            css_url: Some(format!(
                "https://fonts.bunny.net/css?family={}:100,200,300,400,500,600,700,800,900",
                font_family
            )),
            woff2_url: None, // Bunny provides CSS-based delivery
            woff_url: None,
            ttf_url: None,
            preview_html: Some(Self::generate_bunny_preview_html(font_name, &font_family)),
            cdn_provider: CdnProvider::BunnyFonts,
        }
    }

    /// Generate CDN URLs for a Fontsource font
    pub fn for_fontsource_font(font_id: &str) -> FontCdnUrls {
        let font_slug = font_id.to_lowercase().replace(' ', "-");

        FontCdnUrls {
            css_url: Some(format!(
                "https://cdn.jsdelivr.net/npm/@fontsource/{}/index.css",
                font_slug
            )),
            woff2_url: Some(format!(
                "https://cdn.jsdelivr.net/npm/@fontsource/{}/files/{}-latin-400-normal.woff2",
                font_slug, font_slug
            )),
            woff_url: Some(format!(
                "https://cdn.jsdelivr.net/npm/@fontsource/{}/files/{}-latin-400-normal.woff",
                font_slug, font_slug
            )),
            ttf_url: None,
            preview_html: Some(Self::generate_jsdelivr_preview_html(&font_slug)),
            cdn_provider: CdnProvider::JsDelivr,
        }
    }

    /// Generate CDN URLs for a GitHub-hosted font
    pub fn for_github_font(repo: &str, font_path: &str, font_name: &str) -> FontCdnUrls {
        FontCdnUrls {
            css_url: None,
            woff2_url: Some(format!(
                "https://cdn.jsdelivr.net/gh/{}/{}.woff2",
                repo, font_path
            )),
            woff_url: Some(format!(
                "https://cdn.jsdelivr.net/gh/{}/{}.woff",
                repo, font_path
            )),
            ttf_url: Some(format!(
                "https://cdn.jsdelivr.net/gh/{}/{}.ttf",
                repo, font_path
            )),
            preview_html: Some(Self::generate_github_preview_html(
                repo, font_path, font_name,
            )),
            cdn_provider: CdnProvider::GitHubRaw,
        }
    }

    /// Generate a preview HTML snippet with Google Fonts
    fn generate_preview_html(font_name: &str, font_family: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family={}&display=swap" rel="stylesheet">
    <style>
        body {{ font-family: '{}', sans-serif; padding: 20px; }}
        h1 {{ font-size: 48px; }}
        p {{ font-size: 24px; line-height: 1.6; }}
        .weights {{ display: flex; flex-direction: column; gap: 10px; }}
        .weight-100 {{ font-weight: 100; }}
        .weight-300 {{ font-weight: 300; }}
        .weight-400 {{ font-weight: 400; }}
        .weight-700 {{ font-weight: 700; }}
        .weight-900 {{ font-weight: 900; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>The quick brown fox jumps over the lazy dog.</p>
    <p>ABCDEFGHIJKLMNOPQRSTUVWXYZ</p>
    <p>abcdefghijklmnopqrstuvwxyz</p>
    <p>0123456789 !@#$%^&*()</p>
    <div class="weights">
        <p class="weight-100">Thin (100) - The quick brown fox</p>
        <p class="weight-300">Light (300) - The quick brown fox</p>
        <p class="weight-400">Regular (400) - The quick brown fox</p>
        <p class="weight-700">Bold (700) - The quick brown fox</p>
        <p class="weight-900">Black (900) - The quick brown fox</p>
    </div>
</body>
</html>"#,
            font_family, font_name, font_name
        )
    }

    /// Generate a preview HTML snippet with Bunny Fonts
    fn generate_bunny_preview_html(font_name: &str, font_family: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <link rel="preconnect" href="https://fonts.bunny.net">
    <link href="https://fonts.bunny.net/css?family={}" rel="stylesheet">
    <style>
        body {{ font-family: '{}', sans-serif; padding: 20px; }}
        h1 {{ font-size: 48px; }}
        p {{ font-size: 24px; line-height: 1.6; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>The quick brown fox jumps over the lazy dog.</p>
    <p>ABCDEFGHIJKLMNOPQRSTUVWXYZ</p>
    <p>abcdefghijklmnopqrstuvwxyz</p>
    <p>0123456789 !@#$%^&*()</p>
</body>
</html>"#,
            font_family, font_name, font_name
        )
    }

    /// Generate a preview HTML snippet with jsDelivr
    fn generate_jsdelivr_preview_html(font_slug: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <link href="https://cdn.jsdelivr.net/npm/@fontsource/{}/index.css" rel="stylesheet">
    <style>
        body {{ font-family: '{}', sans-serif; padding: 20px; }}
        h1 {{ font-size: 48px; }}
        p {{ font-size: 24px; line-height: 1.6; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>The quick brown fox jumps over the lazy dog.</p>
    <p>ABCDEFGHIJKLMNOPQRSTUVWXYZ</p>
    <p>abcdefghijklmnopqrstuvwxyz</p>
    <p>0123456789 !@#$%^&*()</p>
</body>
</html>"#,
            font_slug, font_slug, font_slug
        )
    }

    /// Generate a preview HTML snippet for GitHub-hosted fonts
    fn generate_github_preview_html(repo: &str, font_path: &str, font_name: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <style>
        @font-face {{
            font-family: '{}';
            src: url('https://cdn.jsdelivr.net/gh/{}/{}.woff2') format('woff2'),
                 url('https://cdn.jsdelivr.net/gh/{}/{}.woff') format('woff');
        }}
        body {{ font-family: '{}', sans-serif; padding: 20px; }}
        h1 {{ font-size: 48px; }}
        p {{ font-size: 24px; line-height: 1.6; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>The quick brown fox jumps over the lazy dog.</p>
    <p>ABCDEFGHIJKLMNOPQRSTUVWXYZ</p>
    <p>abcdefghijklmnopqrstuvwxyz</p>
    <p>0123456789 !@#$%^&*()</p>
</body>
</html>"#,
            font_name, repo, font_path, repo, font_path, font_name, font_name
        )
    }
}

/// Common popular fonts with their CDN URLs
pub fn get_popular_font_cdn_urls() -> Vec<(&'static str, FontCdnUrls)> {
    vec![
        (
            "Roboto",
            CdnUrlGenerator::for_google_font("roboto", "Roboto"),
        ),
        (
            "Open Sans",
            CdnUrlGenerator::for_google_font("open-sans", "Open Sans"),
        ),
        ("Lato", CdnUrlGenerator::for_google_font("lato", "Lato")),
        (
            "Montserrat",
            CdnUrlGenerator::for_google_font("montserrat", "Montserrat"),
        ),
        ("Inter", CdnUrlGenerator::for_google_font("inter", "Inter")),
        (
            "Poppins",
            CdnUrlGenerator::for_google_font("poppins", "Poppins"),
        ),
        (
            "Source Code Pro",
            CdnUrlGenerator::for_google_font("source-code-pro", "Source Code Pro"),
        ),
        (
            "Fira Code",
            CdnUrlGenerator::for_google_font("fira-code", "Fira Code"),
        ),
        (
            "JetBrains Mono",
            CdnUrlGenerator::for_google_font("jetbrains-mono", "JetBrains Mono"),
        ),
        (
            "Playfair Display",
            CdnUrlGenerator::for_google_font("playfair-display", "Playfair Display"),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_font_cdn_urls() {
        let urls = CdnUrlGenerator::for_google_font("roboto", "Roboto");
        assert!(urls.css_url.is_some());
        assert!(urls.woff2_url.is_some());
        assert!(urls.preview_html.is_some());
    }

    #[test]
    fn test_bunny_font_cdn_urls() {
        let urls = CdnUrlGenerator::for_bunny_font("roboto", "Roboto");
        assert!(urls.css_url.is_some());
        assert!(urls.preview_html.is_some());
    }
}
