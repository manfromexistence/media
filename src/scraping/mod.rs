//! Scraping targets and configurations for 200+ free media websites.
//!
//! This module provides pre-configured scraping targets for websites that don't have APIs
//! but offer free media assets. Each target includes:
//! - URL patterns
//! - CSS selectors for media extraction
//! - License information
//! - Rate limiting recommendations

mod registry;
mod targets;

pub use registry::{SCRAPING_TARGETS, ScrapingRegistry};
pub use targets::{ScrapingCategory, ScrapingMethod, ScrapingTarget};
