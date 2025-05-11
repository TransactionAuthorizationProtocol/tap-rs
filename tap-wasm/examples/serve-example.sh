#!/bin/bash

# This script creates a simple HTTP server to test the browser example

echo "Starting a local HTTP server on port 8000..."
echo "Open your browser and navigate to http://localhost:8000/examples/browser-example.html"

if command -v python3 &> /dev/null; then
    python3 -m http.server 8000 -d /Users/pelle/code/notabene/tap-rs/tap-wasm/
elif command -v python &> /dev/null; then
    python -m SimpleHTTPServer 8000
else
    echo "Error: Python is not installed. Please install Python 3 or use a different HTTP server."
    exit 1
fi