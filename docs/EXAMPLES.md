# EXAMPLES

Practical implementation patterns for using nut-shell in your embedded projects.

## Table of Contents

1. [Complete Platform Examples](#complete-platform-examples)
2. [Command Patterns](#command-patterns)
3. [Trait Implementations](#trait-implementations)
4. [Configuration](#configuration)

---

## Complete Platform Examples

Working examples in `examples/` directory:

| Platform | Path | Key Features |
|----------|------|--------------|
| **Native (sync)** | `examples/native/bin/basic/` | Stdio with immediate flush |
| **Native (async)** | `examples/native/bin/async/` | Buffered stdio, Embassy runtime |
| **RP2040 (bare-metal)** | `examples/rp-pico/bin/basic/` | Blocking USB-CDC writes |
| **RP2040 (Embassy)** | `examples/rp-pico/bin/embassy/` | Async USB-CDC with buffered output |
| **STM32F072** | `examples/stm32f072/bin/basic/` | Blocking UART writes |

Each example includes complete `CharIo` implementation in `io.rs`.

---

## Command Patterns

### Synchronous Commands

```rust
// 1. Define command metadata
const STATUS: CommandMeta<MyAccessLevel> = CommandMeta {
    id: "status",
    name: "status",
    description: "Show system status",
    access_level: MyAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// 2. Implement command function
fn status_fn<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError> {
    Ok(Response::success("System OK"))
}

// 3. Add to tree
const SYSTEM: Directory<MyAccessLevel> = Directory {
    name: "system",
    description: "System commands",
    access_level: MyAccessLevel::User,
    children: &[Node::Command(&STATUS)],
};

// 4. Dispatch in handler
impl CommandHandler<MyConfig> for MyHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id {
            "status" => status_fn::<MyConfig>(args),
            _ => Err(CliError::CommandNotFound),
        }
    }
}
```

### Async Commands

```rust
// 1. Define metadata (mark as Async)
const FETCH: CommandMeta<MyAccessLevel> = CommandMeta {
    id: "fetch",
    name: "fetch",
    description: "Fetch data from network",
    access_level: MyAccessLevel::User,
    kind: CommandKind::Async,  // Async marker
    min_args: 1,
    max_args: 1,
};

// 2. Implement async command function
async fn fetch_fn<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError> {
    let data = HTTP_CLIENT.get(args[0]).await?;
    Ok(Response::success(&data))
}

// 3. Dispatch in async handler
#[cfg(feature = "async")]
impl CommandHandler<MyConfig> for MyHandlers {
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id {
            "fetch" => fetch_fn::<MyConfig>(args).await,
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// 4. Use process_char_async in main loop
loop {
    let c = io.read().await;
    shell.process_char_async(c).await?;
}
```

**See [DESIGN.md](DESIGN.md) for metadata/execution separation architecture.**

### Stateful Handlers

```rust
struct MyHandlers<'a> {
    system_state: &'a SystemState,
    config: &'a Config,
}

impl<'a> CommandHandler<MyConfig> for MyHandlers<'a> {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
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
```

---

## Trait Implementations

### `AccessLevel`

Define access hierarchy using the derive macro:

```rust
use nut_shell_macros::AccessLevel;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum MyAccessLevel {
    Guest = 0,
    User = 1,
    Admin = 2,
}
```

The macro generates `from_str` and `as_str` methods for string conversion. Higher numeric values inherit permissions from lower levels due to `PartialOrd`.

**See [SECURITY.md](SECURITY.md#access-control-system) for access control patterns.**

### `CommandHandler`

Maps command IDs to execution functions:

```rust
struct MyHandlers;

impl CommandHandler<MyConfig> for MyHandlers {
    fn execute_sync(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id {
            "status" => status_fn::<MyConfig>(args),
            "info" => info_fn::<MyConfig>(args),
            _ => Err(CliError::CommandNotFound),
        }
    }

    #[cfg(feature = "async")]
    async fn execute_async(&self, id: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match id {
            "fetch" => fetch_fn::<MyConfig>(args).await,
            _ => Err(CliError::CommandNotFound),
        }
    }
}
```

**See [Command Patterns](#command-patterns) for complete examples.**

### `CharIo`

Platform I/O abstraction for character input/output:

```rust
struct MyIo { /* platform-specific fields */ }

impl CharIo for MyIo {
    type Error = MyError;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Return Some(char) if available, None if no input ready
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Write character to output
    }
}
```

**See [CHAR_IO.md](CHAR_IO.md) for platform adapter patterns and buffering strategies.**

---

## Configuration

### Pre-Defined Configs

| Config | MAX_INPUT | MAX_RESPONSE | HISTORY_SIZE | RAM Usage | Use Case |
|--------|-----------|--------------|--------------|-----------|----------|
| `DefaultConfig` | 128 | 256 | 10 | 1.9 KB (with history)<br>0.6 KB (without) | Standard applications |
| `MinimalConfig` | 64 | 128 | 4 | 0.7 KB (with history)<br>0.2 KB (without) | RAM-constrained systems |

**Note:** Due to const generics limitations, buffer sizes are currently hardcoded to `DefaultConfig` values. `MinimalConfig` RAM usage reflects intended values when const generics are fully supported. Currently both configs use identical RAM (`DefaultConfig` values).

```rust
use nut_shell::{Shell, DefaultConfig, MinimalConfig};

// Standard
let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&ROOT, handlers, io);

// Minimal
let mut shell: Shell<_, _, _, _, MinimalConfig> = Shell::new(&ROOT, handlers, io);
```

### Custom `ShellConfig`

```rust
struct MyConfig;

impl ShellConfig for MyConfig {
    const MAX_INPUT: usize = 96;
    const MAX_RESPONSE: usize = 192;
    const HISTORY_SIZE: usize = 6;

    const MSG_WELCOME: &'static str = "MyDevice v1.0";
    // ... other buffer sizes and messages
}

let mut shell: Shell<_, _, _, _, MyConfig> = Shell::new(&ROOT, handlers, io);
```

**Currently customizable:**
- Message strings (`MSG_WELCOME`, `MSG_LOGIN_PROMPT`, etc.)

**Not yet functional:** Buffer size constants (`MAX_INPUT`, `MAX_RESPONSE`, etc.) are hardcoded pending const generics stabilization. See `src/config.rs` for details.

### Feature Flags

| Feature | Default | Use Case |
|---------|---------|----------|
| `completion` | ✅ Enabled | Tab completion for interactive use |
| `history` | ✅ Enabled | Arrow key command recall |
| `authentication` | ❌ Disabled | User login and access control |
| `async` | ❌ Disabled | Async command execution (Embassy, etc.) |

```toml
# Default (completion + history)
nut-shell = "0.1"

# With authentication
nut-shell = { version = "0.1", features = ["authentication"] }

# Minimal (no optional features)
nut-shell = { version = "0.1", default-features = false }

# Async runtime support
nut-shell = { version = "0.1", features = ["async"] }
```

**See [size-analysis/README.md](../size-analysis/README.md) for detailed flash/RAM cost breakdown.**
