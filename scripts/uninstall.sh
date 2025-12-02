#!/bin/sh
# Slides Uninstall Script for Linux and macOS
# Usage: curl -sSL https://raw.githubusercontent.com/MichaelBrauner/slides-rs/main/scripts/uninstall.sh | sh

BINARY_PATH="$HOME/.local/bin/slides"

echo "Uninstalling slides..."

if [ -f "$BINARY_PATH" ]; then
    rm "$BINARY_PATH"
    echo "  Removed $BINARY_PATH"
else
    echo "  Not found: $BINARY_PATH"
fi

echo ""
echo "Uninstall complete!"
