# DX Media

Universal media processing toolkit built in Rust with source-aware providers,
native utilities, and CLI/API tool metadata.

## Status

Receipt and provenance hardening is in progress. Use
`media --format json tools list` for source kind, readiness, route-local
receipt status, type-validation evidence, feature gates, external
dependencies, and credential requirements for each declared tool.

Some tools are implemented locally, some are feature-gated or
external-dependency backed, some require credentials, and some are
declared-only until wired. The registry is the authoritative catalog.

## Tool Catalog

Each listed tool reports whether it is local-only, provider-backed,
direct-url, fixture-backed, requires credentials, feature-gated,
external-dependency backed, or declared-only.

Verified local CLI/API routes currently include provider search, direct URL
download, native image conversion/resizing/compression/palette extraction,
SVG favicon generation when `image-svg` is enabled, native archive
zip/unzip/list, markdown-to-HTML and text extraction when `document-core` is
enabled, audio conversion when FFmpeg is available, and utility
hash/base64/duplicate/checksum/JSON/YAML tools.

Declared video tools and most FFmpeg-backed audio tools are intentionally
reported as external-dependency or declared-only by the registry until their
extended CLI routes produce output receipts.

## Installation

```toml
[dependencies]
dx-media = "1.0"
```

## Feature Flags

```toml
# Core features enabled by default.
default = ["cli", "archive-core", "utility-core"]

# Optional features.
image-core = []       # Native bitmap processing
image-svg = []        # SVG support; implies image-core
audio-core = []       # Native audio parsing helpers
document-core = []    # Native PDF/markdown document helpers
```

## Quick Start

### Generate Favicons from SVG

```rust
use dx_media::tools::image::svg::generate_web_icons;

generate_web_icons("logo.svg", "public/icons")?;
```

### Create Archive

```rust
use dx_media::tools::archive::create_zip;

create_zip(&["file1.txt", "file2.txt"], "archive.zip")?;
```

## Dependencies

### Optional

- **FFmpeg/FFprobe** - Required by FFmpeg-backed audio/video tools.
- **ImageMagick** - Required by compatibility image APIs such as `tools::image::converter::convert`; unified native image CLI routes use `image-core` instead.
- **Ghostscript** - Required by declared PDF operations that are not yet wired.
- **Tesseract** - Required by declared OCR tooling that is not yet wired.
- **wkhtmltopdf** - Required by declared HTML-to-PDF tooling that is not yet wired.
- **pdftotext/xpdf/Tika/antiword/docx2txt/LibreOffice** - Used by
  `document.extract-text` for PDF and Office-style inputs.

## Testing

```bash
# Fast source-honesty checks.
cargo test -p dx-media --test tool_receipt_tests -j1
cargo test -p dx-media --test tool_listing_cli_tests -j1

# Broader local verification when time and dependencies are available.
cargo test -p dx-media --all-features -j1
```

## Examples

```bash
# Generate favicons.
cargo run --example generate_favicons --features image-svg

# Convert logo with native SVG support.
cargo run --example convert_logo_native --features image-svg
```

## Architecture

- **Native Rust** - Used for verified local image, archive, document, and utility routes.
- **FFmpeg Integration** - Used by dependency-backed audio/video APIs where wired.
- **Async/Parallel** - Tokio + Rayon for bounded concurrent work.
- **Type-Safe** - Strong typing with explicit error handling.
- **Receipt-Oriented** - Tool outputs expose source kind, provenance where available, and type-validation metadata.
- **Source-Honest Providers** - Provider URLs distinguish direct files, preview derivatives, manifests, and landing pages when that evidence is available.

## Performance

- **SVG Rendering**: Native resvg when `image-svg` is enabled.
- **Image Processing**: Native Rust `image` crate when `image-core` is enabled.
- **Archive Operations**: Native zip support when `archive-core` is enabled.
- **Parallel Processing**: Rayon for selected multi-file operations.

## License

MIT OR Apache-2.0
