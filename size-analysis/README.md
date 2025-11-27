# Memory Footprint Analysis

This directory contains tools for analyzing the memory footprint of the nut-shell library across different feature combinations.

## Quick Start

```bash
cd size-analysis
./analyze.sh
cat report.md
```

This will build a minimal reference binary with all feature combinations and generate a detailed size report.

## What Gets Analyzed

### Minimal Reference Binary

The analysis uses `size-analysis/minimal/` - a bare-bones embedded binary that:

- **Empty directory tree**: No commands, no directories (measures pure shell overhead)
- **Minimal buffers**: Small `ShellConfig` constants to isolate nut-shell code size
- **Zero-size generics**: Stub implementations for `CharIo`, `CredentialProvider`, and `CommandHandler`
- **Real embedded target**: `thumbv6m-none-eabi` (Cortex-M0/M0+)
- **Size-optimized build**: `opt-level = "z"` with LTO enabled

### Feature Combinations Tested

1. **none** - Minimal build (no features)
2. **authentication** - Login and access control only
3. **completion** - Tab completion only
4. **history** - Command history only
5. **async** - Async support only
6. **authentication,completion** - Auth + completion
7. **authentication,history** - Auth + history
8. **completion,history** - Interactive features
9. **all** - All features enabled

## Understanding the Report

### Memory Sections

The report breaks down binary size into these sections:

- **.text**: Executable code (Flash)
- **.rodata**: Read-only data like string literals (Flash)
- **.data**: Initialized variables (Flash storage, copied to RAM at startup)
- **.bss**: Uninitialized variables (RAM only, zero Flash cost)

**Total Flash usage** = `.text` + `.rodata` + `.data`
**Total RAM usage** = `.data` + `.bss` + stack

### Symbol-Level Analysis

For each feature combination, the report includes:

1. **Binary size breakdown** - Section sizes from `arm-none-eabi-size`
2. **Top 10 largest symbols** - Functions/data contributing most to Flash usage (from `cargo-bloat`)

This helps identify exactly which functions or features consume the most space.

### Generic Type Sizes

nut-shell is generic over several user-provided types. The analysis uses **zero-size stubs** to measure only nut-shell's contribution:

| Generic Type | Analysis Uses | Typical Real Implementation |
|--------------|---------------|------------------------------|
| `CharIo` | Zero-size `MinimalIo` (0 bytes) | UART wrapper (~0-16 bytes) or buffered (~64-512 bytes) |
| `CredentialProvider` | Zero-size `MinCredentials` (0 bytes) | Static array or flash-backed (~4-32 bytes) |
| `CommandHandler` | Zero-size `MinHandlers` (0 bytes) | Stateless (0 bytes) or stateful (depends on fields) |

**Your actual sizes will be higher** based on your implementations.

### Buffer Configuration Impact

The minimal binary uses small buffers to isolate nut-shell overhead:

```rust
const MAX_INPUT: usize = 64;          // vs. 128 (default)
const MAX_PATH_DEPTH: usize = 4;      // vs. 8 (default)
const MAX_ARGS: usize = 8;            // vs. 16 (default)
const MAX_PROMPT: usize = 32;         // vs. 64 (default)
const MAX_RESPONSE: usize = 128;      // vs. 256 (default)
const HISTORY_SIZE: usize = 0;        // vs. 10 (default)
```

**RAM scales linearly with buffer sizes.** Doubling `MAX_INPUT` adds ~64 bytes RAM.

## Tools Required

The analysis script requires:

1. **Rust toolchain** with `thumbv6m-none-eabi` target
   ```bash
   rustup target add thumbv6m-none-eabi
   ```

2. **cargo-bloat** for symbol-level analysis
   ```bash
   cargo install cargo-bloat
   ```

3. **ARM GCC toolchain** (optional, for `arm-none-eabi-size`)
   - macOS: `brew install --cask gcc-arm-embedded`
   - Linux: `sudo apt-get install gcc-arm-none-eabi`
   - Fallback: Script uses `size` command if ARM tools unavailable

## Interpreting Results

### Feature Impact

Compare the "none" baseline with feature-enabled builds:

```
Feature Set          | Flash Delta | Purpose
---------------------|-------------|----------------------------------
authentication       | +2-3KB      | SHA-256 hashing, login state machine
completion           | +1-2KB      | Prefix matching, candidate search
history              | +0.5-1KB    | History buffer (scales with HISTORY_SIZE)
async                | +0.5-1KB    | Async runtime integration
```

### RAM vs. Flash Trade-offs

- **Flash**: Feature code (fixed cost per feature)
- **RAM**: Buffers and state (configurable via `ShellConfig`)

Example:
- Enabling `authentication` adds ~2KB Flash (fixed)
- Setting `HISTORY_SIZE = 10` adds ~1.3KB RAM (configurable)

### Optimization Tips

If you need to reduce size:

1. **Disable unused features**: Use `default-features = false`
2. **Reduce buffer sizes**: Adjust `ShellConfig` constants
3. **Minimize command tree**: Fewer commands = less metadata in `.rodata`
4. **Check symbol sizes**: Use `cargo-bloat` to identify large functions

## CI Integration

The `.github/workflows/size-analysis.yml` workflow:

- Runs on PRs and main branch pushes
- Generates size reports as artifacts
- Comments on PRs with summary table (informational only)
- Does NOT fail builds on size increases

Size increases may be justified for feature additions. The workflow provides data for informed decisions.

## Methodology Rationale

### Why Empty Directory Tree?

An empty tree isolates nut-shell's **baseline overhead** - the cost of the shell infrastructure itself (parser, navigation, I/O handling, feature logic). Your actual binary adds:

- Command metadata (const data in `.rodata`)
- Command implementations (functions in `.text`)
- Directory structure (const data in `.rodata`)

### Why Zero-Size Generics?

Your `CharIo`, `CredentialProvider`, and `CommandHandler` implementations are application-specific. Zero-size stubs ensure measurements reflect only nut-shell's contribution, not your custom logic.

### Why `opt-level = "z"`?

Embedded systems prioritize Flash size. The `"z"` optimization level (optimize for size) with LTO represents realistic production builds.

## Local Development

To test changes to the analysis infrastructure:

```bash
# Make script executable (if needed)
chmod +x analyze.sh

# Run analysis
./analyze.sh

# Check report
cat report.md

# Or open in editor
code report.md
```

The script is idempotent - safe to run multiple times.

## Files

```
size-analysis/
├── README.md              # This file
├── analyze.sh             # Analysis script
├── report.md              # Generated report (gitignored)
└── minimal/               # Minimal reference binary
    ├── Cargo.toml         # Minimal dependencies
    └── src/
        └── main.rs        # Empty tree + zero-size stubs
```

## Questions?

- **"Why is my binary larger?"** - You have commands, a real directory tree, and I/O implementations
- **"Which feature should I disable?"** - Compare feature delta vs. your use case needs
- **"How do I reduce RAM?"** - Adjust `ShellConfig` buffer sizes (see [../docs/EXAMPLES.md](../docs/EXAMPLES.md))
- **"Why aren't generic sizes shown?"** - They're user-defined; document your implementation sizes

For usage examples and configuration guidance, see [../docs/EXAMPLES.md](../docs/EXAMPLES.md).
