#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TS_DIR="$(dirname "$SCRIPT_DIR")"
WASM_DIR="$TS_DIR/../tap-wasm"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
  echo "Error: wasm-pack is not installed."
  echo "Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
  exit 1
fi

# Build WASM
echo "Building WASM..."
cd "$WASM_DIR"
wasm-pack build --target web --out-dir pkg --release

# Copy WASM files to tap-ts/wasm/
echo "Copying WASM files to tap-ts/wasm/..."
rm -rf "$TS_DIR/wasm"
mkdir -p "$TS_DIR/wasm"
cp pkg/tap_wasm_bg.wasm "$TS_DIR/wasm/"
cp pkg/tap_wasm_bg.wasm.d.ts "$TS_DIR/wasm/" 2>/dev/null || true
cp pkg/tap_wasm.js "$TS_DIR/wasm/"
cp pkg/tap_wasm.d.ts "$TS_DIR/wasm/"

echo "WASM build complete."
