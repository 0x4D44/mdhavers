#!/bin/bash
# Update .expected files from current interpreter output
# Use with caution - review changes before committing!

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_DIR"

count=0
for braw in $(find tests/golden -name "*.braw" | sort); do
    expected="${braw%.braw}.expected"
    echo "Updating $expected..."
    cargo run --release -- "$braw" > "$expected" 2>&1 || true
    count=$((count + 1))
done

echo ""
echo "Updated $count expected files."
echo "Review changes with: git diff tests/golden/"
