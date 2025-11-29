# DEVELOPMENT

**Status**: Production-ready library ✅

---

## Quick Reference

```bash
# Pre-commit validation (run before committing)
cargo fmt && \
cargo clippy --all-features -- -D warnings && \
cargo test --all-features && \
cargo check --target thumbv6m-none-eabi --release

# Feature testing (test graceful degradation)
cargo test --all-features
cargo test --no-default-features
cargo test --features authentication

# Embedded verification (no_std compliance)
cargo check --target thumbv6m-none-eabi
cargo size --target thumbv6m-none-eabi --release -- -A
```

**Available features:**
- `authentication` - User authentication with credential providers
- `completion` - Tab completion for commands and paths
- `history` - Command history with arrow key navigation
- `async` - Asynchronous command execution support

**Note:** Both the library and tests are fully `no_std` - tests use `heapless` types to maintain consistency with the library's embedded patterns.

---

## Troubleshooting

### Feature Resolution Issues

```bash
# Check which features are actually enabled
cargo tree --format "{p} {f}" | grep nut-shell

# Verify feature gates in expanded code
cargo expand --lib | grep -A 5 "cfg(feature"
```

### Binary Size Analysis

```bash
# Compare feature impact on binary size
cargo bloat --target thumbv6m-none-eabi --release --all-features
cargo bloat --target thumbv6m-none-eabi --release --no-default-features
```


---

## Contributing Checklist

Before submitting a pull request:

1. Run pre-commit validation (see Quick Reference)
2. Test feature-gated code in both enabled/disabled states
3. Verify `no_std` compliance for core functionality
4. Check binary size impact with `cargo bloat` (if relevant)
5. Update documentation for public API changes
6. Follow existing patterns (see `docs/DESIGN.md`)

---

## Documentation

- **Quick start**: `README.md`
- **Usage examples**: `docs/EXAMPLES.md`
- **Architecture**: `docs/DESIGN.md`
- **Security**: `docs/SECURITY.md`
- **API reference**: `cargo doc --open`
