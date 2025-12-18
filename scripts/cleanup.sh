#!/bin/bash

# cleanup.sh - Run cargo clean for all subprojects

set -e

echo "Cleaning nut-shell workspace..."

# Root project
echo "  [1/6] Root project"
cargo clean

# Examples
echo "  [2/6] examples/native"
(cd examples/native && cargo clean)

echo "  [3/6] examples/rp-pico"
(cd examples/rp-pico && cargo clean)

echo "  [4/6] examples/stm32f072"
(cd examples/stm32f072 && cargo clean)

# Tools
echo "  [5/6] size-analysis/minimal"
(cd size-analysis/minimal && cargo clean)

# Remove tmp directory if it exists
if [ -d "tmp" ]; then
    echo "  [6/6] Removing tmp directory"
    rm -rf tmp
else
    echo "  [6/6] tmp directory (not present)"
fi

echo ""
echo "Cleanup complete!"
