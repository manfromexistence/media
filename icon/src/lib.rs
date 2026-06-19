//! DX Icon Search - World's fastest icon search engine
//!
//! Features:
//! - FST-based prefix search (<0.1ms)
//! - Zero-copy rkyv metadata access
//! - Semantic search with embeddings
//! - Multi-threaded WASM support
//! - LZ4 compression for network transfer

pub mod avx_search;
pub mod bloom;
pub mod builder;
pub mod engine;
// GPU module commented out - not useful for most use cases
// #[cfg(feature = "gpu")]
// pub mod gpu;
pub mod index;
pub mod multipattern;
pub mod optimized;
pub mod parser;
pub mod perfect_hash;
pub mod precomputed;
pub mod search;
pub mod types;
pub mod zero_alloc;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use engine::IconSearchEngine;
pub use search::SearchResult;
pub use types::{IconMetadata, IconPack};
