use crate::types::{IconMetadata, IconPack};
use anyhow::Result;
use std::path::Path;

/// Parse all JSON icon files from data directory
pub fn parse_icon_files(data_dir: &Path) -> Result<Vec<IconMetadata>> {
    let mut all_icons = Vec::new();
    let mut icon_id = 0u32;

    for entry in std::fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = std::fs::read_to_string(&path)?;
            let pack: IconPack = serde_json::from_str(&content)?;

            for (icon_name, _icon_data) in pack.icons.iter() {
                all_icons.push(IconMetadata {
                    id: icon_id,
                    name: icon_name.clone(),
                    pack: pack.prefix.clone(),
                    category: String::new(),
                    tags: vec![],
                    popularity: 0,
                });
                icon_id += 1;
            }
        }
    }

    Ok(all_icons)
}

/// Extract icon names for FST building
pub fn extract_icon_names(icons: &[IconMetadata]) -> Vec<(String, u32)> {
    icons
        .iter()
        .map(|icon| (icon.name.clone(), icon.id))
        .collect()
}
