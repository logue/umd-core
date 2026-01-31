#!/bin/bash
# WASM Build Script for Universal Markdown
#
# This script builds the Rust library to WebAssembly
# and generates TypeScript bindings.
#
# Prerequisites:
#   - wasm-pack: cargo install wasm-pack
#
# Usage:
#   ./build.sh [dev|release]

set -e

BUILD_TYPE="${1:-release}"

echo "🔧 Building Universal Markdown for WASM (${BUILD_TYPE})..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed"
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

# Build for web target
if [ "$BUILD_TYPE" = "dev" ]; then
    wasm-pack build --target web --dev --out-dir pkg
else
    wasm-pack build --target web --release --out-dir pkg
fi

echo "✅ Build completed successfully!"
echo "📦 Output directory: pkg/"
echo ""
echo "Usage example:"
echo "  import init, { parse_wiki } from './pkg/universal_markdown.js';"
echo "  await init();"
echo "  const html = parse_wiki('# Hello World');"
