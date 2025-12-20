#!/bin/bash
# Build all mdhavers examples
# Usage: ./build_all.sh [--clean] [--category NAME]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MDHAVERS="${SCRIPT_DIR}/../target/release/mdhavers"
OUTPUT_DIR="${SCRIPT_DIR}/outputs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
DIM='\033[2m'
NC='\033[0m'

# Check if mdhavers compiler exists
if [[ ! -f "$MDHAVERS" ]]; then
    echo -e "${YELLOW}Building mdhavers compiler first...${NC}"
    (cd "${SCRIPT_DIR}/.." && cargo build --release)
fi

# Clean mode
if [[ "$1" == "--clean" ]]; then
    echo -e "${YELLOW}Cleaning outputs directory...${NC}"
    rm -rf "$OUTPUT_DIR"
    echo -e "${GREEN}Clean complete!${NC}"
    exit 0
fi

# Category filter
CATEGORY_FILTER=""
if [[ "$1" == "--category" && -n "$2" ]]; then
    CATEGORY_FILTER="$2"
    echo -e "${CYAN}Building only: ${CATEGORY_FILTER}${NC}"
fi

echo -e "${CYAN}"
echo "=============================================="
echo "       mdhavers Example Builder"
echo "       \"Braw code fer braw fowk!\""
echo "=============================================="
echo -e "${NC}"

# Create output directory
mkdir -p "$OUTPUT_DIR"

BUILT=0
FAILED=0
SKIPPED=0

# Categories to build (excluding lib which contains modules, not executables)
CATEGORIES=(basics features algorithms games stdlib scottish builtins testing apps)

# Build top-level examples (e.g., examples/hello.braw)
if [[ -z "$CATEGORY_FILTER" || "$CATEGORY_FILTER" == "root" ]]; then
    echo -e "\n${CYAN}=== root ===${NC}"
    mkdir -p "${OUTPUT_DIR}/root"

    for braw_file in "$SCRIPT_DIR"/*.braw; do
        if [[ ! -f "$braw_file" ]]; then
            continue
        fi

        name=$(basename "$braw_file" .braw)
        output_file="${OUTPUT_DIR}/root/${name}"

        echo -ne "  ${name}... "

        if output=$("$MDHAVERS" build "$braw_file" -o "$output_file" 2>&1); then
            echo -e "${GREEN}ok${NC}"
            ((BUILT++))
        else
            echo -e "${RED}FAILED${NC}"
            echo -e "${DIM}    ${output}${NC}" | head -1
            ((FAILED++))
        fi
    done
fi

for category in "${CATEGORIES[@]}"; do
    # Skip if filtering and doesn't match
    if [[ -n "$CATEGORY_FILTER" && "$category" != "$CATEGORY_FILTER" ]]; then
        continue
    fi

    category_dir="${SCRIPT_DIR}/${category}"
    if [[ ! -d "$category_dir" ]]; then
        continue
    fi

    # Create output subdirectory
    mkdir -p "${OUTPUT_DIR}/${category}"

    echo -e "\n${CYAN}=== ${category} ===${NC}"

    for braw_file in "$category_dir"/*.braw; do
        if [[ ! -f "$braw_file" ]]; then
            continue
        fi

        name=$(basename "$braw_file" .braw)
        output_file="${OUTPUT_DIR}/${category}/${name}"

        echo -ne "  ${name}... "

        # Capture both stdout and stderr
        if output=$("$MDHAVERS" build "$braw_file" -o "$output_file" 2>&1); then
            echo -e "${GREEN}ok${NC}"
            ((BUILT++))
        else
            # Check if it's just missing features (not a real error)
            if echo "$output" | grep -q "fetch\|speir\|http\|network"; then
                echo -e "${DIM}skipped (requires runtime)${NC}"
                ((SKIPPED++))
            else
                echo -e "${RED}FAILED${NC}"
                echo -e "${DIM}    ${output}${NC}" | head -1
                ((FAILED++))
            fi
        fi
    done
done

# Build showcases separately (they have their own structure)
if [[ -z "$CATEGORY_FILTER" || "$CATEGORY_FILTER" == "showcases" ]]; then
    echo -e "\n${CYAN}=== showcases ===${NC}"
    mkdir -p "${OUTPUT_DIR}/showcases"

    # Top-level showcase files
    for braw_file in "$SCRIPT_DIR"/showcases/*.braw; do
        if [[ ! -f "$braw_file" ]]; then
            continue
        fi

        name=$(basename "$braw_file" .braw)
        output_file="${OUTPUT_DIR}/showcases/${name}"
        echo -ne "  ${name}... "

        if "$MDHAVERS" build "$braw_file" -o "$output_file" >/dev/null 2>&1; then
            echo -e "${GREEN}ok${NC}"
            ((BUILT++))
        else
            echo -e "${RED}FAILED${NC}"
            ((FAILED++))
        fi
    done

    for demo_dir in "$SCRIPT_DIR"/showcases/*/; do
        if [[ -d "$demo_dir" ]]; then
            demo_name=$(basename "$demo_dir")
            braw_file="$demo_dir/${demo_name}.braw"

            if [[ -f "$braw_file" ]]; then
                output_file="${OUTPUT_DIR}/showcases/${demo_name}"
                echo -ne "  ${demo_name}... "

                if "$MDHAVERS" build "$braw_file" -o "$output_file" >/dev/null 2>&1; then
                    echo -e "${GREEN}ok${NC}"
                    ((BUILT++))
                else
                    echo -e "${RED}FAILED${NC}"
                    ((FAILED++))
                fi
            fi
        fi
    done
fi

# Summary
echo ""
echo "=============================================="
echo -e "  ${GREEN}Built: ${BUILT}${NC}  ${RED}Failed: ${FAILED}${NC}  ${DIM}Skipped: ${SKIPPED}${NC}"
echo "=============================================="
echo ""
echo -e "Outputs in: ${CYAN}${OUTPUT_DIR}${NC}"
echo ""
echo -e "${DIM}Run examples:${NC}"
echo "  ./outputs/root/hello"
echo "  ./outputs/basics/spread"
echo "  ./outputs/showcases/soond_showcase"
echo ""
