#!/bin/bash

# format.sh - Run cargo fmt for all subprojects

set -e

echo "Formatting nut-shell workspace..."

# Root project
echo "  [1/7] Root project"
cargo fmt

# Examples
echo "  [2/7] examples/native"
(cd examples/native && cargo fmt)

echo "  [3/7] examples/rp-pico"
(cd examples/rp-pico && cargo fmt)

echo "  [4/7] examples/stm32f072"
(cd examples/stm32f072 && cargo fmt)

echo "  [5/7] examples/stm32h753"
(cd examples/stm32h753 && cargo fmt)

echo "  [6/7] examples/stm32h753-embassy"
(cd examples/stm32h753-embassy && cargo fmt)

# Tools
echo "  [7/7] size-analysis/minimal"
(cd size-analysis/minimal && cargo fmt)

echo ""
echo "Formatting complete!"
