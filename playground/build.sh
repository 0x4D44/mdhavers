#!/bin/bash
# Build script for mdhavers playground

set -e

echo "Building mdhavers Playground..."

# Build WASM package
echo "Compiling to WebAssembly..."
wasm-pack build --target web

# Copy to web directory
echo "Copying to web directory..."
cp -r pkg web/

echo "Build complete!"
echo ""
echo "To test locally:"
echo "  cd web && python3 -m http.server 8080"
echo "  Then open http://localhost:8080"
