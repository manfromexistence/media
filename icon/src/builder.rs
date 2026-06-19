use crate::index::IconIndex;
use crate::parser::parse_icon_files;
use anyhow::Result;
use std::path::Path;

/// Build icon search index from JSON data
pub struct IndexBuilder;

impl IndexBuilder {
    /// Build index from data directory
    pub fn build_from_dir(data_dir: &Path, output_dir: &Path) -> Result<()> {
        println!("Parsing icon files from {:?}...", data_dir);
        let icons = parse_icon_files(data_dir)?;
        println!("Parsed {} icons", icons.len());

        println!("Building FST and rkyv index...");
        let index = IconIndex::build(icons)?;

        println!("Saving index to {:?}...", output_dir);
        std::fs::create_dir_all(output_dir)?;
        index.save(output_dir)?;

        println!("Index built successfully!");
        Ok(())
    }
}
