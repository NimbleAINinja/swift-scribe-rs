#!/bin/bash

# Installation script for swift-scribe transcription helper
# This installs the Swift helper binary required by the library

set -e

echo " swift-scribe Helper Installation"
echo "================================"
echo ""

# Check if we're in the swift-scribe-rs directory
if [ ! -f "helpers/transcribe.swift" ]; then
    echo "Error: This script must be run from the swift-scribe-rs directory"
    exit 1
fi

# Build the helper if it doesn't exist
if [ ! -f "helpers/transcribe" ]; then
    echo "Building Swift helper..."
    make helpers
    echo ""
fi

# Ask user for install location
echo "Choose installation location:"
echo "  1) User (~/.local/bin) - Recommended"
echo "  2) System (/usr/local/bin) - Requires sudo"
echo "  3) Custom path"
echo ""
read -p "Selection [1]: " choice
choice=${choice:-1}

case $choice in
    1)
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
        INSTALL_PATH="$INSTALL_DIR/transcribe"
        ;;
    2)
        INSTALL_DIR="/usr/local/bin"
        INSTALL_PATH="$INSTALL_DIR/transcribe"
        NEED_SUDO=true
        ;;
    3)
        read -p "Enter install path: " CUSTOM_PATH
        INSTALL_DIR="$(dirname "$CUSTOM_PATH")"
        INSTALL_PATH="$CUSTOM_PATH"
        mkdir -p "$INSTALL_DIR"
        ;;
    *)
        echo "Invalid selection"
        exit 1
        ;;
esac

echo ""
echo "Installing to: $INSTALL_PATH"

# Install
if [ "$NEED_SUDO" = true ]; then
    sudo cp helpers/transcribe "$INSTALL_PATH"
    sudo chmod +x "$INSTALL_PATH"
else
    cp helpers/transcribe "$INSTALL_PATH"
    chmod +x "$INSTALL_PATH"
fi

echo "SUCCESS: Installation complete!"
echo ""
echo "The helper is now available at:"
echo "  $INSTALL_PATH"
echo ""
echo "You can now use swift-scribe in your Rust projects:"
echo ""
echo "  [dependencies]"
echo "  swift-scribe-rs = { git = \"https://github.com/NimbleAINinja/swift-scribe-rs\" }"
echo ""
echo "See LIBRARY_USAGE.md for complete integration guide."
