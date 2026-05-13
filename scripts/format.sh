#!/bin/bash

# format.sh - Run cargo fmt for all subprojects

set -e

echo "Formatting nut-shell workspace..."

# Root project
echo "  [1/6] Root project"
cargo fmt

# Examples
echo "  [2/6] examples/native"
(cd examples/native && cargo fmt)

echo "  [3/6] examples/rp-pico"
(cd examples/rp-pico && cargo fmt)

echo "  [4/6] examples/stm32f072"
(cd examples/stm32f072 && cargo fmt)

echo "  [5/6] examples/stm32h753"
(cd examples/stm32h753 && cargo fmt)

# Tools
echo "  [6/6] size-analysis/minimal"
(cd size-analysis/minimal && cargo fmt)

echo ""
echo "Formatting complete!"
