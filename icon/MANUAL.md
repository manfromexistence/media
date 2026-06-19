# DX Icon Command Manual

Complete guide to using the `icon` CLI for searching and exporting SVG icons.

## Installation

Build and install the icon command:

```bash
cd crates/media/icon
cargo build --release --bin icon
cargo install --path . --bin icon
```

Or run directly without installing:

```bash
cargo run --release --bin icon -- <command>
```

## Prerequisites

Before using the icon command, you need to build the search index:

```bash
cargo run --release --bin build_index
```

This creates an optimized FST-based index in the `index/` directory (~3MB compressed).

## Commands

### 1. Search Icons

Search for icons by name with fuzzy matching support.

```bash
icon search <query> [--limit N]
icon s <query> [--limit N]          # Short form
```

**Options:**
- `--limit N` - Maximum number of results (default: 10)

**Examples:**

```bash
# Search for home icons
icon search home

# Search with custom limit
icon search arrow --limit 20

# Short form
icon s menu --limit 5
```

**Output:**
```
Found 10 results (0.02s):

  1. home (lucide)
  2. home (solar)
  3. home-2 (solar)
  4. home-smile (solar)
  5. home-angle (solar)
  ...
```

### 2. Export Icons

Export matching icons as SVG files to a directory.

```bash
icon export <query> <output_dir> [--limit N] [--pack PACK]
icon e <query> <output_dir> [--limit N] [--pack PACK]    # Short form
```

**Options:**
- `--limit N` - Maximum number of icons to export (default: 10)
- `--pack PACK` - Filter by specific icon pack (e.g., lucide, solar, heroicons)

**Examples:**

```bash
# Export search icons to ./icons directory
icon export search ./icons

# Export only from lucide pack
icon export arrow ./icons --pack lucide --limit 5

# Export to custom directory
icon e menu ~/Desktop/menu-icons --limit 3
```

**Output:**
```
✓ lucide_search.svg
✓ solar_search.svg
✓ heroicons_search.svg

Exported 3 icons (0.05s)
```

### 3. Desktop Export

Export specific icons directly to the DX desktop app assets directory.

```bash
icon desktop <icon_specs...>
icon d <icon_specs...>              # Short form
```

**Icon Spec Format:** `name:pack`

**Examples:**

```bash
# Export single icon
icon desktop search:lucide

# Export multiple icons
icon desktop search:lucide home:solar menu:heroicons

# Short form
icon d arrow:lucide close:solar settings:feather
```

**Output:**
```
✓ search (lucide)
✓ home (solar)
✓ menu (heroicons)

Exported 3 icons (0.03s)
```

Icons are saved to: `apps/desktop/assets/icons/`

### 4. List Icon Packs

Display all available icon packs in the database.

```bash
icon packs
icon p                              # Short form
```

**Output:**
```
Available packs (250):

  academicons
  akar-icons
  ant-design
  arcticons
  basil
  bi
  ...
  lucide
  solar
  heroicons
  ...
```

### 5. Help

Show usage information and available commands.

```bash
icon help
icon -h
icon --help
```

### 6. Version

Display the icon CLI version.

```bash
icon version
icon -v
icon --version
```

## Icon Packs

The icon command includes 250+ icon packs with 100,000+ icons:

**Popular Packs:**
- `lucide` - Modern, clean icons (1000+ icons)
- `solar` - Bold, linear icons (7000+ icons)
- `heroicons` - Tailwind UI icons (500+ icons)
- `feather` - Simple, beautiful icons (280+ icons)
- `material-symbols` - Google Material icons (10,000+ icons)
- `fa-solid` / `fa-brands` - Font Awesome icons (2000+ icons)
- `tabler` - Customizable icons (4000+ icons)
- `carbon` - IBM Carbon icons (2000+ icons)
- `octicon` - GitHub icons (300+ icons)
- `simple-icons` - Brand icons (2500+ icons)

**Specialized Packs:**
- `logos` - Technology logos
- `devicon` - Developer tool icons
- `skill-icons` - Programming language icons
- `cryptocurrency` - Crypto currency icons
- `flag` / `circle-flags` - Country flags
- `emojione` / `twemoji` - Emoji sets
- `game-icons` - Gaming icons (4000+ icons)
- `medical-icon` - Healthcare icons
- `weather-icons` - Weather symbols

## Search Features

### Prefix Search
Fast FST-based prefix matching with <0.1ms latency:

```bash
icon s home      # Matches: home, home-2, home-smile, etc.
```

### Fuzzy Matching
Typo-tolerant search using Levenshtein distance:

```bash
icon s serch     # Finds: search
icon s arow      # Finds: arrow
```

### Multi-Strategy
Combines exact, prefix, and fuzzy matching for best results.

## Performance

- **Index size:** ~3MB (compressed with LZ4)
- **Load time:** <50ms (memory-mapped files)
- **Search latency:** <0.1ms (cached), <1ms (uncached)
- **Memory usage:** ~5MB
- **Icons supported:** 100,000+

## Architecture

```
┌─────────────────────────────────────────────────┐
│  TIER 1: FST Index (~1MB)                       │
│  - Finite State Transducer for prefix search   │
│  - O(k) lookup where k = query length          │
└─────────────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────────────┐
│  TIER 2: rkyv Metadata (~2MB)                   │
│  - Zero-copy archived data                      │
│  - Direct memory access, no parsing             │
└─────────────────────────────────────────────────┘
```

## File Locations

**Index Directory:**
- `index/` (workspace root)
- `crates/media/icon/index/` (crate directory)

**Data Directory:**
- `data/` (workspace root)
- `crates/media/icon/data/` (crate directory)

**Desktop Assets:**
- `apps/desktop/assets/icons/`

## Troubleshooting

### "Index not found" Error

Build the index first:

```bash
cd crates/media/icon
cargo run --release --bin build_index
```

### "Pack not found" Error

List available packs:

```bash
icon packs
```

Ensure the pack name matches exactly (case-sensitive).

### "Data directory not found" Error

The icon command looks for data in multiple locations:
- `data/`
- `crates/media/icon/data/`
- Relative to executable

Ensure you're running from the workspace root or the data directory exists.

### No Results

Try:
- Broader search terms
- Remove `--pack` filter
- Increase `--limit`
- Check spelling (fuzzy matching helps but has limits)

## Advanced Usage

### Batch Export

Export multiple icon sets:

```bash
# Export all arrow icons from lucide
icon e arrow ./icons/arrows --pack lucide --limit 50

# Export all social media icons
icon e facebook ./icons/social --pack simple-icons --limit 100
```

### Integration with Scripts

Use in shell scripts:

```bash
#!/bin/bash
# Export icons for a design system

ICONS=("home:lucide" "search:lucide" "menu:lucide" "close:lucide")

for icon in "${ICONS[@]}"; do
    icon d "$icon"
done
```

### Pipeline Usage

Combine with other tools:

```bash
# Search and count results
icon s home | grep -c "^  [0-9]"

# Export and optimize SVGs
icon e arrow ./icons --limit 10
svgo ./icons/*.svg
```

## Tips

1. **Use short forms** - `s`, `e`, `d`, `p` for faster typing
2. **Start broad** - Search with general terms, then filter with `--pack`
3. **Check packs first** - Run `icon packs` to see available options
4. **Desktop workflow** - Use `icon d` for quick asset integration
5. **Limit wisely** - Default limit (10) is usually sufficient; increase for browsing

## Related Commands

- `build_index` - Build the search index
- `search_cli` - Interactive search CLI
- `generate_svgl` - Generate SVGL icon pack data

## License

MIT
