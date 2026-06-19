use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct IconPack {
    prefix: String,
    info: IconPackInfo,
    #[serde(rename = "lastModified")]
    last_modified: u64,
    icons: HashMap<String, IconData>,
}

#[derive(Serialize, Deserialize)]
struct IconPackInfo {
    name: String,
    total: usize,
    author: Author,
    license: License,
    samples: Vec<String>,
    height: u32,
    category: String,
    tags: Vec<String>,
    palette: bool,
}

#[derive(Serialize, Deserialize)]
struct Author {
    name: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct License {
    title: String,
    spdx: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct IconData {
    body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
}

fn extract_svg_body(svg_content: &str) -> Result<(String, Option<u32>, Option<u32>), String> {
    // Extract viewBox or width/height
    let width = extract_attribute(svg_content, "width");
    let height = extract_attribute(svg_content, "height");

    // Extract viewBox if width/height not found
    let (final_width, final_height) = if width.is_none() || height.is_none() {
        if let Some(viewbox) = extract_viewbox(svg_content) {
            viewbox
        } else {
            (width, height)
        }
    } else {
        (width, height)
    };

    // Extract body (everything between <svg...> and </svg>)
    let body = extract_inner_content(svg_content)?;

    Ok((body, final_width, final_height))
}

fn extract_attribute(svg: &str, attr: &str) -> Option<u32> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = svg.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = svg[start..].find('"') {
            let value = &svg[start..start + end];
            // Parse numeric value, ignoring units
            value
                .chars()
                .take_while(|c| c.is_numeric())
                .collect::<String>()
                .parse()
                .ok()
        } else {
            None
        }
    } else {
        None
    }
}

fn extract_viewbox(svg: &str) -> Option<(Option<u32>, Option<u32>)> {
    if let Some(start) = svg.find("viewBox=\"") {
        let start = start + 9;
        if let Some(end) = svg[start..].find('"') {
            let viewbox = &svg[start..start + end];
            let parts: Vec<&str> = viewbox.split_whitespace().collect();
            if parts.len() == 4 {
                let width = parts[2].parse().ok();
                let height = parts[3].parse().ok();
                return Some((width, height));
            }
        }
    }
    None
}

fn extract_inner_content(svg: &str) -> Result<String, String> {
    // Find the end of opening <svg> tag
    let start = svg.find('>').ok_or("Invalid SVG: no opening tag found")? + 1;

    // Find the closing </svg> tag
    let end = svg
        .rfind("</svg>")
        .ok_or("Invalid SVG: no closing tag found")?;

    let body = svg[start..end].trim().to_string();
    Ok(body)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try multiple possible paths (run from workspace root or icon crate)
    let svgl_dir = if Path::new("apps/www/public/svgl").exists() {
        Path::new("apps/www/public/svgl")
    } else if Path::new("../../../apps/www/public/svgl").exists() {
        Path::new("../../../apps/www/public/svgl")
    } else {
        eprintln!("Error: SVGL directory not found");
        eprintln!("Tried: apps/www/public/svgl and ../../../apps/www/public/svgl");
        std::process::exit(1);
    };

    let output_path = if Path::new("crates/media/icon/data").exists() {
        Path::new("crates/media/icon/data/svgl.json")
    } else {
        Path::new("data/svgl.json")
    };

    if !svgl_dir.exists() {
        eprintln!("Error: SVGL directory not found at {}", svgl_dir.display());
        std::process::exit(1);
    }

    println!("🔍 Scanning SVGL icons...");

    let mut icons = HashMap::new();
    let mut samples = Vec::new();
    let mut count = 0;

    for entry in fs::read_dir(svgl_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("svg") {
            let filename = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid filename")?;

            let svg_content = fs::read_to_string(&path)?;

            match extract_svg_body(&svg_content) {
                Ok((body, width, height)) => {
                    icons.insert(
                        filename.to_string(),
                        IconData {
                            body,
                            width,
                            height,
                        },
                    );

                    if samples.len() < 6 {
                        samples.push(filename.to_string());
                    }

                    count += 1;
                    if count % 100 == 0 {
                        print!("\r📦 Processed: {}", count);
                    }
                }
                Err(e) => {
                    eprintln!("\n⚠️  Skipping {}: {}", filename, e);
                }
            }
        }
    }

    println!("\r✅ Processed: {} icons", count);

    let icon_pack = IconPack {
        prefix: "svgl".to_string(),
        info: IconPackInfo {
            name: "SVGL".to_string(),
            total: icons.len(),
            author: Author {
                name: "SVGL Contributors".to_string(),
                url: "https://svgl.app".to_string(),
            },
            license: License {
                title: "MIT".to_string(),
                spdx: "MIT".to_string(),
                url: "https://github.com/pheralb/svgl/blob/main/LICENSE".to_string(),
            },
            samples,
            height: 24,
            category: "Brand Logos".to_string(),
            tags: vec![
                "Brands".to_string(),
                "Logos".to_string(),
                "Companies".to_string(),
            ],
            palette: true,
        },
        last_modified: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        icons,
    };

    println!("💾 Writing to {}...", output_path.display());

    let json = serde_json::to_string_pretty(&icon_pack)?;
    fs::write(output_path, json)?;

    println!("✅ Generated svgl.json with {} icons", icon_pack.info.total);
    println!("📍 Location: {}", output_path.display());

    Ok(())
}
