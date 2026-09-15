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

if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed"
    exit 1
fi

# Build for web target
if [ "$BUILD_TYPE" = "dev" ]; then
    wasm-pack build --target web --dev --out-dir dist
else
    wasm-pack build --target web --release --out-dir dist
fi

npm run build:css

# Normalize generated package metadata to the author's preferred npm format.
ruby <<'RUBY'
require "json"

path = "dist/package.json"
unless File.exist?(path)
    warn "⚠️  package.json not found at #{path}; skipped metadata normalization."
    exit 0
end

pkg = JSON.parse(File.read(path))
pkg.delete("collaborators")
pkg["$schema"] = "https://json.schemastore.org/package.json"
pkg["author"] = {
    "name" => "Logue",
    "email" => "logue@hotmail.co.jp",
    "url" => "https://logue.dev/"
}
pkg["homepage"] = "https://github.com/logue/umd-core"
pkg["repository"] = {
    "type" => "git",
    "url" => "git+ssh://git@github.com/logue/umd-core.git"
}
pkg["bugs"] = {
    "url" => "https://github.com/logue/umd-core/issues"
}
pkg["sideEffects"] = false
pkg["name"] = "universal-markdown"

# Include every file actually emitted into dist/, rather than a hand-maintained
# list, so new build outputs (fonts, css, etc.) are picked up automatically.
dist_root = File.dirname(path)
pkg["files"] = Dir.glob("**/*", File::FNM_DOTMATCH, base: dist_root)
    .reject { |f| File.directory?(File.join(dist_root, f)) }
    .reject { |f| f.split("/").any? { |part| part.start_with?(".") } }
    .sort

pkg["exports"] = {
    "." => {
        "types" => "./umd.d.ts",
        "default" => "./umd.js"
    },
    "./umd.js" => {
        "types" => "./umd.d.ts",
        "default" => "./umd.js"
    },
    "./umd_bg.wasm" => {
        "types" => "./umd_bg.wasm.d.ts",
        "default" => "./umd_bg.wasm"
    },
    "./umd-reference.css" => "./umd-reference.css",
    "./package.json" => "./package.json"
}

File.write(path, JSON.pretty_generate(pkg) + "\n")
RUBY

# Format dist/package.json with Biome. dist/ is otherwise excluded from
# Biome's scope (see biome.jsonc's "!!**/dist" rule), so a throwaway config
# that extends the project's config but re-includes dist/ is used to target
# just this one file without touching the rest of dist/.
BIOME_TMP_CONFIG_DIR="$(mktemp -d)"
trap 'rm -rf "$BIOME_TMP_CONFIG_DIR"' EXIT
cat > "$BIOME_TMP_CONFIG_DIR/biome.jsonc" <<EOF
{
  "extends": ["$(pwd)/biome.jsonc"],
  "vcs": { "enabled": false },
  "files": { "includes": ["**"] }
}
EOF
npx biome format --config-path="$BIOME_TMP_CONFIG_DIR" --write dist/package.json
rm -rf "$BIOME_TMP_CONFIG_DIR"
trap - EXIT

echo "✅ Build completed successfully!"
echo "📦 Output directory: dist/"
echo ""
echo "Usage example:"
echo "  import init, { parse } from './dist/umd.js';"
echo "  await init();"
echo "  const html = parse('# Hello World');"
