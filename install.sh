#!/bin/bash
# Installation script for aip (AI Providers CLI)

set -e

echo "Building aip..."
cargo build --release

echo ""
echo "Build complete!"
echo ""
echo "To install aip, run one of the following:"
echo ""
echo "  # Option 1: Copy to /usr/local/bin (requires sudo)"
echo "  sudo cp target/release/aip /usr/local/bin/"
echo ""
echo "  # Option 2: Copy to ~/.local/bin (no sudo required)"
echo "  mkdir -p ~/.local/bin"
echo "  cp target/release/aip ~/.local/bin/"
echo "  # Make sure ~/.local/bin is in your PATH"
echo ""
echo "  # Option 3: Create a symlink"
echo "  sudo ln -s $(pwd)/target/release/aip /usr/local/bin/aip"
echo ""
echo "After installation, verify with: aip --version"
