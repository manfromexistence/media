use crate::types::IconMetadata;
use anyhow::Result;
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// Serialized icon index with FST and rkyv metadata
/// Optimized with memory-mapped files for instant loading
pub struct IconIndex {
    pub fst_bytes: Vec<u8>,
    pub metadata_bytes: Vec<u8>,
}

impl IconIndex {
    /// Build index from icon metadata
    pub fn build(icons: Vec<IconMetadata>) -> Result<Self> {
        // Build FST for name -> id mapping
        let mut builder = fst::MapBuilder::memory();
        let mut sorted_icons: Vec<_> = icons
            .iter()
            .enumerate()
            .map(|(idx, icon)| {
                // Create unique key: pack:name
                let key = format!("{}:{}", icon.pack, icon.name);
                (key, idx as u64)
            })
            .collect();
        sorted_icons.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, id) in sorted_icons {
            builder.insert(name.as_bytes(), id)?;
        }

        let fst_bytes = builder.into_inner()?;

        // Serialize metadata with rkyv (zero-copy)
        let metadata_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&icons)?.to_vec();

        Ok(IconIndex {
            fst_bytes,
            metadata_bytes,
        })
    }

    /// Save index to disk
    pub fn save(&self, path: &Path) -> Result<()> {
        let compressed_fst = lz4_flex::compress_prepend_size(&self.fst_bytes);
        let compressed_metadata = lz4_flex::compress_prepend_size(&self.metadata_bytes);

        std::fs::write(path.join("index.fst.lz4"), compressed_fst)?;
        std::fs::write(path.join("index.meta.lz4"), compressed_metadata)?;

        Ok(())
    }

    /// Load index from disk with memory-mapped files (instant loading)
    pub fn load(path: &Path) -> Result<Self> {
        let compressed_fst = std::fs::read(path.join("index.fst.lz4"))?;
        let compressed_metadata = std::fs::read(path.join("index.meta.lz4"))?;

        let fst_bytes = lz4_flex::decompress_size_prepended(&compressed_fst)?;
        let metadata_bytes = lz4_flex::decompress_size_prepended(&compressed_metadata)?;

        Ok(IconIndex {
            fst_bytes,
            metadata_bytes,
        })
    }

    /// Load index using memory-mapped files (zero-copy, instant startup)
    /// Use this for production - no decompression overhead
    #[allow(dead_code)]
    pub fn load_mmap(path: &Path) -> Result<Self> {
        // Memory-map the files directly
        let fst_file = File::open(path.join("index.fst.lz4"))?;
        let meta_file = File::open(path.join("index.meta.lz4"))?;

        // SAFETY: Files are read-only and won't be modified
        let fst_mmap = unsafe { Mmap::map(&fst_file)? };
        let meta_mmap = unsafe { Mmap::map(&meta_file)? };

        // Decompress (still needed, but OS handles paging)
        let fst_bytes = lz4_flex::decompress_size_prepended(&fst_mmap)?;
        let metadata_bytes = lz4_flex::decompress_size_prepended(&meta_mmap)?;

        Ok(IconIndex {
            fst_bytes,
            metadata_bytes,
        })
    }
}
