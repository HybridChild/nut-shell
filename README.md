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
- **Async commands** (`async` feature) - Natural async/await for long-running operations like network requests, flash I/O, and timers. Compatible with Embassy, RTIC, and other async runtimes. Zero overhead when disabled. *(Default: disabled)*
- **Authentication** (`authentication` feature) - SHA-256 password hashing, login flow, session management, and access control enforcement (~2KB flash) *(Default: enabled)*
- **Tab completion** (`completion` feature) - Command and path prefix matching (~2KB flash) *(Default: enabled)*
- **Command history** (`history` feature) - Arrow key navigation with configurable buffer (~0.5-1.3KB RAM) *(Default: enabled)*

### What We Explicitly Exclude
- ❌ Shell scripting (piping, variables, conditionals)
- ❌ Command aliases
- ❌ Output paging
- ❌ Persistent history across reboots
- ❌ Heap allocation

See [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) for rationale.

---

## Quick Start

### Basic Example (Bare-Metal)

```rust
use nut_shell::{Shell, CommandMeta, CommandKind, Directory, Node, AccessLevel};

// Define access levels
#[derive(Copy, Clone, PartialEq, PartialOrd)]
enum Level {
    User = 0,
    Admin = 1,
}

impl AccessLevel for Level {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "User" => Some(Self::User),
            "Admin" => Some(Self::Admin),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Admin => "Admin",
        }
    }
}

// Define commands
fn reboot_fn(args: &[&str]) -> Result<Response, CliError> {
    // Reboot implementation
    Ok(Response::success("Rebooting..."))
}

const REBOOT: CommandMeta<Level> = CommandMeta {
    name: "reboot",
    description: "Reboot the device",
    access_level: Level::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// Build command tree
const ROOT: Directory<Level> = Directory {
    name: "",
    children: &[
        Node::Directory(&Directory {
            name: "system",
            access_level: Level::User,
            children: &[
                Node::Command(&REBOOT),
            ],
        }),
    ],
    access_level: Level::User,
};

// Implement command handlers
struct MyHandlers;

impl CommandHandlers for MyHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response, CliError> {
        match name {
            "reboot" => reboot_fn(args),
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// Main loop (bare-metal)
#[entry]
fn main() -> ! {
    let uart = setup_uart();
    let mut io = UartIo::new(uart);
    let handlers = MyHandlers;
    let mut shell = Shell::new(&ROOT, handlers, &mut io);

    shell.activate().ok();

    loop {
        if let Ok(Some(c)) = io.get_char() {
            shell.process_char(c).ok();
        }
    }
}
```

### Async Example (Embassy)

```rust
use embassy_executor::Spawner;
use embassy_usb::class::cdc_acm::CdcAcmClass;

// Async command
async fn http_get_async(args: &[&str]) -> Result<Response, CliError> {
    let url = args[0];
    let response = HTTP_CLIENT.get(url).await?;
    Ok(Response::success(&response))
}

const HTTP_GET: CommandMeta<Level> = CommandMeta {
    name: "http-get",
    description: "Fetch URL via HTTP",
    access_level: Level::User,
    kind: CommandKind::Async,  // Mark as async
    min_args: 1,
    max_args: 1,
};

// Handler with async support
impl CommandHandlers for MyHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response, CliError> {
        match name {
            "reboot" => reboot_fn(args),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response, CliError> {
        match name {
            "http-get" => http_get_async(args).await,
            _ => Err(CliError::CommandNotFound),
        }
    }
}

#[embassy_executor::task]
async fn shell_task(usb: CdcAcmClass<'static, Driver<'static, USB>>) {
    let mut io = EmbassyUsbIo::new(usb);
    let handlers = MyHandlers;
    let mut shell = Shell::new(&ROOT, handlers, io);

    shell.activate().ok();
    io.flush().await.ok();

    let mut buffer = [0u8; 64];
    loop {
        let n = io.class.read_packet(&mut buffer).await.unwrap();

        for &byte in &buffer[..n] {
            shell.process_char_async(byte as char).await.ok();
        }

        io.flush().await.ok();
    }
}
```

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

### Flash (Code Size)
| Configuration | Size |
|---------------|------|
| Minimal (no features) | ~3-4KB |
| All features enabled | ~8-10KB |
| Authentication only | ~5-6KB |
| Interactive features (completion + history) | ~6-7KB |

### RAM
| Component | Default | Configurable |
|-----------|---------|--------------|
| Input buffer | 128 bytes | `MAX_INPUT` const generic |
| Path stack | 32 bytes | `MAX_PATH_DEPTH` const generic |
| Command history (N=10) | ~1.3KB | `HISTORY_SIZE` const generic |
| Command history (N=4) | ~0.5KB | RAM-constrained config |

**Minimal configuration:** ~0.2KB RAM (no history, minimal buffers)
**Default configuration:** ~1.5KB RAM (history enabled)

See [docs/EXAMPLES.md](docs/EXAMPLES.md) for buffer sizing guidance and configuration examples.

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
- `authentication` - User login and access control (default: enabled)
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

### Development
```bash
cargo check                   # Fast compile check
cargo test                    # Run tests
cargo clippy                  # Lint code
cargo fmt                     # Format code
```

### Feature Testing
```bash
cargo test --all-features                    # Test with all features
cargo test --no-default-features             # Test minimal config
cargo test --features authentication         # Test auth only
cargo test --features completion,history     # Test interactive features
```

### Embedded Target
```bash
cargo check --target thumbv6m-none-eabi                      # Verify no_std
cargo build --target thumbv6m-none-eabi --release            # Release build
cargo size --target thumbv6m-none-eabi --release -- -A       # Measure size
```

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for comprehensive build workflows and CI simulation.

---

## Contributing

Contributions are welcome! Before implementing new features, please review:

1. **[docs/PHILOSOPHY.md](docs/PHILOSOPHY.md)** - Understand what we include/exclude
2. **[docs/DESIGN.md](docs/DESIGN.md)** - Review design patterns and rationale
3. **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** - Follow build and testing workflows

**Feature requests:** Must align with embedded-first philosophy. Ask:
- Is this typical for embedded CLIs?
- Can terminal emulators/host tools handle this instead?
- What's the flash/RAM cost?
- Can it be feature-gated?

---

## Design Principles

1. **Simplicity over features** - Every feature is a liability
2. **Const over runtime** - Prefer compile-time decisions
3. **Embedded-first mindset** - Design for RP2040, not Linux
4. **Graceful degradation** - Features independently disable-able
5. **Security by design** - Either secure or explicitly unsecured
6. **Zero-cost abstractions** - Generics compile to optimal code
7. **Path-based philosophy** - Unix-style navigation
8. **Interactive discovery** - Learn through `?`, `ls`, tab completion

See [docs/PHILOSOPHY.md](docs/PHILOSOPHY.md) for complete framework.

---

## Success Metrics

A successful CLI library for embedded systems should:

- ✅ Compile on `thumbv6m-none-eabi` (RP2040 target)
- ✅ Fit in 32KB flash (with all default features)
- ✅ Use <8KB RAM (with default configuration)
- ✅ Zero heap allocation (pure stack + static)
- ✅ Enable feature toggling (each feature independently disable-able)
- ✅ Provide interactive UX (when features enabled)
- ✅ Degrade gracefully (minimal build still useful)
- ✅ Maintain security (when authentication enabled)
- ✅ Remain maintainable (<5000 lines of code total)
- ✅ Serve real use cases (actual embedded deployments)

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
