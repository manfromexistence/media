
# DX-Media Playground

This folder contains real media assets for testing all 60 dx-media tools interactively.

## Directory Structure

@tree:playground[]

## Test Assets

### Images

- `flower.jpg`
- Flower photo (93KB)
- `white_flower.jpg`
- White flower photo (55KB)
- `landscape.jpg`
- Landscape photo (267KB)
- `sample.jpg`
- Sample image (110KB)
- `wikimedia-2112596.jpg`
- Downloaded from Wikimedia (1.6MB)

### Audio

- `piano.mp3`
- Short piano sample (247KB)
- `calm_piano.mp3`
- Longer piano piece (3.3MB)

### Video

- `sample.mp4`
- Sample video (574KB)

### Documents

- `test.md`
- Markdown test document
- `test.html`
- HTML test document
- `test.txt`
- Plain text test document

### Provider Downloads

- `providers/openverse/openverse-*.jpg`
- Downloaded from Openverse
- `providers/wikimedia/wikimedia-*.jpg`
- Downloaded from Wikimedia

## Running Tests

All 60 tools are tested using real assets. Run the integration tests with:
```bash


# Run all tests (593 tests)


cargo test


# Run tool integration tests


cargo test --test test_all_tools


# Run playground integration tests (downloads from providers)


cargo test --test playground_integration


# Run with output


cargo test --test playground_integration -- --nocapture ```


## CLI Usage Examples



### Basic Search (Single Provider)


```bash
./target/release/dx search "nature" --providers openverse -n 5 ./target/release/dx search "flower" --providers wikimedia -n 5 --format json ```

### 🚀 Unified Search (ALL Providers + Scrapers)

The `--all` flag enables concurrent search across ALL 10 providers + 2 scrapers simultaneously using Rust's async/tokio:
```bash


# Search ALL sources concurrently - returns 40-50+ results in ~4-5 seconds


./target/release/dx search "sunset" --all -n 5 --format json


# With type filter (image, audio, video, document)


./target/release/dx search "ocean" --all --type image --format json


# Compact JSON output


./target/release/dx search "mountains" --all --format json-compact ```
Available Providers (10): cleveland, dpla, europeana, freesound, giphy, loc, met, nasa, openverse, pexels, picsum, pixabay, polyhaven, rijksmuseum, smithsonian, unsplash, wikimedia Available Scrapers (2): Flickr, NASA Gallery


### Download media


```bash
./target/release/dx search "flower" --providers wikimedia -n 1 --download -o playground/assets/images/ ```

### List available providers

```bash
./target/release/dx providers ```


## Search Showcase (Interactive)


Run the interactive search showcase script to test all search modes:
```bash

# Make executable and run

chmod +x playground/search_showcase.sh ./playground/search_showcase.sh

# Or run directly with bash

bash playground/search_showcase.sh ```
This generates JSON result files in `playground/results/`: -`01_basic_search.json` - Single provider search -`02_unified_search.json` - All providers + scrapers (concurrent) -`03_type_filter.json` - Image type filter -`04_provider_filter.json` - Specific provider search -`05_orientation_filter.json` - Portrait/landscape filter -`06_color_filter.json` - Color theme filter

### Results Directory Structure

@tree:playground/results[]

## Test Results

All 593 tests pass: -158 library unit tests -35 archive tests -49 audio tests -44 document tests -43 image tests -72 integration tests (real assets) -48 utility tests -55 video tests -72 doc tests
