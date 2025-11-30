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

The analysis uses `size-analysis/minimal/` - a bare-bones embedded binary with:

- **Minimal command tree**: One sync command (`status`) and one async command (`info`) to prevent optimizer removal
- **MinimalConfig**: Reduced buffers and message strings to isolate nut-shell overhead
- **Minimal generics**: Stub implementations for `CharIo`, `CredentialProvider`, and `CommandHandler`
- **Real embedded target**: `thumbv6m-none-eabi` (Cortex-M0/M0+)
- **Size-optimized build**: `opt-level = "z"` with LTO enabled

### Feature Combinations Tested

1. **none** - Minimal build (no additional features)
2. **authentication** - Login and access control only
3. **completion** - Tab completion only
4. **history** - Command history only
5. **async** - Async support only
6. **completion,history** - Interactive features only (default configuration)
7. **all** - All features enabled

## Reading the Report

The generated `report.md` contains:

1. **Summary Table** - Quick overview of Flash/RAM usage for each feature combination
2. **Detailed Analysis** - Per-feature breakdown with:
   - Binary size by section (.text, .rodata, .data, .bss)
   - Top 10 largest symbols contributing to Flash usage
3. **Interpretation Guide** - Detailed explanations including:
   - Section meanings (.text, .rodata, .data, .bss)
   - How generic type sizes affect your total footprint
   - Buffer and message configuration costs
   - Feature-by-feature impact analysis

### Understanding the Output

**Quick reference for memory calculations:**

- **Flash usage** = `.text` + `.rodata` + `.data` (code + constants + initialized data)
- **RAM usage** = `.data` + `.bss` + stack (initialized data + zero-initialized + runtime stack)

The report's **Interpretation Guide** provides complete details on:
- What each section means and where it's stored
- How your trait implementations add to these baseline numbers
- Buffer configuration impact (RAM scales with buffer sizes)
- Message customization costs (Flash .rodata section)

### Key Insights from the Report

- **Baseline overhead**: Check the "none" configuration for minimum nut-shell footprint
- **Feature costs**: Compare each feature against baseline to see incremental cost
- **Symbol analysis**: Identify which functions consume the most Flash
- **Your actual costs**: The report explains how your trait implementations add to these numbers

## Using the Results

### Optimization Strategies

If you need to reduce size:

1. **Disable unused features**: Use `default-features = false` in your `Cargo.toml`
2. **Reduce buffer sizes**: Customize `ShellConfig` constants for your needs
3. **Minimize command tree**: Fewer commands = less metadata in Flash
4. **Check symbol sizes**: Use the symbol analysis to identify optimization targets

### Understanding Your Total Cost

The analysis measures **nut-shell's overhead** using minimal stub implementations. Your actual binary will be larger due to:

- Your command implementations (beyond the two minimal test commands)
- Your directory tree structure
- Your `CharIo`, `CredentialProvider`, and `CommandHandler` implementations
- Your chosen buffer sizes

See the report's "Interpretation Guide" for detailed explanations of how to calculate your total memory footprint.

## CI Integration

The `.github/workflows/size-analysis.yml` workflow:

- Runs on PRs and main branch pushes
- Generates size reports as artifacts
- Comments on PRs with summary table (informational only)
- Does NOT fail builds on size increases

Size increases may be justified for feature additions. The workflow provides data for informed decisions.
