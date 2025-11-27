# nut-shell

> _A complete CLI framework for embedded systems, in a nutshell._

A lightweight, embedded-first command-line interface library for `no_std` Rust environments with optional async support.

[![Status](https://img.shields.io/badge/status-production--ready-brightgreen)](#project-status)
[![Platform](https://img.shields.io/badge/platform-no_std-blue)](#platform-support)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](#license)

---

## Overview

**nut-shell** provides essential CLI primitives for embedded systems with strict memory constraints. Built specifically for microcontrollers like the Raspberry Pi Pico (RP2040), it offers an interactive command-line interface over serial connections (UART/USB), with optional features including async/await support for long-running operations (Embassy, RTIC compatible), authentication, tab completion, and command history.

**Design Philosophy:** Essential primitives only. No shell scripting, no dynamic allocation, no bloat. See [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) for our feature decision framework.

---

## Project Status

✅ **Production-ready** - All implementation phases complete, fully tested and documented.

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for build instructions and [docs/EXAMPLES.md](docs/EXAMPLES.md) for usage guidance.

---

## Key Features

### Core Functionality (Always Present)
- **Path-based navigation** - Unix-style hierarchical commands (`system/reboot`, `network/status`)
- **Command execution** - Synchronous command support with structured argument parsing
- **Input parsing** - Terminal I/O with line editing (backspace, arrows, double-ESC clear)
- **Const initialization** - Zero runtime overhead, ROM placement
- **Global commands** - `ls`, `?`, `clear` (and `logout` when authentication enabled)

### Optional Features
- **Tab completion** (`completion` feature) - Command and path prefix matching (~2KB flash) *(Default: enabled)*
- **Command history** (`history` feature) - Arrow key navigation with configurable buffer (~0.5-1.3KB RAM) *(Default: enabled)*
- **Async commands** (`async` feature) - Natural async/await for long-running operations like network requests, flash I/O, and timers. Compatible with Embassy, RTIC, and other async runtimes. Zero overhead when disabled. *(Default: disabled)*
- **Authentication** (`authentication` feature) - SHA-256 password hashing, login flow, session management, and access control enforcement (~2KB flash) *(Default: disabled - opt-in)*

### What We Explicitly Exclude
- ❌ Shell scripting (piping, variables, conditionals)
- ❌ Command aliases
- ❌ Output paging
- ❌ Persistent history across reboots
- ❌ Heap allocation

See [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) for rationale.

---

## Quick Start

**Bare-Metal Pattern:**
```rust
// 1. Define command tree with metadata
const REBOOT: CommandMeta<Level> = CommandMeta { /* ... */ };
const ROOT: Directory<Level> = Directory { /* ... */ };

// 2. Implement CommandHandler trait
impl CommandHandler<MyConfig> for MyHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id { "reboot" => reboot_fn::<MyConfig>(args), _ => Err(CliError::CommandNotFound) }
    }
}

// 3. Main loop
let mut shell = Shell::new(&ROOT, handlers, io);
shell.activate().ok();
loop {
    if let Some(c) = io.get_char()? {
        shell.process_char(c)?;
    }
}
```

**Async Pattern** (Embassy):
```rust
#[embassy_executor::task]
async fn shell_task(usb: CdcAcmClass<'static, Driver<'static, USB>>) {
    let mut shell = Shell::new(&ROOT, handlers, io);
    shell.activate().ok();

    loop {
        let c = read_char().await;
        shell.process_char_async(c).await?;
        io.flush().await?;
    }
}
```

**See [docs/EXAMPLES.md](docs/EXAMPLES.md) for complete working examples with full code.**

---

## Platform Support

### Tested Platforms
- **Raspberry Pi Pico (RP2040)** - Primary development target
- **Native (std)** - Testing and development

### Runtime Environments
- **Bare-metal** - Blocking I/O, polling loop
- **Embassy** - Async runtime with USB/UART support
- **RTIC** - Real-time interrupt-driven concurrency

### I/O Adapters
The library uses a `CharIo` trait for platform-agnostic I/O:

```rust
pub trait CharIo {
    type Error;
    fn get_char(&mut self) -> Result<Option<char>, Self::Error>;
    fn put_char(&mut self, c: char) -> Result<(), Self::Error>;
}
```

**Buffering Model:**
- **Bare-metal:** `put_char()` writes directly to UART (blocking acceptable)
- **Async:** `put_char()` buffers to memory, `flush()` called externally after processing

See [docs/IO_DESIGN.md](docs/IO_DESIGN.md) for implementation details.

---

## Memory Footprint

Typical sizes on ARMv6-M (Cortex-M0+, thumbv6m-none-eabi target):

### Flash (Code Size)
| Feature Set | .text + .rodata |
|-------------|-----------------|
| Minimal (no features) | ~3-4KB |
| + authentication | ~5-6KB |
| + completion + history | ~6-7KB |
| All features enabled | ~8-10KB |

### RAM (Runtime)
| Component | Default | Configurable |
|-----------|---------|--------------|
| Input buffer | 128 bytes | `ShellConfig::MAX_INPUT` |
| Path stack | 32 bytes | `ShellConfig::MAX_PATH_DEPTH` |
| Command history (N=10) | ~1.3KB | `ShellConfig::HISTORY_SIZE` |
| Command history (N=4) | ~0.5KB | RAM-constrained config |

**Minimal configuration:** ~0.2KB RAM (no history, minimal buffers)
**Default configuration:** ~1.5KB RAM (history enabled)

### Understanding Generic Type Sizes

nut-shell is generic over user-provided types. Their sizes depend on YOUR implementation:

- **`CharIo`** (I/O): Minimal UART wrapper (~0-16 bytes) or buffered I/O (~64-512 bytes)
- **`CredentialProvider`** (auth): Static array or flash-backed storage (~4-32 bytes)
- **`CommandHandler`**: Stateless (0 bytes) or stateful (size of your state)

### Detailed Analysis

For exact measurements and symbol-level analysis across all feature combinations:

```bash
cd size-analysis
./analyze.sh
cat report.md
```

The analysis uses a minimal reference binary with an empty directory tree to isolate the pure overhead of nut-shell itself. Your actual binary will be larger due to your command implementations, directory tree structure, and I/O adapters.

See [size-analysis/README.md](size-analysis/README.md) for methodology and interpretation guide.

---

## Authentication & Security

When the `authentication` feature is enabled:

- **SHA-256 password hashing** with per-user salts
- **Constant-time comparison** to prevent timing attacks
- **Password masking** during input (shows `*` after colon)
- **Access control** enforced at every path segment
- **Pluggable credential providers** (build-time, flash storage, custom)

```rust
// Login format
> admin:********
  Logged in. Type ? for help.

admin@/> system/reboot
  Rebooting...

admin@/> logout
  Logged out.
```

**Credential Storage Options:**
1. **Build-time environment variables** - Hashed credentials configured during build
2. **Flash storage** - Per-device unique credentials (production recommended)
3. **Const provider** - Hardcoded for examples/testing only
4. **Custom provider** - Trait-based extensibility (LDAP, HSM, etc.)

See [docs/SECURITY.md](docs/SECURITY.md) for security architecture and threat model.

---

## Configuration

### Cargo Features

```toml
[dependencies]
nut-shell = { version = "0.1", features = ["authentication", "completion", "history", "async"] }

# Or minimal build
nut-shell = { version = "0.1", default-features = false }
```

**Available features:**
- `authentication` - User login and access control (default: disabled - opt-in)
- `completion` - Tab completion for commands/paths (default: enabled)
- `history` - Command history with arrow keys (default: enabled)
- `async` - Async command execution support (default: disabled)

### Const Generics

Customize buffer sizes at compile time:

```rust
type InputBuffer = heapless::String<64>;  // Default: 128
type PathStack = heapless::Vec<usize, 4>;  // Default: 8
type History = CommandHistory<4>;  // Default: 10
```

See [docs/EXAMPLES.md](docs/EXAMPLES.md) for complete configuration examples, CharIo implementation patterns, and troubleshooting guide.

---

## Documentation

### For Library Users
- **[README.md](README.md)** (this file) - Quick start and overview
- **[docs/EXAMPLES.md](docs/EXAMPLES.md)** - Complete examples, configuration guide, and troubleshooting
- **[docs/IO_DESIGN.md](docs/IO_DESIGN.md)** - CharIo trait and platform adapter implementation guide
- **[docs/SECURITY.md](docs/SECURITY.md)** - Authentication patterns and security considerations
- **Run `cargo doc --open`** - Complete API reference

### For Contributors
- **[CLAUDE.md](CLAUDE.md)** - AI-assisted development guidance
- **[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)** - Design philosophy and feature criteria
- **[docs/DESIGN.md](docs/DESIGN.md)** - Architecture decisions and design patterns
- **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** - Build commands, testing workflows, and CI

See [docs/README.md](docs/README.md) for complete documentation navigation.

---

## Build Commands

```bash
cargo test --all-features                    # Test all features
cargo test --no-default-features             # Test minimal
cargo check --target thumbv6m-none-eabi      # Verify no_std
```

**See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for complete workflows.**

---

## Contributing

Contributions welcome! Review [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) for feature criteria and [docs/DESIGN.md](docs/DESIGN.md) for architectural patterns before implementing features.

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

## Acknowledgments

Designed for the Rust embedded ecosystem, with inspiration from:
- Unix shell navigation and commands
- Embedded CLI best practices
- `no_std` Rust patterns
- Embassy async runtime architecture

**Maintained by:** Esben Dueholm Nørgaard ([HybridChild](https://github.com/HybridChild))
