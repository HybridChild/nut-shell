# nut-shell

> _Interactive CLI for microcontrollers. No heap, no bloat._

A lightweight command shell library for `no_std` Rust environments with optional async and authentication support.

[![Platform](https://img.shields.io/badge/platform-no_std-blue)](https://github.com/HybridChild/nut-shell)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](https://github.com/HybridChild/nut-shell)

---

## Overview

**nut-shell** provides essential CLI primitives for embedded systems with strict memory constraints. Built specifically for microcontrollers, it offers an interactive command-line interface over serial connections (UART/USB), with optional features including async/await support, authentication, tab completion and command history.

**Design Philosophy:** Essential primitives only. No shell scripting, no dynamic allocation, no bloat.

---

## Key Features

### Core Functionality (Always Present)
- **Path-based navigation** - Unix-style hierarchical commands (`system/status`, `network/status`)
- **Command execution** - Synchronous command support with structured argument parsing
- **Input parsing** - Terminal I/O with line editing (backspace, double-ESC clear)
- **Global commands** - `ls`, `?`, `clear`

### Optional Features
- **Tab completion** (`completion` feature) - Command and path prefix matching (~2KB flash) *(Default: enabled)*
- **Command history** (`history` feature) - Arrow key navigation with configurable buffer (~0.5-1.3KB RAM) *(Default: enabled)*
- **Async commands** (`async` feature) - Natural async/await for long-running operations like network requests, flash I/O, and timers. Compatible with Embassy and other async runtimes. Zero overhead when disabled. *(Default: disabled)*
- **Authentication** (`authentication` feature) - SHA-256 password hashing, login flow, session management, and access control enforcement (~2KB flash) *(Default: disabled - opt-in)*

### What We Explicitly Exclude
- ❌ Shell scripting (piping, variables, conditionals, command substitution)
- ❌ Command aliases
- ❌ Job control (background jobs, fg/bg)
- ❌ Output paging
- ❌ Persistent history across reboots

See [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) for rationale.

---

## Quick Start

**Bare-Metal Pattern:**
```rust
// 1. Implement `CharIo` trait for your platform
impl CharIo for MyIo {
    type Error = MyError;
    fn get_char(&mut self) -> Result<Option<char>, Self::Error> { /* ... */ }
    fn put_char(&mut self, c: char) -> Result<(), Self::Error> { /* ... */ }
}

// 2. Define command tree with metadata
const STATUS: CommandMeta<Level> = CommandMeta {
    id: "status",
    name: "status",
    description: "Show system status",
    access_level: Level::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM: Directory<Level> = Directory {
    name: "system",
    description: "System commands",
    access_level: Level::User,
    children: &[Node::Command(&STATUS)],
};

const ROOT: Directory<Level> = Directory {
    name: "",
    description: "Root",
    access_level: Level::Guest,
    children: &[Node::Directory(&SYSTEM)],
};

// 3. Implement `CommandHandler` trait
impl CommandHandler<MyConfig> for MyHandler {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id {
            "status" => status_fn::<MyConfig>(args),
            _ => Err(CliError::CommandNotFound)
        }
    }
}

// 4. Create shell and run main loop
let handler = MyHandler;    // Your `CommandHandler` implementation
let io = MyIo::new();       // Your `CharIo` implementation
let mut shell = Shell::new(&ROOT, handler, io);
shell.activate().ok();

loop {
    if let Some(c) = io.get_char()? {
        shell.process_char(c)?;
    }
}
```

**Async Pattern** (Embassy):
```rust
// 1. Define async command
const FETCH: CommandMeta<Level> = CommandMeta {
    id: "fetch",
    name: "fetch",
    description: "Fetch data from network",
    access_level: Level::User,
    kind: CommandKind::Async,  // Async command
    min_args: 0,
    max_args: 0,
};

// 2. Implement async handler
impl CommandHandler<MyConfig> for MyHandler {
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id {
            "fetch" => fetch_fn::<MyConfig>(args).await,
            _ => Err(CliError::CommandNotFound)
        }
    }
}

// 3. Run shell in async task
#[embassy_executor::task]
async fn shell_task(usb: CdcAcmClass<'static, Driver<'static, USB>>) {
    let handler = MyHandler;
    let io = MyIo::new(usb);
    let mut shell = Shell::new(&ROOT, handler, io);
    shell.activate().ok();

    loop {
        let c = read_char().await;
        shell.process_char_async(c).await?;
        io.flush().await?;
    }
}
```

---

## Platform Support

Built for `no_std` embedded systems:
- **Tested on:** ARM Cortex-M0 microcontrollers (RP2040, STM32F072)
- **Compatible with:** Any ARMv6-M or higher microcontroller

**Runtime compatibility:**
- Bare-metal (polling loop)
- Embassy (async runtime) and other async runtimes

**I/O abstraction:** Platform-agnostic `CharIo` trait for UART, USB-CDC, or custom adapters.

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
- `CharIo`/`CredentialProvider`/`CommandHandler` trait implementations

**For detailed analysis:** See [size-analysis/README.md](size-analysis/README.md) for methodology and complete breakdown across all feature combinations.

---

## Authentication & Security

Optional `authentication` feature provides:
- SHA-256 password hashing with per-user salts
- Constant-time comparison (prevents timing attacks)
- Password masking during input
- Access control enforced at every path segment
- Pluggable credential providers (build-time, flash storage, custom)

---

## Documentation

| Document | Description |
|----------|-------------|
| **[README.md](README.md)** | Quick start and overview (this file) |
| **[docs/EXAMPLES.md](docs/EXAMPLES.md)** | Implementation patterns, configuration, troubleshooting |
| **[docs/CHAR_IO.md](docs/CHAR_IO.md)** | `CharIo` trait design and platform adapters |
| **[docs/SECURITY.md](docs/SECURITY.md)** | Authentication patterns and security considerations |
| **[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)** | Design philosophy and feature criteria |
| **[docs/DESIGN.md](docs/DESIGN.md)** | Architecture decisions and design patterns |
| **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** | Build workflows, testing, and CI |
| **`cargo doc --open`** | Complete API reference |

---

## Contributing

Contributions welcome! Review [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) for feature criteria and [docs/DESIGN.md](docs/DESIGN.md) for architectural patterns before implementing features.

**Before submitting:** Run `./scripts/ci-local` to verify all CI checks pass.

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
