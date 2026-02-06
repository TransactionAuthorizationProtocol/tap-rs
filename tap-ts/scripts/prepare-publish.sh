#!/bin/bash
set -e

echo "Preparing @taprsvp/agent for publishing..."

# Build WASM if needed
if [ ! -f "../tap-wasm/pkg/tap_wasm_bg.wasm" ]; then
  echo "WASM files not found. Building WASM..."
  
  # Check if wasm-pack is installed
  if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed."
    echo "Please install it with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
  fi
  
  cd ../tap-wasm
  wasm-pack build --target web --out-dir pkg --release
  cd ../tap-ts
fi

# Clean and create wasm directory
rm -rf wasm
mkdir -p wasm

# Copy WASM files
echo "Copying WASM files..."
cp ../tap-wasm/pkg/tap_wasm_bg.wasm wasm/
cp ../tap-wasm/pkg/tap_wasm_bg.wasm.d.ts wasm/ 2>/dev/null || true
cp ../tap-wasm/pkg/tap_wasm.js wasm/
cp ../tap-wasm/pkg/tap_wasm.d.ts wasm/

# Update imports in the copied files to use relative paths
echo "Updating import paths..."
sed -i.bak "s|'./tap_wasm_bg.wasm'|'./tap_wasm_bg.wasm'|g" wasm/tap_wasm.js
rm wasm/*.bak 2>/dev/null || true

# Build TypeScript
echo "Building TypeScript..."
npm run build

echo "Package ready for publishing!"
echo "Files to be included:"
ls -la wasm/
echo ""
echo "To publish, run: npm publish"