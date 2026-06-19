//! Core data models for dx-font
//!
//! This module contains all the data structures used throughout the library,
//! including font representations, search queries, and provider information.
//!
//! ## Key Types
//!
//! - [`Font`] - A simplified font representation for search results
//! - [`FontFamily`] - A complete font family with all variants
//! - [`FontProvider`] - Enumeration of supported font providers
//! - `SearchQuery` - Parameters for font searches
//! - `SearchResults` - Results from a font search operation
//!
//! ## Example
//!
//! ```rust
//! use dx_font::models::{Font, FontProvider, FontCategory};
//!
//! // Font providers can be compared and used as keys
//! let provider = FontProvider::GoogleFonts;
//! assert_eq!(provider.name(), "Google Fonts");
//! ```

use serde::{Deserialize, Serialize};

/// Represents a font provider/source.
///
/// Font providers are the sources from which fonts can be searched and downloaded.
/// Each provider has a unique name and base URL.
///
/// # Example
///
/// ```rust
/// use dx_font::FontProvider;
///
/// let provider = FontProvider::GoogleFonts;
/// assert_eq!(provider.name(), "Google Fonts");
/// assert_eq!(provider.base_url(), "https://fonts.google.com");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FontProvider {
    // Tier 1: Primary APIs
    GoogleFonts,
    BunnyFonts,
    Fontsource,
    GoogleWebfontsHelper,
    FontLibrary,

    // Tier 2: Major Free Sites
    FontSquirrel,
    DaFont,
    Fonts1001,
    FontSpace,
    AbstractFonts,
    UrbanFonts,
    FontZone,
    FFonts,
    FontMeme,
    FontRiver,

    // Tier 3: Curated Foundries
    FontShare,
    Velvetyne,
    OpenFoundry,
    LeagueOfMoveableType,
    Uncut,
    Collletttivo,
    OmnibusType,
    FreeFacesGallery,
    UseModify,
    BeautifulWebType,
    Fontain,
    GoodFonts,
    Befonts,
    LostType,
    AtipoFoundry,

    // Tier 4: GitHub Repositories
    GitHub,

    // Tier 5: International
    NotoFonts,
    ArabicFonts,
    ChinazFonts,
    FreeJapaneseFonts,
    Noonnu,
    HindiFonts,
    ThaiFonts,
    FonterRu,
    FontsIr,
    TamilFonts,
    BengaliFonts,
    SMCMalayalam,

    // Custom/Other
    Custom(String),
}

impl FontProvider {
    /// Get the human-readable name of this provider.
    pub fn name(&self) -> &str {
        match self {
            FontProvider::GoogleFonts => "Google Fonts",
            FontProvider::BunnyFonts => "Bunny Fonts",
            FontProvider::Fontsource => "Fontsource",
            FontProvider::GoogleWebfontsHelper => "Google Webfonts Helper",
            FontProvider::FontLibrary => "Font Library",
            FontProvider::FontSquirrel => "Font Squirrel",
            FontProvider::DaFont => "DaFont",
            FontProvider::Fonts1001 => "1001 Fonts",
            FontProvider::FontSpace => "FontSpace",
            FontProvider::AbstractFonts => "Abstract Fonts",
            FontProvider::UrbanFonts => "Urban Fonts",
            FontProvider::FontZone => "Font Zone",
            FontProvider::FFonts => "FFonts",
            FontProvider::FontMeme => "Font Meme",
            FontProvider::FontRiver => "Font River",
            FontProvider::FontShare => "FontShare",
            FontProvider::Velvetyne => "Velvetyne",
            FontProvider::OpenFoundry => "Open Foundry",
            FontProvider::LeagueOfMoveableType => "The League of Moveable Type",
            FontProvider::Uncut => "Uncut.wtf",
            FontProvider::Collletttivo => "Collletttivo",
            FontProvider::OmnibusType => "OMNIBUS-TYPE",
            FontProvider::FreeFacesGallery => "Free Faces Gallery",
            FontProvider::UseModify => "Use & Modify",
            FontProvider::BeautifulWebType => "Beautiful Web Type",
            FontProvider::Fontain => "Fontain",
            FontProvider::GoodFonts => "Good Fonts",
            FontProvider::Befonts => "Befonts",
            FontProvider::LostType => "Lost Type Co-op",
            FontProvider::AtipoFoundry => "Atipo Foundry",
            FontProvider::GitHub => "GitHub",
            FontProvider::NotoFonts => "Noto Fonts",
            FontProvider::ArabicFonts => "Arabic Fonts",
            FontProvider::ChinazFonts => "Chinaz Fonts",
            FontProvider::FreeJapaneseFonts => "Free Japanese Fonts",
            FontProvider::Noonnu => "Noonnu (Korean)",
            FontProvider::HindiFonts => "Hindi Fonts",
            FontProvider::ThaiFonts => "Thai Fonts",
            FontProvider::FonterRu => "Fonter.ru",
            FontProvider::FontsIr => "Fonts.ir",
            FontProvider::TamilFonts => "Tamil Fonts",
            FontProvider::BengaliFonts => "Bengali Fonts",
            FontProvider::SMCMalayalam => "SMC Malayalam",
            FontProvider::Custom(name) => name,
        }
    }

    /// Get the base URL for this provider's website or API.
    pub fn base_url(&self) -> &str {
        match self {
            FontProvider::GoogleFonts => "https://fonts.google.com",
            FontProvider::BunnyFonts => "https://fonts.bunny.net",
            FontProvider::Fontsource => "https://api.fontsource.org/v1/fonts",
            FontProvider::GoogleWebfontsHelper => "https://gwfh.mranftl.com/api/fonts",
            FontProvider::FontLibrary => "https://fontlibrary.org",
            FontProvider::FontSquirrel => "https://www.fontsquirrel.com",
            FontProvider::DaFont => "https://www.dafont.com",
            FontProvider::Fonts1001 => "https://www.1001fonts.com",
            FontProvider::FontSpace => "https://www.fontspace.com",
            FontProvider::AbstractFonts => "https://www.abstractfonts.com",
            FontProvider::UrbanFonts => "https://www.urbanfonts.com",
            FontProvider::FontZone => "https://fontzone.net",
            FontProvider::FFonts => "https://www.ffonts.net",
            FontProvider::FontMeme => "https://fontmeme.com",
            FontProvider::FontRiver => "https://www.fontriver.com",
            FontProvider::FontShare => "https://www.fontshare.com",
            FontProvider::Velvetyne => "https://velvetyne.fr",
            FontProvider::OpenFoundry => "https://open-foundry.com",
            FontProvider::LeagueOfMoveableType => "https://www.theleagueofmoveabletype.com",
            FontProvider::Uncut => "https://uncut.wtf",
            FontProvider::Collletttivo => "https://www.collletttivo.it",
            FontProvider::OmnibusType => "https://www.omnibus-type.com",
            FontProvider::FreeFacesGallery => "https://www.freefaces.gallery",
            FontProvider::UseModify => "https://usemodify.com",
            FontProvider::BeautifulWebType => "https://beautifulwebtype.com",
            FontProvider::Fontain => "https://fontain.org",
            FontProvider::GoodFonts => "https://goodfonts.io",
            FontProvider::Befonts => "https://befonts.com",
            FontProvider::LostType => "https://www.losttype.com",
            FontProvider::AtipoFoundry => "https://www.atipofoundry.com",
            FontProvider::GitHub => "https://github.com",
            FontProvider::NotoFonts => "https://fonts.google.com/noto",
            FontProvider::ArabicFonts => "https://arabicfonts.net",
            FontProvider::ChinazFonts => "https://font.chinaz.com",
            FontProvider::FreeJapaneseFonts => "https://freejapanesefont.com",
            FontProvider::Noonnu => "https://noonnu.cc",
            FontProvider::HindiFonts => "https://hindityping.com/fonts",
            FontProvider::ThaiFonts => "https://f0nt.com",
            FontProvider::FonterRu => "https://fonter.ru",
            FontProvider::FontsIr => "https://fonts.ir",
            FontProvider::TamilFonts => "https://tamilfonts.net",
            FontProvider::BengaliFonts => "https://banglafonts.net",
            FontProvider::SMCMalayalam => "https://smc.org.in/fonts",
            FontProvider::Custom(url) => url,
        }
    }
}

/// Font weight enumeration.
///
/// Represents the weight (thickness) of a font, from Thin (100) to Black (900).
///
/// # Example
///
/// ```rust
/// use dx_font::FontWeight;
///
/// let weight = FontWeight::Bold;
/// assert_eq!(weight.to_numeric(), 700);
///
/// let weight = FontWeight::from_numeric(400);
/// assert_eq!(weight, FontWeight::Regular);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FontWeight {
    Thin,       // 100
    ExtraLight, // 200
    Light,      // 300
    Regular,    // 400
    Medium,     // 500
    SemiBold,   // 600
    Bold,       // 700
    ExtraBold,  // 800
    Black,      // 900
}

impl FontWeight {
    /// Convert a numeric weight value to a FontWeight enum.
    ///
    /// Values are mapped to the nearest standard weight:
    /// - 0-150: Thin (100)
    /// - 151-250: ExtraLight (200)
    /// - 251-350: Light (300)
    /// - 351-450: Regular (400)
    /// - 451-550: Medium (500)
    /// - 551-650: SemiBold (600)
    /// - 651-750: Bold (700)
    /// - 751-850: ExtraBold (800)
    /// - 851+: Black (900)
    pub fn from_numeric(weight: u16) -> Self {
        match weight {
            0..=150 => FontWeight::Thin,
            151..=250 => FontWeight::ExtraLight,
            251..=350 => FontWeight::Light,
            351..=450 => FontWeight::Regular,
            451..=550 => FontWeight::Medium,
            551..=650 => FontWeight::SemiBold,
            651..=750 => FontWeight::Bold,
            751..=850 => FontWeight::ExtraBold,
            _ => FontWeight::Black,
        }
    }

    /// Convert this FontWeight to its numeric value.
    pub fn to_numeric(&self) -> u16 {
        match self {
            FontWeight::Thin => 100,
            FontWeight::ExtraLight => 200,
            FontWeight::Light => 300,
            FontWeight::Regular => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
        }
    }
}

/// Font style (normal or italic).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FontStyle {
    /// Normal (upright) style.
    Normal,
    /// Italic (slanted) style.
    Italic,
}

/// Font category/classification.
///
/// Categories help organize fonts by their visual characteristics and intended use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FontCategory {
    /// Fonts with serifs (small decorative strokes).
    Serif,
    /// Fonts without serifs.
    SansSerif,
    /// Decorative fonts for headlines and titles.
    Display,
    /// Fonts that mimic handwriting.
    Handwriting,
    /// Fixed-width fonts for code and technical content.
    Monospace,
}

/// License type for fonts.
///
/// Indicates the licensing terms under which a font can be used.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FontLicense {
    /// SIL Open Font License - most common for open source fonts.
    OFL,
    /// Apache License 2.0.
    Apache2,
    /// MIT License.
    MIT,
    /// GNU General Public License.
    GPL,
    /// Public domain - no restrictions.
    PublicDomain,
    /// Free for commercial use (specific terms vary).
    FreeCommercial,
    /// Custom license with specific terms.
    Custom(String),
}

/// A single font variant (e.g., Regular, Bold Italic).
///
/// A variant represents a specific combination of weight and style within a font family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontVariant {
    /// The weight of this variant.
    pub weight: FontWeight,
    /// The style of this variant.
    pub style: FontStyle,
    /// URL to download this variant's font file.
    pub file_url: Option<String>,
    /// File format (ttf, otf, woff, woff2).
    pub file_format: String,
}

/// Represents a font family with all its variants.
///
/// A font family contains metadata about the font and all available variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamily {
    /// Unique identifier for this font family.
    pub id: String,
    /// Display name of the font family.
    pub name: String,
    /// Provider this font comes from.
    pub provider: FontProvider,
    /// Category/classification of the font.
    pub category: Option<FontCategory>,
    /// All available variants (weight/style combinations).
    pub variants: Vec<FontVariant>,
    /// License under which the font is distributed.
    pub license: Option<FontLicense>,
    /// Name of the font designer.
    pub designer: Option<String>,
    /// Description of the font.
    pub description: Option<String>,
    /// URL to preview the font.
    pub preview_url: Option<String>,
    /// URL to download the font.
    pub download_url: Option<String>,
    /// Languages supported by the font.
    pub languages: Vec<String>,
    /// Character subsets available.
    pub subsets: Vec<String>,
    /// Popularity ranking (if available).
    pub popularity: Option<u32>,
    /// Last modification date.
    pub last_modified: Option<String>,
}

/// A simplified font representation for search results.
///
/// This is a lightweight version of [`FontFamily`] used in search results
/// to reduce memory usage and improve performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Font {
    /// Unique identifier for this font.
    pub id: String,
    /// Display name of the font.
    pub name: String,
    /// Provider this font comes from.
    pub provider: FontProvider,
    /// Category/classification of the font.
    pub category: Option<FontCategory>,
    /// Number of variants available.
    pub variant_count: usize,
    /// License under which the font is distributed.
    pub license: Option<FontLicense>,
    /// URL to preview the font.
    pub preview_url: Option<String>,
    /// URL to download the font.
    pub download_url: Option<String>,
}

impl From<FontFamily> for Font {
    fn from(family: FontFamily) -> Self {
        Font {
            id: family.id,
            name: family.name,
            provider: family.provider,
            category: family.category,
            variant_count: family.variants.len(),
            license: family.license,
            preview_url: family.preview_url,
            download_url: family.download_url,
        }
    }
}

/// Search query parameters.
///
/// Use this struct to customize font searches with filters and pagination.
///
/// # Example
///
/// ```rust
/// use dx_font::models::{SearchQuery, FontCategory, FontProvider};
///
/// let query = SearchQuery {
///     query: "roboto".to_string(),
///     category: Some(FontCategory::SansSerif),
///     limit: Some(10),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// Search term to match against font names.
    pub query: String,
    /// Limit search to specific providers.
    pub providers: Option<Vec<FontProvider>>,
    /// Filter by font category.
    pub category: Option<FontCategory>,
    /// Filter by license type.
    pub license: Option<FontLicense>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
    /// Number of results to skip (for pagination).
    pub offset: Option<usize>,
}

/// Provider error information for reporting.
///
/// When a provider fails during a search, this struct captures the error details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderError {
    /// Name of the provider that failed.
    pub provider: String,
    /// Type of error that occurred.
    pub error_type: ProviderErrorType,
    /// Human-readable error message.
    pub message: String,
}

/// Types of provider errors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderErrorType {
    /// Network connection failed.
    Network,
    /// Request timed out.
    Timeout,
    /// Rate limit exceeded.
    RateLimit,
    /// Failed to parse response.
    Parse,
    /// Provider not available.
    NotAvailable,
}

/// Search results with metadata.
///
/// Contains the fonts found by a search operation along with metadata
/// about the search itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// List of fonts matching the search criteria.
    pub fonts: Vec<Font>,
    /// Total number of fonts found.
    pub total: usize,
    /// The search query that was executed.
    pub query: String,
    /// Names of providers that were searched.
    pub providers_searched: Vec<String>,
    /// Providers that failed and their error messages.
    #[serde(default)]
    pub provider_errors: Vec<ProviderError>,
    /// Whether results came from cache.
    #[serde(default)]
    pub from_cache: bool,
}

/// Download options for font files.
///
/// Configure how fonts should be downloaded, including output directory
/// and which formats/variants to include.
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    /// Directory where downloaded fonts will be saved.
    pub output_dir: std::path::PathBuf,
    /// Font formats to download (ttf, otf, woff, woff2).
    pub formats: Vec<String>,
    /// Specific weights to download (None = all weights).
    pub weights: Option<Vec<FontWeight>>,
    /// Specific styles to download (None = all styles).
    pub styles: Option<Vec<FontStyle>>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            output_dir: std::path::PathBuf::from("./fonts"),
            formats: vec!["ttf".to_string(), "woff2".to_string()],
            weights: None,
            styles: None,
        }
    }
}
