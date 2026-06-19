#!/bin/bash
# Install dx-icons globally with index and data

set -e

echo "Building icon CLI..."
cargo build --release --bin icon

echo "Building index..."
cargo run --release --bin build_index

echo "Installing to ~/.dx/icon/..."
mkdir -p ~/.dx/icon
cp -r index ~/.dx/icon/
cp -r data ~/.dx/icon/

echo "Installing binary..."
cargo install --path . --bin icon

echo ""
echo "✓ Installation complete!"
echo ""
echo "The icon command is now available globally."
echo "Index location: ~/.dx/icon/index"
echo "Data location: ~/.dx/icon/data"
echo ""
echo "Try: icon search home --limit 5"
