
# Figlet Fonts Collection

This directory contains 440+ figlet-style fonts for ASCII art text rendering in CLI applications.

## Purpose

These fonts are used to render stylized text in terminal environments, creating ASCII art banners and decorative text output. They are commonly used in: -CLI tool banners and headers -Terminal-based applications -ASCII art generators -Text-based user interfaces

## Font Format

Each font file uses the `.dx` format, which is a custom format for figlet-style character definitions. The format includes: -Character width and height definitions -ASCII art patterns for each printable character -Spacing and alignment metadata

## Usage

The fonts can be accessed programmatically through the `dx-font` crate:
```rust
use dx_font::figlet;
// List all available fonts let fonts = figlet::list_fonts().unwrap();
println!("Available fonts: {}", fonts.len());
// Get the path to a specific font if let Some(path) = figlet::font_path("Banner") { println!("Banner font at: {:?}", path);
}
// Read font content let content = figlet::read_font("Banner").unwrap();
```

## Future Integration Plans

- Font Rendering Engine: Implement a text-to-ASCII-art renderer using these fonts
- Font Preview: Add CLI command to preview fonts with sample text
- Font Search: Enable searching fonts by style, width, or character support
- Custom Fonts: Support for user-defined custom fonts
- Web Integration: Generate ASCII art for web applications

## Font Sources

These fonts are collected from various open-source figlet font repositories and are provided for use in CLI applications. Each font maintains its original licensing terms.

## Contributing

To add new fonts: -Ensure the font is in `.dx` format -Place the font file in this directory -Run the test suite to verify integrity: `cargo test -p dx-font figlet`

## License

Individual fonts may have their own licenses. Please check the font file headers for specific licensing information.
