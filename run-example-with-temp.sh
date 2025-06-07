#!/bin/bash
# Script to run TAP examples with temporary storage instead of ~/.tap

# Check if an example name was provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <example-name> [additional-args...]"
    echo ""
    echo "This script runs TAP examples with temporary storage to protect your ~/.tap directory."
    echo ""
    echo "Available examples:"
    echo "  - key_labels_demo"
    echo "  - key_management"
    echo ""
    echo "Example:"
    echo "  $0 key_labels_demo"
    exit 1
fi

EXAMPLE_NAME=$1
shift

# Create a temporary directory
TEMP_DIR=$(mktemp -d)

# Set environment variables to use temporary storage
export TAP_HOME="$TEMP_DIR"
export TAP_TEST_DIR="$TEMP_DIR"

echo "Running example '$EXAMPLE_NAME' with temporary storage at: $TEMP_DIR"
echo ""

# Run the example from the tap-agent package
cargo run --example "$EXAMPLE_NAME" -p tap-agent -- "$@"

# Clean up
echo ""
echo "Cleaning up temporary directory: $TEMP_DIR"
rm -rf "$TEMP_DIR"