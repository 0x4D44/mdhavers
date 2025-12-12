#!/bin/bash
# Build all showcase demos
# Usage: ./build_all.sh [--clean]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MDHAVERS="${SCRIPT_DIR}/../../target/release/mdhavers"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check if mdhavers compiler exists
if [[ ! -f "$MDHAVERS" ]]; then
    echo -e "${YELLOW}Building mdhavers compiler first...${NC}"
    (cd "${SCRIPT_DIR}/../.." && cargo build --release)
fi

# Clean mode
if [[ "$1" == "--clean" ]]; then
    echo -e "${YELLOW}Cleaning built executables...${NC}"
    find "$SCRIPT_DIR" -type f -executable ! -name "*.sh" -delete
    echo -e "${GREEN}Clean complete!${NC}"
    exit 0
fi

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║           mdhavers Showcase Demo Builder                  ║"
echo "║              \"Braw demos fer braw fowk!\"                  ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo -e "${NC}"

BUILT=0
FAILED=0
DEMOS=()

# Find all .braw files in showcase subdirectories
for demo_dir in "$SCRIPT_DIR"/*/; do
    if [[ -d "$demo_dir" ]]; then
        demo_name=$(basename "$demo_dir")
        braw_file="$demo_dir/${demo_name}.braw"

        if [[ -f "$braw_file" ]]; then
            echo -ne "  Building ${CYAN}${demo_name}${NC}... "

            output_file="$demo_dir/${demo_name}"

            if "$MDHAVERS" build "$braw_file" -o "$output_file" 2>/dev/null; then
                echo -e "${GREEN}✓${NC}"
                DEMOS+=("$demo_name")
                ((BUILT++))
            else
                echo -e "${RED}✗${NC}"
                ((FAILED++))
            fi
        fi
    fi
done

echo ""
echo -e "${GREEN}Built: ${BUILT}${NC}  ${RED}Failed: ${FAILED}${NC}"
echo ""

if [[ ${#DEMOS[@]} -gt 0 ]]; then
    echo -e "${CYAN}Available demos:${NC}"
    for demo in "${DEMOS[@]}"; do
        echo "  ./showcases/${demo}/${demo}"
    done
fi

echo ""
echo -e "${YELLOW}Tip: Run individual demos from this directory${NC}"
