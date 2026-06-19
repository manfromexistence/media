#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# DX-MEDIA SEARCH SHOWCASE
# Interactive demonstration of all search modes
# ═══════════════════════════════════════════════════════════════════════════════

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DX_BIN="${SCRIPT_DIR}/../target/release/dx"
OUTPUT_DIR="${SCRIPT_DIR}/results"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Header
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║${NC}     ${BOLD}${CYAN}DX-MEDIA SEARCH SHOWCASE${NC}     ${MAGENTA}║${NC}"
echo -e "${MAGENTA}║${NC}     Interactive Demo of All Search Modes                        ${MAGENTA}║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if dx binary exists
if [ ! -f "$DX_BIN" ]; then
    echo -e "${RED}Error: dx binary not found at $DX_BIN${NC}"
    echo -e "${YELLOW}Please build the project first: cargo build --release${NC}"
    exit 1
fi

echo -e "${GREEN}✓ DX binary found${NC}"
echo -e "${BLUE}Output directory: ${OUTPUT_DIR}${NC}"
echo ""

# Function to run search and save results
run_search() {
    local name="$1"
    local description="$2"
    local query="$3"
    local args="$4"
    local output_file="$5"
    
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}${CYAN}[$name]${NC} $description"
    echo -e "${BLUE}Command:${NC} dx search \"$query\" $args --format json"
    echo ""
    
    # Run the search
    start_time=$(date +%s%3N)
    if $DX_BIN search $query $args --format json > "$output_file" 2>&1; then
        end_time=$(date +%s%3N)
        duration=$((end_time - start_time))
        
        # Parse results
        total=$(grep -o '"total_count":[0-9]*' "$output_file" | cut -d: -f2 || echo "0")
        returned=$(grep -o '"returned_count":[0-9]*' "$output_file" | cut -d: -f2 || echo "0")
        providers=$(grep -o '"providers_searched":\[[^]]*\]' "$output_file" | tr ',' '\n' | wc -l || echo "0")
        
        echo -e "${GREEN}✓ Success!${NC}"
        echo -e "  ${CYAN}Results:${NC} $returned returned / $total total available"
        echo -e "  ${CYAN}Providers searched:${NC} $providers"
        echo -e "  ${CYAN}Time:${NC} ${duration}ms"
        echo -e "  ${CYAN}Output:${NC} $output_file"
    else
        echo -e "${RED}✗ Search failed${NC}"
    fi
    echo ""
}

# Function for interactive search
interactive_search() {
    echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${MAGENTA}║${NC}     ${BOLD}INTERACTIVE SEARCH MODE${NC}                                      ${MAGENTA}║${NC}"
    echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    read -p "Enter search query: " query
    if [ -z "$query" ]; then
        query="nature"
    fi
    
    echo ""
    echo "Select search mode:"
    echo "  1) Single provider (NASA)"
    echo "  2) Multiple providers (NASA, Wikimedia, Met)"
    echo "  3) All providers (concurrent)"
    echo "  4) All providers + scrapers (--all)"
    echo ""
    read -p "Choice [1-4]: " choice
    
    case $choice in
        1) args="-P nasa -n 10" ;;
        2) args="-P nasa,wikimedia,met -n 5" ;;
        3) args="-n 5" ;;
        4) args="--all -n 5" ;;
        *) args="-n 5" ;;
    esac
    
    output_file="${OUTPUT_DIR}/interactive_$(date +%Y%m%d_%H%M%S).json"
    run_search "INTERACTIVE" "Custom search" "$query" "$args" "$output_file"
    
    read -p "View JSON output? [y/N]: " view
    if [[ "$view" =~ ^[Yy]$ ]]; then
        cat "$output_file" | head -100
        echo -e "\n${YELLOW}... (truncated, see full file at $output_file)${NC}"
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# SHOWCASE DEMOS
# ═══════════════════════════════════════════════════════════════════════════════

echo -e "${BOLD}${MAGENTA}DEMO 1: SINGLE PROVIDER SEARCH${NC}"
echo -e "${BLUE}Search a specific provider for targeted results${NC}"
run_search "NASA" "Space & astronomy images from NASA" \
    "mars rover" "-P nasa -n 10" \
    "${OUTPUT_DIR}/01_single_provider_nasa.json"

echo -e "${BOLD}${MAGENTA}DEMO 2: MULTI-PROVIDER SEARCH${NC}"
echo -e "${BLUE}Search multiple providers simultaneously${NC}"
run_search "MULTI" "Multiple museum collections" \
    "landscape painting" "-P met,rijksmuseum,cleveland -n 5" \
    "${OUTPUT_DIR}/02_multi_provider_museums.json"

echo -e "${BOLD}${MAGENTA}DEMO 3: ALL AVAILABLE PROVIDERS${NC}"
echo -e "${BLUE}Search all 10 free providers concurrently (no API keys needed)${NC}"
run_search "ALL-FREE" "All free providers (10 sources)" \
    "sunset mountains" "-n 5" \
    "${OUTPUT_DIR}/03_all_free_providers.json"

echo -e "${BOLD}${MAGENTA}DEMO 4: UNIFIED SEARCH (--all)${NC}"
echo -e "${BLUE}Search ALL providers + scrapers for maximum coverage${NC}"
run_search "UNIFIED" "All providers + scrapers (12+ sources)" \
    "ocean waves" "--all -n 5" \
    "${OUTPUT_DIR}/04_unified_all_sources.json"

echo -e "${BOLD}${MAGENTA}DEMO 5: MEDIA TYPE FILTER${NC}"
echo -e "${BLUE}Filter by specific media type${NC}"
run_search "AUDIO" "Audio files only" \
    "ambient music" "-t audio -n 5" \
    "${OUTPUT_DIR}/05_audio_search.json"

echo -e "${BOLD}${MAGENTA}DEMO 6: HIGH-VOLUME SEARCH${NC}"
echo -e "${BLUE}Large result set from cultural heritage sources${NC}"
run_search "HERITAGE" "European cultural heritage" \
    "medieval art" "-P europeana -n 20" \
    "${OUTPUT_DIR}/06_europeana_heritage.json"

echo -e "${BOLD}${MAGENTA}DEMO 7: 3D MODELS${NC}"
echo -e "${BLUE}Search for 3D models and textures${NC}"
run_search "3D" "3D models and HDRIs from Poly Haven" \
    "forest" "-P polyhaven -n 10" \
    "${OUTPUT_DIR}/07_polyhaven_3d.json"

# ═══════════════════════════════════════════════════════════════════════════════
# GENERATE SUMMARY
# ═══════════════════════════════════════════════════════════════════════════════

echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║${NC}     ${BOLD}GENERATING SUMMARY${NC}                                            ${MAGENTA}║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Create summary JSON
SUMMARY_FILE="${OUTPUT_DIR}/search_summary.json"
cat > "$SUMMARY_FILE" << 'EOF'
{
  "showcase_info": {
    "title": "DX-Media Search Showcase",
    "generated_at": "TIMESTAMP",
    "dx_version": "0.1.0",
    "description": "Demonstrates all search modes and capabilities"
  },
  "search_modes": [
    {
      "mode": "single_provider",
      "description": "Search a specific provider",
      "example": "dx search \"query\" -P nasa -n 10",
      "use_case": "When you need images from a specific source"
    },
    {
      "mode": "multi_provider",
      "description": "Search multiple specific providers",
      "example": "dx search \"query\" -P met,rijksmuseum,cleveland -n 5",
      "use_case": "When you need results from curated sources"
    },
    {
      "mode": "all_free_providers",
      "description": "Search all available free providers",
      "example": "dx search \"query\" -n 5",
      "use_case": "Default mode - searches all 10 free providers"
    },
    {
      "mode": "unified_all",
      "description": "Search all providers AND scrapers",
      "example": "dx search \"query\" --all -n 5",
      "use_case": "Maximum coverage from 12+ sources"
    }
  ],
  "available_providers": {
    "free_no_api_key": [
      "openverse (700M+ images)",
      "wikimedia (92M+ files)",
      "europeana (50M+ items)",
      "loc (3M+ images)",
      "rijksmuseum (700K+ artworks)",
      "met (500K+ artworks)",
      "nasa (140K+ images)",
      "cleveland (61K+ artworks)",
      "polyhaven (3.7K+ 3D/HDRIs)",
      "picsum (unlimited placeholders)"
    ],
    "premium_with_api_key": [
      "unsplash",
      "pexels",
      "pixabay",
      "freesound",
      "giphy",
      "smithsonian",
      "dpla"
    ]
  },
  "output_formats": [
    "text (human-readable)",
    "json (pretty-printed)",
    "json-compact (single line)",
    "tsv (tab-separated)"
  ],
  "result_files": [
    "01_single_provider_nasa.json",
    "02_multi_provider_museums.json",
    "03_all_free_providers.json",
    "04_unified_all_sources.json",
    "05_audio_search.json",
    "06_europeana_heritage.json",
    "07_polyhaven_3d.json"
  ]
}
EOF

# Replace timestamp
sed -i "s/TIMESTAMP/$(date -Iseconds)/" "$SUMMARY_FILE" 2>/dev/null || \
    sed -i '' "s/TIMESTAMP/$(date -Iseconds)/" "$SUMMARY_FILE" 2>/dev/null || true

echo -e "${GREEN}✓ Summary saved to: ${SUMMARY_FILE}${NC}"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# RESULTS OVERVIEW
# ═══════════════════════════════════════════════════════════════════════════════

echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║${NC}     ${BOLD}RESULTS OVERVIEW${NC}                                               ${MAGENTA}║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BOLD}Generated Files:${NC}"
ls -la "$OUTPUT_DIR"/*.json 2>/dev/null | awk '{print "  " $NF " (" $5 " bytes)"}' | sed "s|$OUTPUT_DIR/||"
echo ""

# Count total results
total_assets=0
for f in "$OUTPUT_DIR"/*.json; do
    if [ -f "$f" ]; then
        count=$(grep -o '"returned_count":[0-9]*' "$f" 2>/dev/null | cut -d: -f2 || echo "0")
        total_assets=$((total_assets + count))
    fi
done

echo -e "${GREEN}${BOLD}Total assets found across all demos: ${total_assets}${NC}"
echo ""

# Interactive option
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
read -p "Run interactive search? [y/N]: " interactive
if [[ "$interactive" =~ ^[Yy]$ ]]; then
    interactive_search
fi

echo ""
echo -e "${GREEN}${BOLD}Showcase complete!${NC}"
echo -e "${BLUE}All results saved to: ${OUTPUT_DIR}${NC}"
echo ""
