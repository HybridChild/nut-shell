#!/bin/bash

# cleanup.sh - Run cargo clean for all subprojects

set -e

echo "Cleaning nut-shell workspace..."

# Root project
echo "  [1/7] Root project"
cargo clean

# Examples
echo "  [2/7] examples/native"
(cd examples/native && cargo clean)

echo "  [3/7] examples/rp-pico"
(cd examples/rp-pico && cargo clean)

echo "  [4/7] examples/stm32f072"
(cd examples/stm32f072 && cargo clean)

echo "  [5/7] examples/stm32h753"
(cd examples/stm32h753 && cargo clean)

# Tools
echo "  [6/7] size-analysis/minimal"
(cd size-analysis/minimal && cargo clean)

# Remove tmp directory if it exists
if [ -d "tmp" ]; then
    echo "  [7/7] Removing tmp directory"
    rm -rf tmp
else
    echo "  [7/7] tmp directory (not present)"
fi

echo ""
echo "Cleanup complete!"
