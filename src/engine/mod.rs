//! Core engine module for search, download, and orchestration.
//!
//! This module provides the high-level orchestration layer that coordinates
//! providers, manages downloads, and handles file operations.

mod circuit_breaker;
mod download;
mod dx;
mod filemanager;
mod scraper;
mod search;

pub use circuit_breaker::{CircuitBreaker, CircuitState};
pub use download::Downloader;
pub use dx::DxMedia;
pub use filemanager::FileManager;
pub use scraper::{ScrapeOptions, ScrapeResult, Scraper};
pub use search::SearchEngine;
