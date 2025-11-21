# nut-shell - Development Guide

**Status**: Production-ready library ✅

This document provides build commands, testing workflows, and troubleshooting guidance for developing and contributing to nut-shell.

---

## Quick Reference

**Common development tasks:**

```bash
# Development iteration
cargo check                              # Fast compile check
cargo test                               # Run tests (all features enabled by default)
cargo clippy                             # Lint code
cargo fmt                                # Format code
cargo run --example basic                # Manual testing

# Feature testing
cargo test --all-features                # Test with all features
cargo test --no-default-features         # Test minimal configuration
cargo test --features authentication     # Test specific feature

# Embedded verification
cargo check --target thumbv6m-none-eabi  # Verify no_std compliance
cargo build --target thumbv6m-none-eabi --release  # Release build
cargo size --target thumbv6m-none-eabi --release -- -A  # Measure binary size
```

---

## Build & Validation Commands

### Quick Iteration (Development)

Fast feedback during development:

```bash
cargo check                              # Fast compile check
cargo test                               # Run all tests
cargo test test_name                     # Run specific test
cargo clippy                             # Lint
cargo fmt                                # Format
cargo run --example basic                # Manual testing
```

### Feature Validation

Test all feature combinations to ensure graceful degradation:

```bash
# Test all feature combinations
cargo test --all-features
cargo test --no-default-features
cargo test --no-default-features --features authentication
cargo test --no-default-features --features completion
cargo test --no-default-features --features history

# Verify compilation with specific features
cargo check --features authentication
cargo check --features completion,history
cargo clippy --features completion
```

**Available features:**
- `authentication` - User authentication with credential providers
- `completion` - Tab completion for commands and paths
- `history` - Command history with arrow key navigation
- `async` - Asynchronous command execution (requires std)

### Embedded Target Verification

Verify `no_std` compliance and measure binary sizes:

```bash
# Verify no_std compliance
cargo check --target thumbv6m-none-eabi

# Build for embedded (various configurations)
cargo build --target thumbv6m-none-eabi --release
cargo build --target thumbv6m-none-eabi --release --no-default-features
cargo build --target thumbv6m-none-eabi --release --features authentication

# Measure and compare binary sizes
cargo size --target thumbv6m-none-eabi --release -- -A
cargo size --target thumbv6m-none-eabi --release --no-default-features -- -A

# Compare feature impact
cargo size --target thumbv6m-none-eabi --release --all-features -- -A
cargo size --target thumbv6m-none-eabi --release --features authentication -- -A
```

**Note**: The `async` feature is incompatible with `no_std` embedded targets as it requires an async runtime (typically `std`).

### Pre-Commit Validation

Run before committing changes:

```bash
# Full check (one-liner)
cargo fmt && \
cargo clippy --all-features -- -D warnings && \
cargo test --all-features && \
cargo check --target thumbv6m-none-eabi --release

# Or step-by-step:
cargo fmt                                          # 1. Format
cargo check --all-features                         # 2. Compile check
cargo clippy --all-features -- -D warnings         # 3. Lint (deny warnings)
cargo test --all-features                          # 4. Run tests
cargo check --target thumbv6m-none-eabi --release  # 5. Embedded check
cargo doc --no-deps --all-features                 # 6. Doc check
```

### CI Simulation (Full Validation)

Simulate complete CI pipeline locally:

```bash
# Test all feature combinations
cargo test --all-features
cargo test --no-default-features
cargo test --features authentication
cargo test --features completion
cargo test --features history

# Build all feature combinations
cargo build --all-features
cargo build --no-default-features
cargo build --features authentication
cargo build --features completion,history

# Embedded builds
cargo build --target thumbv6m-none-eabi --release --all-features
cargo build --target thumbv6m-none-eabi --release --no-default-features

# Quality checks
cargo fmt -- --check                     # Verify formatting
cargo clippy --all-features -- -D warnings         # Lint with all features
cargo clippy --no-default-features -- -D warnings  # Lint minimal config
cargo doc --no-deps --all-features       # Generate documentation
```

---

## Troubleshooting

### Compilation Issues

```bash
cargo build -vv                          # Verbose build output
cargo clean && cargo build               # Clean rebuild
cargo expand --lib                       # Expand macros (requires cargo-expand)
```

### Dependency Issues

```bash
cargo tree                               # Show dependency tree
cargo tree --target thumbv6m-none-eabi   # Embedded dependencies
cargo tree --format "{p} {f}"            # Show feature resolution
cargo update                             # Update dependencies
```

### Feature Resolution

If features aren't behaving as expected:

```bash
# Check which features are enabled
cargo tree --format "{p} {f}" | grep nut-shell

# Verify feature gates in code
cargo expand --lib | grep -A 5 "cfg(feature"

# Test specific feature combinations
cargo test --no-default-features --features authentication -vv
```

### Binary Size Analysis

For embedded targets, analyze binary size:

```bash
# Compare configurations
cargo size --target thumbv6m-none-eabi --release --all-features -- -A > size_all.txt
cargo size --target thumbv6m-none-eabi --release --no-default-features -- -A > size_min.txt
diff size_all.txt size_min.txt

# Detailed section breakdown
cargo bloat --target thumbv6m-none-eabi --release --all-features
cargo bloat --target thumbv6m-none-eabi --release --no-default-features

# Function-level analysis (requires cargo-bloat)
cargo bloat --target thumbv6m-none-eabi --release -n 20
```

### Testing Issues

```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test with backtrace
RUST_BACKTRACE=1 cargo test test_name -- --nocapture

# Run tests for specific feature
cargo test --features authentication -- --nocapture

# Run doc tests
cargo test --doc
```

---

## Project Structure

```
nut-shell/
├── src/
│   ├── lib.rs              # Library root, feature gates
│   ├── io.rs               # CharIo trait
│   ├── config.rs           # ShellConfig trait, default configs
│   ├── error.rs            # CliError enum
│   ├── response.rs         # Response type
│   ├── auth/               # Authentication module
│   │   ├── mod.rs          # AccessLevel, User, CredentialProvider
│   │   ├── password.rs     # Password hashing (Sha256Hasher)
│   │   └── providers/      # Credential provider implementations
│   ├── tree/               # Command tree module
│   │   ├── mod.rs          # Node, CommandMeta, Directory
│   │   ├── path.rs         # Path parsing
│   │   └── completion.rs   # Tab completion (feature-gated)
│   └── shell/              # Shell orchestration module
│       ├── mod.rs          # Shell struct, core methods
│       ├── handlers.rs     # CommandHandlers trait
│       ├── parser.rs       # InputParser, ParseEvent
│       └── history.rs      # CommandHistory (feature-gated)
├── tests/
│   ├── fixtures/           # Test fixtures (MockIo, MockAccessLevel, TEST_TREE)
│   ├── test_*.rs           # Integration tests
│   └── ...
├── examples/
│   ├── basic.rs            # Native stdio example
│   └── rp-pico/            # RP2040 embedded example (dedicated project)
└── docs/
    ├── README.md           # Documentation index
    ├── EXAMPLES.md         # Usage examples and patterns
    ├── DESIGN.md           # Architecture and design decisions
    ├── SECURITY.md         # Authentication and security
    ├── PHILOSOPHY.md       # Design philosophy
    ├── IO_DESIGN.md        # CharIo implementation guide
    └── DEVELOPMENT.md      # This file
```

---

## Test Suite

**Total test count**: 329 tests (all features), 228 tests (no features)

**Test categories:**
- Unit tests: Per-module testing (path parsing, tree navigation, input parsing, etc.)
- Integration tests: End-to-end CLI workflows
- Optimization tests: Zero-size types, const initialization, memory layout
- Doc tests: Documentation examples (11 tests)

**Running tests:**

```bash
# All tests with all features
cargo test --all-features

# Minimal configuration
cargo test --no-default-features

# Specific feature
cargo test --features authentication

# Specific test file
cargo test --test test_shell_integration

# Specific test
cargo test test_basic_command_execution
```

---

## Documentation

Generate and view API documentation:

```bash
# Generate docs
cargo doc --no-deps --all-features

# Generate and open in browser
cargo doc --no-deps --all-features --open

# Check for broken links
cargo doc --no-deps --all-features 2>&1 | grep warning
```

**Documentation locations:**
- API docs: `target/doc/nut_shell/index.html`
- User guide: `docs/EXAMPLES.md`
- Architecture: `docs/DESIGN.md`
- Security: `docs/SECURITY.md`

---

## Contributing

Before submitting a pull request:

1. **Run pre-commit validation** (see above)
2. **Add tests** for new functionality
3. **Update documentation** if adding public APIs
4. **Verify all feature combinations** compile and pass tests
5. **Check binary size impact** for embedded targets (if relevant)
6. **Follow existing patterns** (see `docs/DESIGN.md`)

**Code style:**
- Follow Rust standard style (`cargo fmt`)
- Address all clippy warnings (`cargo clippy -- -D warnings`)
- Add doc comments for public APIs
- Use descriptive variable names
- Keep functions focused and small

**Testing:**
- Write tests for all new functionality
- Include both unit and integration tests
- Test feature-gated code in both enabled/disabled states
- Verify `no_std` compliance for core functionality

---

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` (if present)
3. Run full CI simulation (see above)
4. Tag release: `git tag -a v0.1.0 -m "Release v0.1.0"`
5. Push tag: `git push origin v0.1.0`
6. Publish to crates.io: `cargo publish`

---

## Additional Resources

- **Main README**: Project overview and quick start
- **EXAMPLES.md**: Usage examples and configuration patterns
- **DESIGN.md**: Architecture decisions and patterns
- **PHILOSOPHY.md**: Design philosophy and feature framework
- **SECURITY.md**: Authentication and security design
- **IO_DESIGN.md**: CharIo trait and platform adapters
- **CLAUDE.md**: Claude Code integration (for AI-assisted development)
