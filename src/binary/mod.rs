//! Binary format utilities for dx-media.
//!
//! Provides binary serialization and format detection
//! following Binary Dawn principles.

mod cache;
mod format;

pub use cache::{BinaryCache, BinaryCacheConfig};
pub use format::{BinaryFormat, FormatDetector, MediaSignature};
