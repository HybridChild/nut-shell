#!/bin/bash

# cleanup.sh - Run cargo clean for all subprojects

set -e

echo "Cleaning nut-shell workspace..."

# Root project
echo "  [1/8] Root project"
cargo clean

# Examples
echo "  [2/8] examples/native"
(cd examples/native && cargo clean)

echo "  [3/8] examples/rp-pico"
(cd examples/rp-pico && cargo clean)

echo "  [4/8] examples/stm32f072"
(cd examples/stm32f072 && cargo clean)

echo "  [5/8] examples/stm32h753"
(cd examples/stm32h753 && cargo clean)

echo "  [6/8] examples/stm32h753-embassy"
(cd examples/stm32h753-embassy && cargo clean)

# Tools
echo "  [7/8] size-analysis/minimal"
(cd size-analysis/minimal && cargo clean)

# Remove tmp directory if it exists
if [ -d "tmp" ]; then
    echo "  [8/8] Removing tmp directory"
    rm -rf tmp
else
    echo "  [8/8] tmp directory (not present)"
fi

echo ""
echo "Cleanup complete!"
