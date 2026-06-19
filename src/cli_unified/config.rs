//! Configuration management for media CLI
//!
//! Reads media CLI settings from a `dx` config file.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::cli_unified::config_format::{ConfigValue, parse_config_values};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaConfig {
    /// Base output directory for all downloads
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// Media-specific output directory
    #[serde(default)]
    pub media_dir: Option<PathBuf>,

    /// Icon-specific output directory
    #[serde(default)]
    pub icon_dir: Option<PathBuf>,

    /// Font-specific output directory
    #[serde(default)]
    pub font_dir: Option<PathBuf>,

    /// Archive output directory
    #[serde(default)]
    pub archive_dir: Option<PathBuf>,

    /// Cache directory for temporary files
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,

    /// Default media provider
    #[serde(default)]
    pub default_media_provider: Option<String>,

    /// Default font provider
    #[serde(default = "default_font_provider")]
    pub default_font_provider: String,

    /// Default font formats
    #[serde(default = "default_font_formats")]
    pub font_formats: Vec<String>,

    /// Default font subsets
    #[serde(default = "default_font_subsets")]
    pub font_subsets: Vec<String>,

    /// Auto-create directories
    #[serde(default = "default_true")]
    pub auto_create_dirs: bool,

    /// Organize downloads by date
    #[serde(default)]
    pub organize_by_date: bool,

    /// Organize downloads by type
    #[serde(default = "default_true")]
    pub organize_by_type: bool,
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("./downloads")
}

fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dx-media")
}

fn default_font_provider() -> String {
    "google".to_string()
}

fn default_font_formats() -> Vec<String> {
    vec!["ttf".to_string(), "woff2".to_string()]
}

fn default_font_subsets() -> Vec<String> {
    vec!["latin".to_string()]
}

fn default_true() -> bool {
    true
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            media_dir: None,
            icon_dir: None,
            font_dir: None,
            archive_dir: None,
            cache_dir: default_cache_dir(),
            default_media_provider: None,
            default_font_provider: default_font_provider(),
            font_formats: default_font_formats(),
            font_subsets: default_font_subsets(),
            auto_create_dirs: true,
            organize_by_date: false,
            organize_by_type: true,
        }
    }
}

impl MediaConfig {
    /// Load config from a dx file.
    pub fn load() -> Result<Self> {
        // Try current directory first
        if let Ok(config) = Self::load_from_path("dx") {
            return Ok(config);
        }

        // Try ~/.config/dx/dx
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("dx").join("dx");
            if let Ok(config) = Self::load_from_path(&config_path) {
                return Ok(config);
            }
        }

        // Try home directory
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join("dx");
            if let Ok(config) = Self::load_from_path(&config_path) {
                return Ok(config);
            }
        }

        // Return default if no config found
        Ok(Self::default())
    }

    fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let values = parse_config_values(&content)?;
        let mut config = Self::default();

        if let Some(base_dir) = value_string(&values, "base_dir") {
            config.output_dir = PathBuf::from(base_dir);
        }
        if let Some(auto) = value_bool(&values, "auto_create") {
            config.auto_create_dirs = auto;
        }
        if let Some(by_type) = value_bool(&values, "organize_by_type") {
            config.organize_by_type = by_type;
        }
        if let Some(by_date) = value_bool(&values, "organize_by_date") {
            config.organize_by_date = by_date;
        }

        if let Some(media_dir) = value_string(&values, "directories.media") {
            config.media_dir = Some(PathBuf::from(media_dir));
        }
        if let Some(icons) = value_string(&values, "directories.icons") {
            config.icon_dir = Some(PathBuf::from(icons));
        }
        if let Some(fonts) = value_string(&values, "directories.fonts") {
            config.font_dir = Some(PathBuf::from(fonts));
        }
        if let Some(archives) = value_string(&values, "directories.archives") {
            config.archive_dir = Some(PathBuf::from(archives));
        }
        if let Some(cache) = value_string(&values, "directories.cache") {
            config.cache_dir = PathBuf::from(cache);
        }

        if let Some(media_prov) = value_string(&values, "providers.default_media") {
            config.default_media_provider = Some(media_prov);
        }
        if let Some(font) = value_string(&values, "providers.default_font") {
            config.default_font_provider = font;
        }

        if let Some(formats) = value_string_list(&values, "fonts.formats") {
            config.font_formats = formats;
        }
        if let Some(subsets) = value_string_list(&values, "fonts.subsets") {
            config.font_subsets = subsets;
        }

        Ok(config)
    }

    /// Get output directory for media downloads
    pub fn get_media_dir(&self) -> PathBuf {
        let dir = if let Some(ref dir) = self.media_dir {
            self.output_dir.join(dir)
        } else if self.organize_by_type {
            self.output_dir.join("media")
        } else {
            self.output_dir.clone()
        };
        dir
    }

    /// Get output directory for icon downloads
    pub fn get_icon_dir(&self) -> PathBuf {
        let dir = if let Some(ref dir) = self.icon_dir {
            self.output_dir.join(dir)
        } else if self.organize_by_type {
            self.output_dir.join("icons")
        } else {
            self.output_dir.clone()
        };
        dir
    }

    /// Get output directory for font downloads
    pub fn get_font_dir(&self) -> PathBuf {
        let dir = if let Some(ref dir) = self.font_dir {
            self.output_dir.join(dir)
        } else if self.organize_by_type {
            self.output_dir.join("fonts")
        } else {
            self.output_dir.clone()
        };
        dir
    }

    /// Get output directory for archive operations
    pub fn get_archive_dir(&self) -> PathBuf {
        let dir = if let Some(ref dir) = self.archive_dir {
            self.output_dir.join(dir)
        } else if self.organize_by_type {
            self.output_dir.join("archives")
        } else {
            self.output_dir.clone()
        };
        dir
    }

    /// Ensure directory exists if auto_create_dirs is enabled
    pub fn ensure_dir(&self, path: &Path) -> Result<()> {
        if self.auto_create_dirs && !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }
}

fn value_string(values: &HashMap<String, ConfigValue>, suffix: &str) -> Option<String> {
    match value(values, suffix) {
        Some(ConfigValue::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn value_bool(values: &HashMap<String, ConfigValue>, suffix: &str) -> Option<bool> {
    match value(values, suffix) {
        Some(ConfigValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn value_string_list(values: &HashMap<String, ConfigValue>, suffix: &str) -> Option<Vec<String>> {
    match value(values, suffix) {
        Some(ConfigValue::StringList(values)) => Some(values.clone()),
        _ => None,
    }
}

fn value<'a>(values: &'a HashMap<String, ConfigValue>, suffix: &str) -> Option<&'a ConfigValue> {
    values
        .get(&format!("media.cli.{suffix}"))
        .or_else(|| values.get(&format!("cli.{suffix}")))
        .or_else(|| values.get(suffix))
}
