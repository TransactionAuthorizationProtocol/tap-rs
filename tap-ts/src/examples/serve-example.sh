#!/bin/bash

# Build the package for browser
echo "Building package for browser..."
npm run build:browser

# Serve the examples directory with a simple HTTP server
echo "Starting server on http://localhost:8000/src/examples/browser-example.html"

# Use Python HTTP server if available
if command -v python3 &> /dev/null; then
    python3 -m http.server 8000
elif command -v python &> /dev/null; then
    python -m SimpleHTTPServer 8000
else
    echo "Error: Python is not installed. Please install Python or use another HTTP server to serve the examples."
    exit 1
fi