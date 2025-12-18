#!/bin/bash

# format.sh - Run cargo fmt for all subprojects

set -e

echo "Formatting nut-shell workspace..."

# Root project
echo "  [1/5] Root project"
cargo fmt

# Examples
echo "  [2/5] examples/native"
(cd examples/native && cargo fmt)

echo "  [3/5] examples/rp-pico"
(cd examples/rp-pico && cargo fmt)

echo "  [4/5] examples/stm32f072"
(cd examples/stm32f072 && cargo fmt)

# Tools
echo "  [5/5] size-analysis/minimal"
(cd size-analysis/minimal && cargo fmt)

echo ""
echo "Formatting complete!"
