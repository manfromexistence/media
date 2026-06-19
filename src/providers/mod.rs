//! Provider abstraction layer.
//!
//! This module defines the [`Provider`] trait and implementations for various
//! media asset providers. Provider listings expose source kind, credential
//! requirements, credential status, and unavailable reasons so callers can
//! distinguish keyless public sources from credential-gated sources.
//!
//! Credential-gated providers gracefully degrade - they remain discoverable in
//! registry metadata but do not participate in searches when credentials are not
//! configured.

// ═══════════════════════════════════════════════════════════════════════════════
// CORE PROVIDER IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════
mod archive; // 26M+ media items (images, video, audio, docs)
mod artic; // 50K+ CC0 artworks
mod catapi; // 60K+ cat images
mod cleveland;
mod datagov; // 300K+ US Government datasets (JSON, CSV, XML)
mod dicebear; // Unlimited avatar generation
mod dogceo; // 20K+ dog images
mod dpla;
mod europeana;
mod github; // Data files (JSON, CSV, PDF, Excel)
pub(crate) mod listing;
mod loc;
mod loremflickr; // Generated image endpoints; original Flickr provenance unresolved
mod met;
mod nasa;
mod nekosbest; // Anime images and GIFs
mod openlibrary; // 30M+ book covers
mod openverse;
mod picsum;
mod polyhaven;
mod provenance;
mod randomfox; // Unlimited fox images
mod rijksmuseum;
mod robohash; // Unlimited robot avatars
mod scryfall; // 80K+ MTG cards
mod vanda; // 1.2M+ V&A Museum items
mod waifupics; // Anime images and GIFs
mod walters; // 25K+ Walters artworks
mod wikimedia;
mod xkcd; // 2,900+ comics

// ═══════════════════════════════════════════════════════════════════════════════
// CREDENTIAL-GATED PROVIDERS
// ═══════════════════════════════════════════════════════════════════════════════
mod freesound; // 600K+ sound effects (free API key)
mod giphy; // Millions of GIFs (free API key)
mod pexels; // 3.5M+ photos & videos (free API key)
mod pixabay; // 4.2M+ images, videos, music (free API key)
mod smithsonian;
mod unsplash; // 5M+ high-quality photos (free API key) // 4.5M+ CC0 images (free API key)

mod registry;

/// Provider traits module.
pub mod traits;

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC EXPORTS - CORE PROVIDERS
// ═══════════════════════════════════════════════════════════════════════════════
pub use archive::InternetArchiveProvider;
pub use artic::ArtInstituteChicagoProvider;
pub use catapi::CatApiProvider;
pub use cleveland::ClevelandMuseumProvider;
pub use datagov::DataGovProvider;
pub use dicebear::DiceBearProvider;
pub use dogceo::DogCeoProvider;
pub use dpla::DplaProvider;
pub use europeana::EuropeanaProvider;
pub use github::GitHubProvider;
pub use loc::LibraryOfCongressProvider;
pub use loremflickr::LoremFlickrProvider;
pub use met::MetMuseumProvider;
pub use nasa::NasaImagesProvider;
pub use nekosbest::NekosBestProvider;
pub use openlibrary::OpenLibraryProvider;
pub use openverse::OpenverseProvider;
pub use picsum::LoremPicsumProvider;
pub use polyhaven::PolyHavenProvider;
pub use randomfox::RandomFoxProvider;
pub use rijksmuseum::RijksmuseumProvider;
pub use robohash::RoboHashProvider;
pub use scryfall::ScryfallProvider;
pub use vanda::VandAMuseumProvider;
pub use waifupics::WaifuPicsProvider;
pub use walters::WaltersArtMuseumProvider;
pub use wikimedia::WikimediaCommonsProvider;
pub use xkcd::XkcdProvider;

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC EXPORTS - CREDENTIAL-GATED PROVIDERS
// ═══════════════════════════════════════════════════════════════════════════════
pub use freesound::FreesoundProvider;
pub use giphy::GiphyProvider;
pub use pexels::PexelsProvider;
pub use pixabay::PixabayProvider;
pub use smithsonian::SmithsonianProvider;
pub use unsplash::UnsplashProvider;

// ═══════════════════════════════════════════════════════════════════════════════
// REGISTRY & TRAITS
// ═══════════════════════════════════════════════════════════════════════════════
pub use registry::ProviderRegistry;
pub use traits::{Provider, ProviderInfo};
