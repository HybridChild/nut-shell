#!/bin/bash
set -e

# Memory footprint analysis script for nut-shell
# Builds minimal reference binary with different feature combinations
# and generates a detailed size report

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MINIMAL_DIR="$SCRIPT_DIR/minimal"
REPORT="$SCRIPT_DIR/report.md"
TARGET="thumbv6m-none-eabi"
BINARY_NAME="minimal"

# Feature combinations to test
FEATURES=(
    "none"
    "authentication"
    "completion"
    "history"
    "async"
    "authentication,completion"
    "authentication,history"
    "completion,history"
    "all"
)

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== nut-shell Memory Footprint Analysis ===${NC}"
echo ""

# Check for required tools
echo "Checking for required tools..."
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found"
    exit 1
fi

if ! command -v cargo-bloat &> /dev/null; then
    echo "Installing cargo-bloat..."
    cargo install cargo-bloat
fi

# Check if target is installed
if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "Installing target $TARGET..."
    rustup target add $TARGET
fi

# Navigate to minimal directory
cd "$MINIMAL_DIR"

# Generate report header
cat > "$REPORT" <<EOF
# nut-shell Memory Footprint Analysis

**Generated:** $(date)
**Target:** $TARGET (ARMv6-M, Cortex-M0/M0+)
**Optimization:** \`opt-level = "z"\`, LTO enabled

This analysis uses a minimal reference binary with an empty directory tree to measure
the pure overhead of nut-shell with different feature combinations.

## Summary Table

| Feature Set | .text (Flash) | .rodata (Flash) | .data (RAM) | .bss (RAM) | Total Flash |
|-------------|---------------|-----------------|-------------|------------|-------------|
EOF

# Store summary data for table
declare -a SUMMARY_ROWS

# Build and analyze each feature combination
for feat in "${FEATURES[@]}"; do
    echo -e "${GREEN}Analyzing: ${feat}${NC}"

    # Determine cargo flags
    if [ "$feat" = "none" ]; then
        FEAT_FLAGS="--no-default-features"
        FEAT_NAME="none"
    elif [ "$feat" = "all" ]; then
        FEAT_FLAGS="--all-features"
        FEAT_NAME="all features"
    else
        FEAT_FLAGS="--no-default-features --features $feat"
        FEAT_NAME="$feat"
    fi

    # Build
    echo "  Building..."
    cargo build --release --target $TARGET $FEAT_FLAGS 2>&1 | grep -v "Compiling\|Finished" || true

    # Get binary path
    BINARY_PATH="target/$TARGET/release/$BINARY_NAME"

    if [ ! -f "$BINARY_PATH" ]; then
        echo "  Error: Binary not found at $BINARY_PATH"
        continue
    fi

    # Extract size information using size command
    SIZE_OUTPUT=$(arm-none-eabi-size "$BINARY_PATH" 2>/dev/null || size "$BINARY_PATH" 2>/dev/null || cargo size --release --target $TARGET $FEAT_FLAGS 2>/dev/null)

    # Parse size output (format: text data bss dec hex filename)
    TEXT=$(echo "$SIZE_OUTPUT" | tail -n 1 | awk '{print $1}')
    DATA=$(echo "$SIZE_OUTPUT" | tail -n 1 | awk '{print $2}')
    BSS=$(echo "$SIZE_OUTPUT" | tail -n 1 | awk '{print $3}')

    # Calculate rodata (approximation: we'll get this from cargo-bloat)
    # For now, use a placeholder and fill from detailed analysis
    RODATA="TBD"

    # Store for summary table
    TOTAL_FLASH=$((TEXT + DATA))
    SUMMARY_ROWS+=("| $FEAT_NAME | ${TEXT}B | ${RODATA} | ${DATA}B | ${BSS}B | ${TOTAL_FLASH}B |")

    # Detailed analysis section
    cat >> "$REPORT" <<EOF

---

## Feature Set: \`$FEAT_NAME\`

### Binary Size Breakdown

\`\`\`
EOF

    # Run arm-none-eabi-size or fallback to size
    if command -v arm-none-eabi-size &> /dev/null; then
        arm-none-eabi-size -A "$BINARY_PATH" >> "$REPORT" 2>&1 || true
    else
        size -A "$BINARY_PATH" >> "$REPORT" 2>&1 || true
    fi

    cat >> "$REPORT" <<EOF
\`\`\`

### Top 10 Largest Symbols (Flash Usage)

\`\`\`
EOF

    # Run cargo-bloat for symbol-level analysis
    cargo bloat --release --target $TARGET $FEAT_FLAGS -n 10 >> "$REPORT" 2>&1 || true

    cat >> "$REPORT" <<EOF
\`\`\`

EOF

    echo "  ✓ Complete"
done

# Add summary rows to table
for row in "${SUMMARY_ROWS[@]}"; do
    echo "$row" >> "$REPORT"
done

# Add interpretation section
cat >> "$REPORT" <<EOF

---

## Interpretation Guide

### Section Meanings

- **.text**: Executable code (stored in Flash)
- **.rodata**: Read-only data like string literals (stored in Flash)
- **.data**: Initialized variables (stored in Flash, copied to RAM at startup)
- **.bss**: Uninitialized/zero-initialized variables (RAM only, no Flash cost)

### Total Memory Cost

- **Flash usage** = .text + .rodata + .data
- **RAM usage** = .data + .bss + stack

### Understanding Generic Type Sizes

nut-shell is generic over several user-provided types. Their sizes depend on YOUR implementation:

#### CharIo (I/O Implementation)
- **Minimal UART wrapper**: ~0-16 bytes (typically just register addresses or indices)
- **Buffered I/O**: ~64-512 bytes (if you add internal buffers)
- **This analysis uses**: Zero-size \`MinimalIo\` stub (0 bytes)

#### CredentialProvider (Authentication Storage)
- **Static array**: ~N × sizeof(Credential)
- **Flash-backed storage**: ~4-16 bytes (pointer + metadata)
- **This analysis uses**: Zero-size \`MinCredentials\` stub (0 bytes)

#### CommandHandler (Command Execution Logic)
- **Stateless handler**: 0 bytes (zero-size type)
- **Stateful handler**: Size of your state fields
- **This analysis uses**: Zero-size \`MinHandlers\` (0 bytes)

#### ShellConfig (Buffer Sizes)
The analysis uses minimal buffers:
- MAX_INPUT: 64 bytes
- MAX_PATH_DEPTH: 4 (= 4 × sizeof(usize) = 16-32 bytes depending on target)
- MAX_ARGS: 8
- MAX_PROMPT: 32 bytes
- MAX_RESPONSE: 128 bytes

**Your actual RAM usage will be higher if you use larger buffers.**

### Feature Impact Summary

Compare the "none" baseline with feature-enabled builds to see the cost of each feature:

- **authentication**: Adds login state machine, password hashing, access control
- **completion**: Adds tab-completion logic, candidate matching
- **history**: Adds command history buffer (size = HISTORY_SIZE × MAX_INPUT)
- **async**: Adds async runtime support (minimal overhead, mainly in binary size)

### Methodology

This analysis uses:
1. **Empty directory tree**: No commands, no directories (measures pure shell overhead)
2. **Minimal config**: Small buffers to isolate nut-shell code size
3. **Zero-size generics**: Stub implementations to measure only nut-shell contribution
4. **Release build**: Optimized for size (\`opt-level = "z"\`, LTO enabled)
5. **Real embedded target**: $TARGET (Cortex-M0/M0+)

The results show the **minimum overhead** of nut-shell itself. Your actual binary will be larger due to:
- Your command implementations
- Your directory tree structure
- Your I/O and credential provider implementations
- Larger buffer sizes in ShellConfig

---

## Report Location

This report is generated at: \`size-analysis/report.md\`

To regenerate: \`cd size-analysis && ./analyze.sh\`

EOF

echo ""
echo -e "${GREEN}✓ Analysis complete!${NC}"
echo -e "Report generated at: ${BLUE}$REPORT${NC}"
echo ""
echo "To view the report:"
echo "  cat $REPORT"
echo "  or open $REPORT in your editor"
