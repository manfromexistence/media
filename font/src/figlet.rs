//! Figlet Fonts Module
//!
//! This module provides access to the figlet font collection for ASCII art text rendering.
//! These fonts are used by CLI tools to render stylized text in terminal environments.
//!
//! ## Font Format
//!
//! The fonts are stored in `.dx` format, which is a custom format for figlet-style fonts.
//! Each font file contains character definitions for ASCII art rendering.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use dx_font::figlet;
//!
//! // Get the path to the figlet fonts directory
//! let fonts_dir = figlet::fonts_dir();
//!
//! // List available fonts
//! let fonts = figlet::list_fonts().unwrap();
//! println!("Available fonts: {:?}", fonts);
//! ```

use std::fs;
use std::io;
use std::path::PathBuf;

/// Returns the path to the figlet fonts directory.
///
/// This directory contains all the `.dx` font files for ASCII art rendering.
pub fn fonts_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("figlet")
}

/// Lists all available figlet font names.
///
/// Returns a vector of font names (without the `.dx` extension).
pub fn list_fonts() -> io::Result<Vec<String>> {
    let dir = fonts_dir();
    let mut fonts = Vec::new();

    if dir.exists() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "dx")
                && let Some(stem) = path.file_stem()
                && let Some(name) = stem.to_str()
            {
                fonts.push(name.to_string());
            }
        }
    }

    fonts.sort();
    Ok(fonts)
}

/// Returns the path to a specific font file.
///
/// # Arguments
///
/// * `name` - The font name (without the `.dx` extension)
///
/// # Returns
///
/// The path to the font file, or `None` if the font doesn't exist.
pub fn font_path(name: &str) -> Option<PathBuf> {
    let path = fonts_dir().join(format!("{}.dx", name));
    if path.exists() { Some(path) } else { None }
}

/// Reads the content of a font file.
///
/// # Arguments
///
/// * `name` - The font name (without the `.dx` extension)
///
/// # Returns
///
/// The font file content as bytes, or an error if the font doesn't exist.
pub fn read_font(name: &str) -> io::Result<Vec<u8>> {
    let path = font_path(name).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Font '{}' not found", name),
        )
    })?;
    fs::read(path)
}

/// Returns the total number of available fonts.
pub fn font_count() -> io::Result<usize> {
    list_fonts().map(|fonts| fonts.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fonts_dir_exists() {
        let dir = fonts_dir();
        assert!(
            dir.exists(),
            "Figlet fonts directory should exist at {:?}",
            dir
        );
    }

    #[test]
    fn test_list_fonts_not_empty() {
        let fonts = list_fonts().expect("Should be able to list fonts");
        assert!(!fonts.is_empty(), "Should have at least one font");
    }

    #[test]
    fn test_font_count() {
        let count = font_count().expect("Should be able to count fonts");
        // We expect 440 fonts based on the migration
        assert!(
            count >= 400,
            "Should have at least 400 fonts, got {}",
            count
        );
    }

    #[test]
    fn test_font_path_exists() {
        let fonts = list_fonts().expect("Should be able to list fonts");
        if let Some(first_font) = fonts.first() {
            let path = font_path(first_font);
            assert!(
                path.is_some(),
                "Font path should exist for '{}'",
                first_font
            );
        }
    }

    #[test]
    fn test_read_font() {
        let fonts = list_fonts().expect("Should be able to list fonts");
        if let Some(first_font) = fonts.first() {
            let content = read_font(first_font);
            assert!(
                content.is_ok(),
                "Should be able to read font '{}'",
                first_font
            );
            assert!(
                !content.unwrap().is_empty(),
                "Font content should not be empty"
            );
        }
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    /// Generate a random font name from the available fonts
    fn arb_font_name() -> impl Strategy<Value = String> {
        let fonts = list_fonts().unwrap_or_default();
        if fonts.is_empty() {
            Just("default".to_string()).boxed()
        } else {
            proptest::sample::select(fonts).boxed()
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: dx-style-extension-enhancements, Property 1: Font Migration Integrity
        /// *For any* font file in the figlet directory, reading the file SHALL return
        /// non-empty content that is valid UTF-8 or binary data.
        /// **Validates: Requirements 1.4**
        #[test]
        fn prop_font_migration_integrity(font_name in arb_font_name()) {
            // Skip if no fonts available
            if font_name == "default" {
                return Ok(());
            }

            // Verify font file exists
            let path = font_path(&font_name);
            prop_assert!(path.is_some(), "Font '{}' should have a valid path", font_name);

            // Verify font content is readable
            let content = read_font(&font_name);
            prop_assert!(content.is_ok(), "Font '{}' should be readable", font_name);

            // Verify content is not empty
            let bytes = content.unwrap();
            prop_assert!(!bytes.is_empty(), "Font '{}' should have non-empty content", font_name);

            // Verify file has .dx extension
            let path = path.unwrap();
            prop_assert!(
                path.extension().is_some_and(|ext| ext == "dx"),
                "Font file should have .dx extension"
            );
        }

        /// Property: All font files are consistently named
        /// *For any* font in the collection, the filename should match the expected pattern.
        #[test]
        fn prop_font_naming_consistency(font_name in arb_font_name()) {
            if font_name == "default" {
                return Ok(());
            }

            // Font names should not be empty
            prop_assert!(!font_name.is_empty(), "Font name should not be empty");

            // Font names should not contain path separators
            prop_assert!(
                !font_name.contains('/') && !font_name.contains('\\'),
                "Font name should not contain path separators"
            );
        }
    }
}
