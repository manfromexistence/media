
# dx-font Playground

This folder contains example scripts to test the dx-font library functionality.

## Examples

### Search Fonts

```bash
cargo run --example search_fonts ```
This example demonstrates: -Basic font search -Searching for specific fonts (roboto, inter, mono) -Getting font statistics -Provider health checks


### Download Fonts


```bash
cargo run --example download_fonts ```
This example demonstrates: -Downloading fonts from Google Fonts -Downloading fonts from Fontsource CDN -Downloading multiple font weights -Search-then-download workflow

## Output

Downloaded fonts will be saved to `./playground/downloaded_fonts/`

## JSON Response Examples

### Search Response

```json
{ "fonts": [ { "id": "roboto", "name": "Roboto", "provider": "GoogleFonts", "category": "SansSerif", "variant_count": 12, "license": "OFL", "preview_url": "https://fonts.google.com/specimen/Roboto", "download_url": "https://gwfh.mranftl.com/api/fonts/roboto?download=zip"
}
], "total": 1, "query": "roboto", "providers_searched": ["Google Fonts", "Bunny Fonts", "Fontsource", "FontShare"]
}
```

### Statistics Response

```json
{ "total_fonts": 5000, "providers_count": 4, "providers": ["Google Fonts", "Bunny Fonts", "Fontsource", "FontShare"], "serif_count": 500, "sans_serif_count": 1200, "display_count": 800, "handwriting_count": 400, "monospace_count": 100, "uncategorized_count": 2000 }
```
