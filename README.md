# nut-shell

> _Interactive CLI for microcontrollers. No heap, no bloat._

A lightweight command shell library for `no_std` Rust environments with optional async and authentication support.

[![Status](https://img.shields.io/badge/status-production--ready-brightgreen)](https://github.com/HybridChild/nut-shell)
[![Platform](https://img.shields.io/badge/platform-no_std-blue)](https://github.com/HybridChild/nut-shell)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](https://github.com/HybridChild/nut-shell)

---

## Overview

**nut-shell** provides essential CLI primitives for embedded systems with strict memory constraints. Built specifically for microcontrollers like the Raspberry Pi Pico (RP2040), it offers an interactive command-line interface over serial connections (UART/USB), with optional features including async/await support for long-running operations (Embassy, RTIC compatible), authentication, tab completion, and command history.

**Design Philosophy:** Essential primitives only. No shell scripting, no dynamic allocation, no bloat. See [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) for our feature decision framework.

---

## Key Features

### Core Functionality (Always Present)
- **Path-based navigation** - Unix-style hierarchical commands (`system/status`, `network/status`)
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
const STATUS: CommandMeta<Level> = CommandMeta { /* ... */ };
const ROOT: Directory<Level> = Directory { /* ... */ };

// 2. Implement CommandHandler trait
impl CommandHandler<MyConfig> for MyHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id {
            "status" => status_fn::<MyConfig>(args),
            _ => Err(CliError::CommandNotFound)
        }
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

**Tested platforms:**
- Raspberry Pi Pico (RP2040) - Primary development target
- STM32F072 - ARM Cortex-M0
- Native (std) - Testing and development

**Runtime environments:**
- Bare-metal (blocking I/O, polling loop)
- Embassy (async runtime with USB/UART)
- RTIC (real-time interrupt-driven concurrency)

**I/O abstraction:** Platform-agnostic `CharIo` trait for UART, USB-CDC, or custom adapters. See [docs/CHAR_IO.md](docs/CHAR_IO.md) for implementation guide.

---

## Memory Footprint

**Typical footprint** (measured on ARMv6-M with default features):
- **Flash:** ~4-6KB (core + completion + history)
- **RAM:** ~1.5KB (128-byte input buffer + 10-entry history)

**Minimal configuration:**
- **Flash:** ~1.6KB (no optional features)
- **RAM:** ~0.2KB (no history, minimal buffers)

**Your actual size** will be larger due to:
- Command implementations (simple GPIO ~50 bytes, network ~2-5KB each)
- Directory tree metadata (names, descriptions)
- CharIo/CredentialProvider/CommandHandler trait implementations

**For detailed analysis:** See [size-analysis/README.md](size-analysis/README.md) for methodology and complete breakdown across all feature combinations.

---

## Authentication & Security

Optional `authentication` feature provides:
- SHA-256 password hashing with per-user salts
- Constant-time comparison (prevents timing attacks)
- Password masking during input
- Access control enforced at every path segment
- Pluggable credential providers (build-time, flash storage, custom)

**See [docs/SECURITY.md](docs/SECURITY.md) for security architecture, threat model, and implementation patterns.**

---

## Documentation

| Document | Description |
|----------|-------------|
| **[README.md](README.md)** | Quick start and overview (this file) |
| **[docs/EXAMPLES.md](docs/EXAMPLES.md)** | Implementation patterns, configuration, troubleshooting |
| **[docs/CHAR_IO.md](docs/CHAR_IO.md)** | CharIo trait design and platform adapters |
| **[docs/SECURITY.md](docs/SECURITY.md)** | Authentication patterns and security considerations |
| **[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)** | Design philosophy and feature criteria |
| **[docs/DESIGN.md](docs/DESIGN.md)** | Architecture decisions and design patterns |
| **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** | Build workflows, testing, and CI |
| **`cargo doc --open`** | Complete API reference |

---

## Contributing

Contributions welcome! Review [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) for feature criteria and [docs/DESIGN.md](docs/DESIGN.md) for architectural patterns before implementing features.

**Before submitting:** Run `./scripts/ci-local` to verify all CI checks pass. See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for build workflows.

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
