# EXAMPLES

Practical implementation patterns for using nut-shell in your embedded projects.

**For architecture and additional resources, see:**
- **[DESIGN.md](DESIGN.md)** - Architecture decisions and patterns
- **[CHAR_IO.md](CHAR_IO.md)** - CharIo trait design and reference implementations
- **[SECURITY.md](SECURITY.md)** - Authentication and access control patterns

---

## Table of Contents

1. [Platform Examples](#platform-examples)
2. [Command Implementation Patterns](#command-implementation-patterns)
3. [Custom Configuration](#custom-configuration)
4. [Library Configuration Reference](#library-configuration-reference)

---

## Platform Examples

Complete working examples for specific platforms are in the `examples/` directory:

| Platform | Example | Description |
|----------|---------|-------------|
| **Native** | `examples/native_simple.rs` | Basic CLI with stdio (testing/development) |
| **RP2040** | `examples/pico_uart.rs` | Bare-metal UART with ISR-driven input queue |
| **Embassy** | `examples/embassy_usb_cdc.rs` | Async USB-CDC with buffered output |

**See each example for complete CharIo implementations and platform-specific setup.**

---

## Command Implementation Patterns

### Async Commands

```rust
use nut_shell::{DefaultConfig, Response, CliError, ShellConfig};

// Async command function (generic over config)
async fn http_get_async<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError> {
    let url = args[0];  // Shell validates argument count

    let result = embassy_time::with_timeout(
        embassy_time::Duration::from_secs(30),
        async { HTTP_CLIENT.get(url).await }
    ).await;

    match result {
        Ok(Ok(data)) => Ok(Response::success(&data)),
        Ok(Err(e)) => {
            let mut msg = heapless::String::new();
            write!(msg, "HTTP error: {:?}", e).ok();
            Err(CliError::CommandFailed(msg))
        }
        Err(_) => Err(CliError::CommandFailed("Request timeout".into())),
    }
}

// Mark as async in metadata
const HTTP_GET: CommandMeta<MyAccessLevel> = CommandMeta {
    id: "http_get",
    name: "http-get",
    description: "Fetch URL via HTTP",
    access_level: MyAccessLevel::User,
    kind: CommandKind::Async,  // Async marker
    min_args: 1,
    max_args: 1,
};

// Dispatch in handler
impl CommandHandler<DefaultConfig> for MyHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        Err(CliError::CommandNotFound)
    }

    #[cfg(feature = "async")]
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "http_get" => http_get_async::<DefaultConfig>(args).await,
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// Use process_char_async in main loop
let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&ROOT, handlers, io);
loop {
    if let Some(c) = io.get_char()? {
        shell.process_char_async(c).await?;
    }
}
```

### Stateful Handlers

```rust
use nut_shell::{DefaultConfig, CommandHandler, Response, CliError};

struct MyHandlers<'a> {
    system_state: &'a SystemState,
    config: &'a Config,
}

impl<'a> CommandHandler<DefaultConfig> for MyHandlers<'a> {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
        match id {
            "status" => {
                let info = self.system_state.get_status();
                Ok(Response::success(&info))
            }
            "config_get" => {
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
let handlers = MyHandlers {
    system_state: &system_state,
    config: &config
};
let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&ROOT, handlers, io);
```

### Custom CharIo Implementation

```rust
use nut_shell::CharIo;

pub struct UartIo {
    uart: Uart,
    rx_buffer: Deque<u8, 64>,
}

impl CharIo for UartIo {
    type Error = UartError;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        Ok(self.rx_buffer.pop_front().map(|b| b as char))
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.uart.write_byte(c as u8);
        Ok(())
    }
}
```

**See [CHAR_IO.md](CHAR_IO.md) for buffering patterns, async implementations, and platform-specific guidance.**

### Custom AccessLevel Implementation

Define your own access level hierarchy using the `AccessLevel` derive macro:

```rust
use nut_shell::AccessLevel;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum MyAccessLevel {
    Guest = 0,
    User = 1,
    Operator = 2,
    Admin = 3,
}

// Use in command metadata
const REBOOT: CommandMeta<MyAccessLevel> = CommandMeta {
    id: "reboot",
    name: "reboot",
    description: "Reboot system",
    access_level: MyAccessLevel::Admin,  // Only admins can reboot
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};
```

**The derive macro automatically implements:**
- `from_str()` - Converts variant names to enum values
- `as_str()` - Converts enum values to variant names

**Manual implementation** (if you need custom string mappings):

```rust
impl nut_shell::auth::AccessLevel for MyAccessLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "guest" => Some(Self::Guest),  // Custom lowercase mapping
            "user" => Some(Self::User),
            "operator" => Some(Self::Operator),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Guest => "guest",
            Self::User => "user",
            Self::Operator => "operator",
            Self::Admin => "admin",
        }
    }
}
```

**See [SECURITY.md](SECURITY.md) for access control security details.**

---

## Custom Configuration

### Custom ShellConfig

Implement `ShellConfig` to customize buffer sizes and user-visible messages:

```rust
use nut_shell::ShellConfig;

struct MyAppConfig;

impl ShellConfig for MyAppConfig {
    // Buffer sizes
    const MAX_INPUT: usize = 96;
    const MAX_PATH_DEPTH: usize = 6;
    const MAX_ARGS: usize = 12;
    const MAX_PROMPT: usize = 48;
    const MAX_RESPONSE: usize = 192;
    const HISTORY_SIZE: usize = 8;

    // Custom messages (stored in ROM)
    const MSG_WELCOME: &'static str = "MyDevice v1.0 Ready";
    const MSG_LOGIN_PROMPT: &'static str = "Login (user:pass): ";
    const MSG_LOGIN_SUCCESS: &'static str = "Access granted";
    const MSG_LOGIN_FAILED: &'static str = "Access denied";
    const MSG_LOGOUT: &'static str = "Session terminated";
    const MSG_INVALID_LOGIN_FORMAT: &'static str = "Format: username:password";
}

// Use custom config
let mut shell: Shell<_, MyAccessLevel, UartIo, MyHandlers, MyAppConfig> =
    Shell::new(&TREE, handlers, io);
```

**Use cases:**
- Fine-tune buffer sizes for your specific needs
- Brand your CLI with custom welcome messages
- Localize messages for different languages
- Match your device's personality

### Pre-Defined Configurations

**DefaultConfig** (recommended for most applications):
- MAX_INPUT: 128, MAX_RESPONSE: 256, HISTORY_SIZE: 10
- RAM usage: ~1.5 KB (with history enabled)

**MinimalConfig** (RAM-constrained systems):
- MAX_INPUT: 64, MAX_RESPONSE: 128, HISTORY_SIZE: 5
- RAM usage: ~0.5 KB (with history enabled)

```rust
use nut_shell::{Shell, DefaultConfig, MinimalConfig};

// Standard configuration
let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&TREE, handlers, io);

// Minimal configuration
let mut shell: Shell<_, _, _, _, MinimalConfig> = Shell::new(&TREE, handlers, io);
```

---

## Library Configuration Reference

### Buffer Sizing

Configure buffer sizes via the `ShellConfig` trait:

| Buffer | Default | Range | RAM Cost | Overflow Behavior |
|--------|---------|-------|----------|-------------------|
| MAX_INPUT | 128 | 64-256 | 128 bytes | Ignores excess chars |
| MAX_PATH_DEPTH | 8 | 4-16 | 32 bytes | Returns `PathTooDeep` |
| MAX_ARGS | 16 | 8-32 | 0 (stack) | Returns error |
| MAX_PROMPT | 64 | 32-128 | 64 bytes | Truncates prompt |
| MAX_RESPONSE | 256 | 128-512 | 256 bytes | Truncates response |
| HISTORY_SIZE | 10 | 0-20 | ~1.3 KB | Oldest dropped |

**RAM calculation example (DefaultConfig with history):**
```
128 + 32 + 64 + 256 + (10 × 128) = ~1.5 KB
```

### Feature Selection

| Feature | Default | Flash Cost | RAM Cost | Use Case |
|---------|---------|------------|----------|----------|
| `authentication` | disabled | +~2 KB | 0 bytes | User login and access control |
| `completion` | enabled | +~2 KB | 0 bytes | Tab completion for interactive use |
| `history` | enabled | +~0.5-0.8 KB | ~1.3 KB | Command recall with arrow keys |
| `async` | disabled | +~1.2-1.8 KB | 0 bytes | Async command execution |

**Common configurations:**
```toml
# Default - Interactive UX
[dependencies]
nut-shell = "0.1"

# Production with authentication
[dependencies]
nut-shell = { version = "0.1", features = ["authentication"] }

# Minimal (bootloaders, recovery mode)
[dependencies]
nut-shell = { version = "0.1", default-features = false }

# Async executor environments
[dependencies]
nut-shell = { version = "0.1", features = ["async"] }
```

### Common Issues

| Problem | Solution |
|---------|----------|
| Input buffer overflow | Increase `MAX_INPUT` in custom `ShellConfig` |
| PathTooDeep errors | Increase `MAX_PATH_DEPTH` in custom `ShellConfig` |
| High RAM usage | Use `MinimalConfig` or disable `history` feature |

---

**For complete platform integration guides, see `examples/` directory.**
**For CharIo implementation details, see [CHAR_IO.md](CHAR_IO.md).**
**For authentication patterns, see [SECURITY.md](SECURITY.md).**
