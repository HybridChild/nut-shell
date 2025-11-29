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
    "completion,history"
    "all"
)

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

printf "${BLUE}=== nut-shell Memory Footprint Analysis ===${NC}\n"
printf "\n"

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

# Remove old report to start fresh
rm -f "$REPORT"

# Store summary data for table
declare -a SUMMARY_ROWS

# Build and analyze each feature combination
for feat in "${FEATURES[@]}"; do
    printf "${GREEN}Analyzing: ${feat}${NC}\n"

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
    if ! cargo build --release --target $TARGET $FEAT_FLAGS; then
        echo "  Error: Build failed for feature set: $feat"
        exit 1
    fi

    # Get binary path
    BINARY_PATH="target/$TARGET/release/$BINARY_NAME"

    if [ ! -f "$BINARY_PATH" ]; then
        echo "  Error: Binary not found at $BINARY_PATH"
        continue
    fi

    # Extract size information using size command
    SIZE_OUTPUT=$(arm-none-eabi-size "$BINARY_PATH" 2>/dev/null || size "$BINARY_PATH" 2>/dev/null || cargo size --release --target $TARGET $FEAT_FLAGS 2>/dev/null)

    # Extract individual sections using cargo size -A format
    # Parse .text, .rodata, .data, .bss sizes
    SIZE_DETAIL=$(cargo size --release --target $TARGET $FEAT_FLAGS -- -A 2>/dev/null)

    TEXT=$(echo "$SIZE_DETAIL" | grep "^\.text" | awk '{print $2}' || echo "0")
    RODATA=$(echo "$SIZE_DETAIL" | grep "^\.rodata" | awk '{print $2}' || echo "0")
    DATA=$(echo "$SIZE_DETAIL" | grep "^\.data" | awk '{print $2}' || echo "0")
    BSS=$(echo "$SIZE_DETAIL" | grep "^\.bss" | awk '{print $2}' || echo "0")
    VECTOR=$(echo "$SIZE_DETAIL" | grep "^\.vector_table" | awk '{print $2}' || echo "0")

    # Calculate total flash (text + rodata + data + vector_table)
    TOTAL_FLASH=$((TEXT + RODATA + DATA + VECTOR))

    # Store for summary table
    SUMMARY_ROWS+=("| $FEAT_NAME | ${TEXT}B | ${RODATA}B | ${DATA}B | ${BSS}B | ${TOTAL_FLASH}B |")

    # Detailed analysis section
    cat >> "$REPORT" <<EOF

---

## Feature Set: \`$FEAT_NAME\`

### Binary Size Breakdown

\`\`\`
EOF

    # Use cargo size for consistent output showing actual sections
    cargo size --release --target $TARGET $FEAT_FLAGS -- -A >> "$REPORT" 2>&1 || true

    cat >> "$REPORT" <<EOF
\`\`\`

### Top 10 Largest Symbols (Flash Usage)

\`\`\`
EOF

    # Run cargo-bloat for symbol-level analysis
    # Note: May fail if symbols are stripped
    BLOAT_OUTPUT=$(cargo bloat --release --target $TARGET $FEAT_FLAGS -n 10 2>&1 || echo "Symbol analysis not available (binary may be stripped)")
    echo "$BLOAT_OUTPUT" >> "$REPORT"

    cat >> "$REPORT" <<EOF
\`\`\`

EOF

    echo "  ✓ Complete"
done

# Now generate the report header with summary table
TEMP_REPORT="${REPORT}.tmp"
mv "$REPORT" "$TEMP_REPORT"

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

# Add summary rows to table
for row in "${SUMMARY_ROWS[@]}"; do
    echo "$row" >> "$REPORT"
done

# Append the detailed sections
cat "$TEMP_REPORT" >> "$REPORT"
rm "$TEMP_REPORT"

# Add interpretation section
cat >> "$REPORT" <<EOF

---

## Interpretation Guide

**Build configuration:** Release build optimized for size (\`opt-level = "z"\`, LTO enabled) targeting $TARGET (Cortex-M0/M0+).

### Section Meanings

- **.text**: Executable code (stored in Flash)
- **.rodata**: Read-only data like string literals (stored in Flash)
- **.data**: Initialized variables (stored in Flash, copied to RAM at startup)
- **.bss**: Uninitialized/zero-initialized variables (RAM only, no Flash cost)

### Total Memory Cost

- **Flash usage** = .text + .rodata + .data
- **RAM usage** = .data + .bss + stack

### Understanding Generic Type Sizes

nut-shell is generic over several user-provided types. The analysis uses **zero-size stubs** to measure only nut-shell's contribution.

**Your implementations have two cost components:**

#### Runtime Size (RAM - struct instance on stack)

| Generic Type | Analysis Uses | Typical Real Implementation |
|--------------|---------------|------------------------------|
| \`CharIo\` | Zero-size \`MinimalIo\` (0 bytes) | Simple UART (~4-8 bytes), buffered UART (~64-128 bytes), or USB CDC-ACM (~340+ bytes with packet buffers) |
| \`CredentialProvider\` | Zero-size \`MinCredentials\` (0 bytes) | Zero-size build-time (0 bytes), static array reference (~4-8 bytes), or flash-backed (~8-16 bytes) |
| \`CommandHandler\` | Zero-size \`MinHandlers\` (0 bytes) | Stateless (0 bytes) or stateful (depends on fields) |

#### Code Size (Flash - trait implementation logic)

Beyond the struct size, your trait implementations add Flash code:

- **\`CharIo\`**:
  - Simple UART: ~100-200 bytes (register read/write only)
  - Buffered UART: ~500 bytes (ring buffer management)
  - USB CDC-ACM: ~1-3KB (packet handling + USB protocol)
- **\`CredentialProvider\`**:
  - Static array lookup: ~500 bytes - 1KB (linear search + constant-time compare)
  - Flash-backed storage: ~1-3KB (flash I/O + deserialization + crypto)
- **\`CommandHandler\`**:
  - Your command implementations (varies widely: simple GPIO toggle ~50 bytes, complex network request ~2-5KB per command)

**This analysis measures only nut-shell's code.** Your trait implementations add additional Flash/RAM costs on top.

#### Buffer Configuration (RAM cost in .bss)

Runtime buffers are defined via \`ShellConfig\` and allocated in the \`Shell\` struct instance. The analysis uses **\`MinimalConfig\`** (smaller buffers than \`DefaultConfig\`) to isolate nut-shell's code overhead from your application's RAM budget.

**\`MinimalConfig\` buffer sizes:**

\`\`\`rust
const MAX_INPUT: usize = 64;          // vs. 128 (DefaultConfig)
const MAX_PATH_DEPTH: usize = 4;      // vs. 8 (DefaultConfig)
const MAX_ARGS: usize = 8;            // vs. 16 (DefaultConfig)
const MAX_PROMPT: usize = 32;         // vs. 64 (DefaultConfig)
const MAX_RESPONSE: usize = 128;      // vs. 256 (DefaultConfig)

#[cfg(feature = "history")]
const HISTORY_SIZE: usize = 4;        // vs. 10 (DefaultConfig)

#[cfg(not(feature = "history"))]
const HISTORY_SIZE: usize = 0;        // 0 when history disabled
\`\`\`

**RAM scales linearly with buffer sizes.** Doubling \`MAX_INPUT\` adds ~64 bytes RAM (but Flash stays constant).

**History buffer RAM cost** = \`HISTORY_SIZE × MAX_INPUT\` = 4 × 64 = ~256 bytes (when history enabled).

#### Message Configuration (Flash cost in .rodata)

User-visible messages (welcome, login prompts, errors) are defined via \`ShellConfig\` as const strings. The analysis uses **\`MinimalConfig\`** (shorter messages than \`DefaultConfig\`) to minimize .rodata overhead.

**\`MinimalConfig\` messages:**

\`\`\`rust
const MSG_WELCOME: &'static str = "Welcome!";                                  // 8 bytes vs. 21 (DefaultConfig)
const MSG_LOGIN_PROMPT: &'static str = "Login> ";                              // 7 bytes vs. 7 (DefaultConfig)
const MSG_LOGIN_SUCCESS: &'static str = "Logged in.";                          // 10 bytes vs. 34 (DefaultConfig)
const MSG_LOGIN_FAILED: &'static str = "Login failed.";                        // 13 bytes vs. 26 (DefaultConfig)
const MSG_LOGOUT: &'static str = "Logged out.";                                // 11 bytes vs. 11 (DefaultConfig)
const MSG_INVALID_LOGIN_FORMAT: &'static str = "Invalid format. Use <name>:<password>";  // 38 bytes vs. 45 (DefaultConfig)
\`\`\`

**Total message overhead:**
- \`MinimalConfig\`: ~87 bytes
- \`DefaultConfig\`: ~144 bytes

**Both buffers and messages are fully customizable** via the \`ShellConfig\` trait to match your application's needs and hardware constraints.

### Feature Impact Summary

Compare the "none" baseline with feature-enabled builds to see the cost of each feature:

| Feature Set | Flash (Code) | RAM (Data) | Notes |
|-------------|--------------|------------|-------|
| authentication | ~2-3KB | ~100 bytes | SHA-256 hashing, login state machine |
| completion | ~1-2KB | ~0 bytes | Prefix matching (stateless) |
| history | ~1KB | HISTORY_SIZE × MAX_INPUT | Command history buffer (10 × 128 = ~1.3KB default) |
| async | ~1KB | ~0 bytes | Async runtime integration (stateless) |

**Note:** Flash costs are the compiled code for each feature. RAM costs are the runtime state/buffers.

EOF

printf "\n"
printf "${GREEN}✓ Analysis complete!${NC}\n"
printf "Report generated at: ${BLUE}%s${NC}\n" "$REPORT"
printf "\n"
echo "To view the report:"
echo "  cat $REPORT"
echo "  or open $REPORT in your editor"

# Clean up build artifacts
printf "\n"
printf "${GREEN}Cleaning up build artifacts...${NC}\n"
cargo clean --quiet
cd ..
