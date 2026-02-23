#!/bin/bash
set -e

echo "Preparing @taprsvp/agent for publishing..."

# Build WASM and copy files
npm run build:wasm

# Build TypeScript
echo "Building TypeScript..."
npm run build

echo "Package ready for publishing!"
echo "Files to be included:"
ls -la wasm/
echo ""
echo "To publish, run: npm publish"
