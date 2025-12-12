#!/bin/bash
# Update .expected files from current interpreter output
# Use with caution - review changes before committing!

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_DIR"

# Build first so we don't include compilation warnings
cargo build --release --bin mdhavers 2>/dev/null

count=0
for braw in $(find tests/golden -name "*.braw" | sort); do
    expected="${braw%.braw}.expected"
    echo "Updating $expected..."
    # Run and capture only stdout (discard stderr/warnings)
    ./target/release/mdhavers "$braw" > "$expected" 2>/dev/null || true
    count=$((count + 1))
done

echo ""
echo "Updated $count expected files."
echo "Review changes with: git diff tests/golden/"
