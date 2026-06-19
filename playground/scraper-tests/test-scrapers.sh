#!/bin/bash

# Test Script for DX Media Scrapers - Testing 228 Scraping Targets
# This script tests a sample from each category

RESULTS_FILE="playground/scraper-tests/results.txt"
> $RESULTS_FILE

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║        DX MEDIA SCRAPER TEST SUITE - 228 Targets                ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

test_url() {
    local name="$1"
    local url="$2"
    local category="$3"
    
    echo -n "Testing $name ($category)... "
    
    START_TIME=$(date +%s%N)
    result=$(timeout 30 cargo run --release --quiet -- scrape "$url" --count 3 --dry-run -f json-compact 2>&1)
    exit_code=$?
    END_TIME=$(date +%s%N)
    
    DURATION_MS=$(( (END_TIME - START_TIME) / 1000000 ))
    
    if [ $exit_code -eq 0 ]; then
        assets=$(echo "$result" | grep -o '"assets_found":[0-9]*' | grep -o '[0-9]*' | head -1)
        if [ -n "$assets" ] && [ "$assets" -gt 0 ]; then
            echo "✓ ${assets} assets found (${DURATION_MS}ms)"
            echo "PASS: $name | $category | $assets assets | ${DURATION_MS}ms" >> $RESULTS_FILE
        else
            echo "⚠ No assets (${DURATION_MS}ms)"
            echo "WARN: $name | $category | 0 assets | ${DURATION_MS}ms" >> $RESULTS_FILE
        fi
    else
        echo "✗ Failed (${DURATION_MS}ms)"
        echo "FAIL: $name | $category | error | ${DURATION_MS}ms" >> $RESULTS_FILE
    fi
}

echo "═══════════════════════════════════════════════════════════════════"
echo "                      IMAGES (Stock Photos)"
echo "═══════════════════════════════════════════════════════════════════"
test_url "StockSnap" "https://stocksnap.io/search/nature" "Images"
test_url "Burst (Shopify)" "https://burst.shopify.com/photos/search?q=business" "Images"
test_url "Reshot" "https://www.reshot.com/search/nature" "Images"
test_url "PicJumbo" "https://picjumbo.com/?s=landscape" "Images"
test_url "Pixnio" "https://pixnio.com/search?q=flowers" "Images"
test_url "FreeImages" "https://www.freeimages.com/search/sunset" "Images"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                      AUDIO (Sound Effects)"
echo "═══════════════════════════════════════════════════════════════════"
test_url "FreeSound" "https://freesound.org/search/?q=nature" "Audio"
test_url "Mixkit" "https://mixkit.co/free-stock-music/" "Audio"
test_url "ZapSplat" "https://www.zapsplat.com/sound-effect-category/nature/" "Audio"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                      VIDEO (Stock Footage)"
echo "═══════════════════════════════════════════════════════════════════"
test_url "Coverr" "https://coverr.co/search?q=nature" "Video"
test_url "Mazwai" "https://mazwai.com/#/grid" "Video"
test_url "Videvo" "https://www.videvo.net/search/nature/" "Video"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                      3D MODELS"
echo "═══════════════════════════════════════════════════════════════════"
test_url "Sketchfab" "https://sketchfab.com/search?q=tree&type=models" "3D Models"
test_url "TurboSquid Free" "https://www.turbosquid.com/Search/3D-Models/free" "3D Models"
test_url "Clara.io" "https://clara.io/library" "3D Models"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                      TEXTURES"
echo "═══════════════════════════════════════════════════════════════════"
test_url "Textures.com" "https://www.textures.com/" "Textures"
test_url "TextureLib" "https://texturelib.com/" "Textures"
test_url "Poliigon Free" "https://www.poliigon.com/textures/free" "Textures"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                      VECTORS & ICONS"
echo "═══════════════════════════════════════════════════════════════════"
test_url "Flaticon" "https://www.flaticon.com/search?word=arrow" "Vectors"
test_url "SVG Repo" "https://www.svgrepo.com/collections/monocolor/" "Vectors"
test_url "IconFinder" "https://www.iconfinder.com/search?q=home&price=free" "Vectors"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                      GAME ASSETS"
echo "═══════════════════════════════════════════════════════════════════"
test_url "OpenGameArt" "https://opengameart.org/content/search" "Game Assets"
test_url "itch.io Assets" "https://itch.io/game-assets/free" "Game Assets"
test_url "Kenney" "https://kenney.nl/assets" "Game Assets"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                      SUMMARY"
echo "═══════════════════════════════════════════════════════════════════"
PASS_COUNT=$(grep -c "^PASS:" $RESULTS_FILE)
WARN_COUNT=$(grep -c "^WARN:" $RESULTS_FILE)
FAIL_COUNT=$(grep -c "^FAIL:" $RESULTS_FILE)
TOTAL=$((PASS_COUNT + WARN_COUNT + FAIL_COUNT))

echo ""
echo "Results: $PASS_COUNT passed, $WARN_COUNT warnings, $FAIL_COUNT failed (out of $TOTAL tested)"
echo ""
echo "Full results saved to: $RESULTS_FILE"
