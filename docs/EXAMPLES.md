# nut-shell Examples and Tutorials

This document provides practical examples, configuration guidance, and tutorials for using nut-shell in your embedded projects.

**For architecture and additional resources, see:**
- **[DESIGN.md](DESIGN.md)** - Architecture decisions and patterns
- **[IO_DESIGN.md](IO_DESIGN.md)** - CharIo trait design and reference implementations
- **[SECURITY.md](SECURITY.md)** - Authentication and access control patterns

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Buffer Sizing Guide](#buffer-sizing-guide)
3. [Platform Examples](#platform-examples)
4. [Configuration Examples](#configuration-examples)
5. [Common Patterns](#common-patterns)
6. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Minimal Example (Native)

```rust
use nut_shell::{
    Shell, CharIo, CommandHandler, Response, CliError,
    AccessLevel, CommandMeta, CommandKind, Directory, Node, DefaultConfig
};

// Define your access levels
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum MyAccessLevel {
    User = 0,
    Admin = 1,
}

impl AccessLevel for MyAccessLevel {
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

// Define command handlers
struct MyHandlers;

impl CommandHandler<DefaultConfig> for MyHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match name {
            "hello" => Ok(Response::success("Hello, World!")),
            "echo" => Ok(Response::success(args.get(0).unwrap_or(&""))),
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// Define command tree
const HELLO_CMD: CommandMeta<MyAccessLevel> = CommandMeta {
    id: "hello",
    name: "hello",
    description: "Print hello world",
    access_level: MyAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const ROOT: Directory<MyAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Command(&HELLO_CMD),
    ],
    access_level: MyAccessLevel::User,
};

fn main() {
    let io = StdioIo::new();  // Your CharIo implementation
    let handlers = MyHandlers;
    let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&ROOT, handlers, io);

    shell.activate().unwrap();

    // Main loop
    loop {
        if let Ok(Some(c)) = io.get_char() {
            shell.process_char(c).ok();
        }
    }
}
```

---

## Buffer Sizing Guide

Configure buffer sizes via the `ShellConfig` trait. Choose based on your command complexity and RAM constraints.

| Buffer | Default | Range | RAM Cost | When to Adjust | Overflow Behavior |
|--------|---------|-------|----------|----------------|-------------------|
| **MAX_INPUT** | 128 | 64-256 | 128 bytes | Commands exceed length | Ignores excess chars |
| **MAX_OUTPUT** (async only) | 256 | 128-1024 | 256 bytes | Responses truncated | Returns error |
| **MAX_PATH_DEPTH** | 8 | 4-16 | 32 bytes | Tree depth >8 levels | Returns `PathTooDeep` |
| **MAX_ARGS** | 16 | 8-32 | 0 (stack) | Commands >16 args | Returns error |
| **MAX_PROMPT** | 64 | 32-128 | 64 bytes | Long usernames/paths | Truncates prompt |
| **MAX_RESPONSE** | 256 | 128-512 | 256 bytes | Multi-line output | Truncates response |
| **HISTORY_SIZE** | 10 | 0-20 | ~1.3 KB | More/less history | Oldest dropped |

**RAM calculation example (DefaultConfig with history):**
```
128 + 256 + 32 + 64 + 256 + (10 × 128) = ~2 KB
```

**Disable history entirely:** `cargo build --no-default-features --features completion`

---

## Configuration Examples

Configuration is done via the `ShellConfig` trait, which defines buffer sizes and user-visible messages at compile time.

### Using Pre-Defined Configurations

#### Standard Configuration (Recommended)

Use `DefaultConfig` for most embedded applications:

```rust
use nut_shell::{Shell, DefaultConfig};

let mut shell: Shell<_, MyAccessLevel, UartIo, MyHandlers, DefaultConfig> =
    Shell::new(&TREE, handlers, io);

// Or with type inference:
let mut shell = Shell::<_, _, _, _, DefaultConfig>::new(&TREE, handlers, io);
```

**Buffer sizes:**
- MAX_INPUT: 128 bytes
- MAX_PATH_DEPTH: 8 levels
- MAX_ARGS: 16 arguments
- MAX_PROMPT: 64 bytes
- MAX_RESPONSE: 256 bytes
- HISTORY_SIZE: 10 commands

**Total RAM:** ~1.5 KB (with history enabled)

**Use for:**
- Production embedded devices (RP2040, STM32, nRF52, ESP32)
- Interactive debugging interfaces
- General-purpose embedded CLIs

#### Minimal Configuration

Use `MinimalConfig` for RAM-constrained systems:

```rust
use nut_shell::{Shell, MinimalConfig};

let mut shell: Shell<_, MyAccessLevel, UartIo, MyHandlers, MinimalConfig> =
    Shell::new(&TREE, handlers, io);
```

**Buffer sizes:**
- MAX_INPUT: 64 bytes
- MAX_PATH_DEPTH: 4 levels
- MAX_ARGS: 8 arguments
- MAX_PROMPT: 32 bytes
- MAX_RESPONSE: 128 bytes
- HISTORY_SIZE: 5 commands

**Total RAM:** ~0.5 KB (with history enabled)

**Use for:**
- Bootloaders
- Recovery mode interfaces
- Systems with <2KB RAM available

### Custom Configurations

Create custom configurations by implementing `ShellConfig`:

```rust
use nut_shell::{Shell, ShellConfig};

// Define custom config
struct MyCustomConfig;

impl ShellConfig for MyCustomConfig {
    // Buffer sizes
    const MAX_INPUT: usize = 96;
    const MAX_PATH_DEPTH: usize = 6;
    const MAX_ARGS: usize = 12;
    const MAX_PROMPT: usize = 48;
    const MAX_RESPONSE: usize = 192;
    const HISTORY_SIZE: usize = 8;

    // Custom messages (all stored in ROM)
    const MSG_WELCOME: &'static str = "🚀 MyDevice v1.0 Ready\r\n";
    const MSG_LOGIN_PROMPT: &'static str = "Login (user:pass): ";
    const MSG_LOGIN_SUCCESS: &'static str = "✓ Access granted\r\n";
    const MSG_LOGIN_FAILED: &'static str = "✗ Access denied\r\n";
    const MSG_LOGOUT: &'static str = "Session terminated\r\n";
    const MSG_INVALID_LOGIN_FORMAT: &'static str = "Format: username:password\r\n";
}

// Use custom config
let mut shell: Shell<_, MyAccessLevel, UartIo, MyHandlers, MyCustomConfig> =
    Shell::new(&TREE, handlers, io);
```

**Benefits of custom configs:**
- Fine-tune buffer sizes for your specific needs
- Customize all user-visible messages (branding, localization)
- Zero runtime overhead (all values are const)

### Configuration Comparison

| Configuration | MAX_INPUT | PATH_DEPTH | ARGS | HISTORY | RAM Usage |
|---------------|-----------|------------|------|---------|-----------|
| MinimalConfig | 64 | 4 | 8 | 5 | ~0.5 KB |
| DefaultConfig | 128 | 8 | 16 | 10 | ~1.5 KB |
| Custom (high) | 256 | 12 | 32 | 20 | ~3.0 KB |

### Feature Combinations

Choose Cargo features based on your requirements:

```toml
[dependencies]
nut-shell = { version = "0.1", features = ["authentication", "completion", "history"] }

# Or minimal build
nut-shell = { version = "0.1", default-features = false }

# Or specific features
nut-shell = { version = "0.1", default-features = false, features = ["authentication"] }
```

**Available features:**
- `authentication` - User login and access control (default: disabled - opt-in)
- `completion` - Tab completion for commands/paths (default: enabled)
- `history` - Command history with arrow keys (default: enabled)
- `async` - Async command execution support (default: disabled)

**Build examples:**
```bash
# Default (completion + history)
cargo build

# All features including authentication
cargo build --features authentication

# Minimal (no optional features)
cargo build --no-default-features

# Authentication only
cargo build --no-default-features --features authentication

# Interactive only (completion + history, same as default)
cargo build --no-default-features --features completion,history

# With async support
cargo build --features async
```

### Estimating RAM Usage

Calculate your RAM requirements:

```
Input buffer:      MAX_INPUT bytes
Path stack:        MAX_PATH_DEPTH * 4 bytes (usize indices)
History (enabled): HISTORY_SIZE * MAX_INPUT bytes
History (disabled): 0 bytes

Example (DefaultConfig with history):
  128 + (8 * 4) + (10 * 128) = 128 + 32 + 1280 = ~1.5 KB
```

**Note:** Shell struct overhead is minimal (~50-100 bytes for other fields)

---

## Platform Examples

Each example shows the key nut-shell integration points. See `examples/` directory for complete working code.

### Bare-Metal UART (RP2040)

**Key pattern:** ISR fills queue, main loop processes

```rust
// ISR fills global queue (platform boilerplate)
static RX_QUEUE: Mutex<RefCell<Deque<u8, 64>>> = /* ... */;

// CharIo implementation
impl CharIo for UartIo {
    fn get_char(&mut self) -> Result<Option<char>> {
        let byte = cortex_m::interrupt::free(|cs| {
            RX_QUEUE.borrow(cs).borrow_mut().pop_front()
        });
        Ok(byte.map(|b| b as char))
    }

    fn put_char(&mut self, c: char) -> Result<()> {
        self.uart.write_byte(c as u8);  // Blocking OK
        Ok(())
    }
}

// Main loop
let mut shell = Shell::new(&ROOT, handlers, io);
shell.activate()?;
loop {
    if let Some(c) = io.get_char()? {
        shell.process_char(c)?;
    }
}
```

### Embassy USB-CDC (Async)

**Key pattern:** Buffer output, flush after processing

```rust
struct UsbCdcIo<'d> {
    class: CdcAcmClass<'d, Driver<'d>>,
    tx_buffer: Vec<u8, 256>,
}

impl CharIo for UsbCdcIo<'_> {
    fn put_char(&mut self, c: char) -> Result<()> {
        // Normalize line endings
        let bytes = if c == '\r' { b"\r\n" } else { &[c as u8] };
        for &b in bytes {
            self.tx_buffer.push(b)?;
        }
        Ok(())
    }
}

// Async task
loop {
    let c = read_from_usb().await;
    shell.process_char_async(c).await?;
    io.flush().await?;  // Flush once per char
}
```

### Native (Testing)

```rust
impl CharIo for StdioIo {
    fn get_char(&mut self) -> Result<Option<char>> {
        let mut buf = [0u8; 1];
        Ok(match io::stdin().read(&mut buf)? {
            1 => Some(buf[0] as char),
            _ => None,
        })
    }

    fn put_char(&mut self, c: char) -> Result<()> {
        print!("{}", c);
        io::stdout().flush()
    }
}
```

---

## Common Patterns

### Pattern: Async Commands

```rust
use nut_shell::{DefaultConfig, Response};

// Define async command function (generic over config)
async fn http_get_async<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError> {
    // Shell validates argument count, so args[0] is guaranteed to exist if min_args >= 1
    let url = args[0];

    // Use embassy_time for timeout
    let result = embassy_time::with_timeout(
        embassy_time::Duration::from_secs(30),
        async {
            HTTP_CLIENT.get(url).await
        }
    ).await;

    match result {
        Ok(Ok(data)) => Ok(Response::success(&data)),
        Ok(Err(e)) => {
            let mut msg = heapless::String::new();
            write!(msg, "HTTP error: {:?}", e).ok();
            Err(CliError::CommandFailed(msg))
        }
        Err(_) => {
            let mut msg = heapless::String::new();
            msg.push_str("Request timeout").unwrap();
            Err(CliError::CommandFailed(msg))
        }
    }
}

// Mark as async in metadata
const HTTP_GET: CommandMeta<MyAccessLevel> = CommandMeta {
    id: "http_get",
    name: "http-get",
    description: "Fetch URL via HTTP",
    access_level: MyAccessLevel::User,
    kind: CommandKind::Async,  // Mark as async
    min_args: 1,
    max_args: 1,
};

// Dispatch in handler
impl CommandHandler<DefaultConfig> for MyHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        // ... sync commands
        Err(CliError::CommandNotFound)
    }

    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match name {
            "http-get" => http_get_async::<DefaultConfig>(args).await,
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// Use process_char_async in main loop
let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&ROOT, handlers, io);
loop {
    if let Some(c) = io.get_char()? {
        shell.process_char_async(c).await?;  // Async processing
    }
}
```

### Pattern: Stateful Handlers

```rust
use nut_shell::{DefaultConfig, CommandHandler, Response};

struct MyHandlers<'a> {
    system_state: &'a SystemState,
    config: &'a Config,
}

impl<'a> CommandHandler<DefaultConfig> for MyHandlers<'a> {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match name {
            "status" => {
                let info = self.system_state.get_status();
                Ok(Response::success(&info))
            }
            "config-get" => {
                let value = self.config.get(args[0]);
                Ok(Response::success(value))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// Usage
let system_state = SystemState::new();
let config = Config::load();
let handlers = MyHandlers { system_state: &system_state, config: &config };
let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&ROOT, handlers, io);
```

### Pattern: Custom Access Levels

```rust
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum MyAccessLevel {
    Guest = 0,
    User = 1,
    Operator = 2,
    Admin = 3,
}

// Fine-grained control
const REBOOT: CommandMeta<MyAccessLevel> = CommandMeta {
    access_level: MyAccessLevel::Admin,  // Only admins
    // ...
};

const STATUS: CommandMeta<MyAccessLevel> = CommandMeta {
    access_level: MyAccessLevel::Guest,  // Everyone
    // ...
};
```

### Pattern: Build-Time Credentials

```rust
// Use environment variables at build time
const ADMIN_SALT: [u8; 16] = *b"random_salt_0001";
const ADMIN_HASH: [u8; 32] = /* hash of env!("ADMIN_PASSWORD") with salt */;

struct BuildTimeProvider;

impl CredentialProvider<MyAccessLevel> for BuildTimeProvider {
    fn find_user(&self, username: &str) -> Option<User<MyAccessLevel>> {
        match username {
            "admin" => Some(User {
                username: heapless::String::from("admin"),
                password_hash: ADMIN_HASH,
                salt: ADMIN_SALT,
                access_level: MyAccessLevel::Admin,
            }),
            _ => None,
        }
    }

    fn verify_password(&self, user: &User<MyAccessLevel>, password: &str) -> bool {
        let hasher = Sha256Hasher;
        hasher.verify(password, &user.salt, &user.password_hash)
    }
}

// Build with:
// ADMIN_PASSWORD=secret123 cargo build
```

---

## Troubleshooting

### Library Configuration

| Problem | Solution |
|---------|----------|
| Input buffer overflow | Increase `MAX_INPUT` in custom `ShellConfig` |
| Output buffer overflow (async) | Increase `tx_buffer` size in CharIo implementation |
| PathTooDeep errors | Increase `MAX_PATH_DEPTH` in custom `ShellConfig` |
| High RAM usage | Use `MinimalConfig`, reduce buffer sizes, disable `history` feature |

### Platform Issues

| Problem | Solution |
|---------|----------|
| UART drops characters | Increase ISR buffer (`Deque<u8, 128>`), enable flow control |
| USB disconnect | Implement reconnect detection in CharIo `get_char()` |
| Async commands block | Expected behavior - commands should implement timeouts or spawn background tasks |
| High CPU (async) | Use `.await` instead of polling loops |

### Terminal Configuration

| Problem | Solution |
|---------|----------|
| Double echo | Disable local echo: `stty -echo` |
| Backspace not working | CLI handles both BS/DEL - check terminal config: `stty erase ^H` |

**See [IO_DESIGN.md](IO_DESIGN.md) for CharIo implementation guidance.**

---

**For CharIo implementation details, buffering patterns, and platform-specific guides, see [IO_DESIGN.md](IO_DESIGN.md).**
