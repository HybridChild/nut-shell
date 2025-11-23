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
    Shell, CharIo, CommandHandlers, Response, CliError,
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

impl CommandHandlers<DefaultConfig> for MyHandlers {
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

### How to Choose Buffer Sizes

Buffer sizes are configured via the `ShellConfig` trait (see [Configuration Examples](#configuration-examples)). Getting them right balances RAM usage with functionality.

#### Input Buffer (MAX_INPUT)

The input buffer holds the command being typed.

**Formula:**
```
MAX_INPUT = longest_path + spaces + longest_args + safety_margin
```

**Examples:**

| Command Example | Calculation | Recommended Size |
|----------------|-------------|------------------|
| `reboot` | 6 bytes + 20% = 8 bytes | 64 bytes |
| `system/network/status` | 22 bytes + 20% = 27 bytes | 64 bytes |
| `/system/network/wifi/configure SSID password 192.168.1.100 255.255.255.0` | 80 bytes + 20% = 96 bytes | 128 bytes |
| Long multi-arg commands | 200+ bytes | 256 bytes |

**Recommendations:**

| Use Case | Size | RAM Cost | Good For |
|----------|------|----------|----------|
| Simple CLI | 64 bytes | 64 bytes | Basic commands, short paths |
| **Standard (default)** | **128 bytes** | **128 bytes** | Most embedded CLIs |
| Complex commands | 256 bytes | 256 bytes | Long paths, many arguments |

**What happens when full:** Characters beyond capacity are silently ignored. Backspace still works to make room.

#### Output Buffer (CharIo - Async Only)

For async platforms (Embassy, RTIC), your CharIo implementation needs an output buffer. Bare-metal can flush immediately (no buffer needed).

**Formula:**
```
Find your longest response + 20% safety margin
```

**Common response sizes:**

| Response Type | Example | Typical Size |
|--------------|---------|--------------|
| Status | `"OK"` | 4 bytes |
| Prompt | `"admin@/system> "` | 20 bytes |
| Error | `"Invalid path"` | 20 bytes |
| Directory listing | 5 items × ~40 bytes | 200 bytes |
| Help text | Multiple lines | 400 bytes |

**Recommendations:**

| Use Case | Size | Good For |
|----------|------|----------|
| Minimal | 64 bytes | Simple status responses only |
| **Standard (recommended)** | **256 bytes** | Directory listings, typical commands |
| Verbose | 512 bytes | Long help text, detailed diagnostics |
| Maximum safety | 1024 bytes | Any single response guaranteed |

**What happens when full:** `put_char()` returns error, response is truncated.

#### Path Depth (MAX_PATH_DEPTH)

Maximum directory nesting depth.

**How to determine:**
```
Count levels in your deepest path:
/system/network/wifi/security/wpa2/enterprise/config
 1      2       3    4        5    6          7       = 7 levels
```

**Recommendations:**

| Tree Complexity | Size | RAM Cost | Good For |
|----------------|------|----------|----------|
| Flat | 4 | 16 bytes | 2-3 levels max |
| **Standard (default)** | **8** | **32 bytes** | Most CLIs |
| Deep | 12 | 48 bytes | Complex hierarchies |

**What happens when exceeded:** Returns `PathTooDeep` error, current directory unchanged.

#### Command History (HISTORY_SIZE)

Number of commands to remember.

**Considerations:**
- Each entry: ~130 bytes RAM
- Interactive users benefit from 10-20 entries
- RAM-constrained systems: 4 entries or disable entirely

**Recommendations:**

| Use Case | Size | RAM Cost | Good For |
|----------|------|----------|----------|
| **RAM-constrained** | **4** | **~520 bytes** | Minimal history |
| Standard | 10 (default) | ~1.3 KB | Interactive debugging |
| Power users | 20 | ~2.6 KB | Frequent command reuse |
| Disabled | 0 (feature flag) | 0 bytes | Non-interactive use |

**To disable entirely:**
```bash
cargo build --no-default-features --features authentication,completion
```

#### Argument Count (MAX_ARGS)

Maximum arguments per command.

**How to determine:**
```
Find your command with the most arguments:
led_set R G B brightness mode duration = 6 arguments
Add safety margin: 6 × 2 = 12
```

**Recommendations:**

| Use Case | Size | RAM Cost | Good For |
|----------|------|----------|----------|
| Simple | 8 | Stack only | 1-4 argument commands |
| **Standard (default)** | **16** | **Stack only** | Most CLIs |
| Complex | 32 | Stack only | Many-argument commands |

**Note:** Arguments are stack-allocated during parsing only. No persistent RAM cost.

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
- `authentication` - User login and access control (default: enabled)
- `completion` - Tab completion for commands/paths (default: enabled)
- `history` - Command history with arrow keys (default: enabled)
- `async` - Async command execution support (default: disabled)

**Build examples:**
```bash
# All features (default)
cargo build

# Minimal (no optional features)
cargo build --no-default-features

# Secure only (authentication only)
cargo build --no-default-features --features authentication

# Interactive only (no security)
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

### Bare-Metal UART (RP2040)

```rust
use rp2040_hal as hal;
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;
use heapless::Deque;

// Global RX buffer (ISR-safe)
static RX_QUEUE: Mutex<RefCell<Deque<u8, 64>>> =
    Mutex::new(RefCell::new(Deque::new()));

#[interrupt]
fn UART0_IRQ() {
    let uart = unsafe { &*hal::pac::UART0::ptr() };

    while uart.uartfr.read().rxfe().bit_is_clear() {
        let byte = uart.uartdr.read().data().bits();
        cortex_m::interrupt::free(|cs| {
            RX_QUEUE.borrow(cs).borrow_mut().push_back(byte).ok();
        });
    }
}

struct UartIo {
    uart: hal::uart::UartPeripheral<hal::uart::Enabled, hal::pac::UART0, hal::gpio::bank0::Gpio0, hal::gpio::bank0::Gpio1>,
}

impl CharIo for UartIo {
    type Error = core::convert::Infallible;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        let byte = cortex_m::interrupt::free(|cs| {
            RX_QUEUE.borrow(cs).borrow_mut().pop_front()
        });
        Ok(byte.map(|b| b as char))
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.uart.write_full_blocking(&[c as u8]);
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    // Initialize hardware
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let core = hal::pac::CorePeripherals::take().unwrap();

    // Set up clocks, pins, UART...
    let uart = /* initialize UART */;

    // Create shell
    let io = UartIo { uart };
    let handlers = MyHandlers;
    let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&ROOT, handlers, io);

    shell.activate().ok();

    loop {
        if let Ok(Some(c)) = io.get_char() {
            shell.process_char(c).ok();
        }
        cortex_m::asm::wfi(); // Sleep until interrupt
    }
}
```

### Embassy USB-CDC

```rust
use embassy_executor::Spawner;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use heapless::Vec;

struct UsbCdcIo<'d> {
    class: CdcAcmClass<'d, Driver<'d>>,
    rx_buffer: Vec<u8, 64>,
    tx_buffer: Vec<u8, 256>,
}

impl<'d> UsbCdcIo<'d> {
    pub fn new(class: CdcAcmClass<'d, Driver<'d>>) -> Self {
        Self {
            class,
            rx_buffer: Vec::new(),
            tx_buffer: Vec::new(),
        }
    }

    pub async fn flush(&mut self) -> Result<(), embassy_usb::driver::EndpointError> {
        if !self.tx_buffer.is_empty() {
            self.class.write_packet(&self.tx_buffer).await?;
            self.tx_buffer.clear();
        }
        Ok(())
    }
}

impl CharIo for UsbCdcIo<'_> {
    type Error = embassy_usb::driver::EndpointError;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        if let Some(&byte) = self.rx_buffer.first() {
            self.rx_buffer.remove(0);
            Ok(Some(byte as char))
        } else {
            Ok(None)
        }
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Normalize line endings
        let bytes = match c {
            '\r' => b"\r\n",
            '\n' => return Ok(()),
            _ => &[c as u8],
        };

        for &b in bytes {
            self.tx_buffer.push(b).ok();
        }
        Ok(())
    }
}

#[embassy_executor::task]
async fn usb_read_task(class: &'static mut CdcAcmClass<'static, Driver<'static>>, io: &'static Mutex<UsbCdcIo<'static>>) {
    let mut buf = [0u8; 64];
    loop {
        let n = class.read_packet(&mut buf).await.ok();
        if let Some(n) = n {
            let mut io = io.lock().await;
            for &byte in &buf[..n] {
                io.rx_buffer.push(byte).ok();
            }
        }
    }
}

#[embassy_executor::task]
async fn shell_task(io: &'static Mutex<UsbCdcIo<'static>>) {
    let handlers = MyHandlers;
    let mut shell = Shell::new(&ROOT, handlers, /* need to solve ownership */);

    shell.activate().ok();

    loop {
        let c = {
            let mut io = io.lock().await;
            io.get_char().ok().flatten()
        };

        if let Some(c) = c {
            shell.process_char_async(c).await.ok();
            io.lock().await.flush().await.ok();
        } else {
            embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize USB, spawn tasks...
    spawner.spawn(usb_read_task(class, io)).unwrap();
    spawner.spawn(shell_task(io)).unwrap();
}
```

### Native (Testing/Development)

```rust
use std::io::{self, Read, Write};

struct StdioIo;

impl CharIo for StdioIo {
    type Error = io::Error;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        let mut buf = [0u8; 1];
        match io::stdin().read(&mut buf) {
            Ok(1) => Ok(Some(buf[0] as char)),
            Ok(0) => Ok(None),
            Err(e) => Err(e),
            _ => Ok(None),
        }
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        print!("{}", c);
        io::stdout().flush()?;
        Ok(())
    }
}

fn main() {
    let io = StdioIo;
    let handlers = MyHandlers;
    let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&ROOT, handlers, io);

    shell.activate().unwrap();

    loop {
        if let Ok(Some(c)) = io.get_char() {
            shell.process_char(c).ok();
        }
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
    name: "http-get",
    description: "Fetch URL via HTTP",
    access_level: MyAccessLevel::User,
    kind: CommandKind::Async,  // Mark as async
    min_args: 1,
    max_args: 1,
};

// Dispatch in handler
impl CommandHandlers<DefaultConfig> for MyHandlers {
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
use nut_shell::{DefaultConfig, CommandHandlers, Response};

struct MyHandlers<'a> {
    system_state: &'a SystemState,
    config: &'a Config,
}

impl<'a> CommandHandlers<DefaultConfig> for MyHandlers<'a> {
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

### Problem: Input Buffer Overflow

**Symptom:** Long commands get truncated

**Solution:** Create a custom config with larger MAX_INPUT
```rust
struct LargeInputConfig;

impl ShellConfig for LargeInputConfig {
    const MAX_INPUT: usize = 256;  // Increased from 128
    const MAX_PATH_DEPTH: usize = 8;
    const MAX_ARGS: usize = 16;
    const MAX_PROMPT: usize = 64;
    const MAX_RESPONSE: usize = 256;
    const HISTORY_SIZE: usize = 10;

    // ... (include all required message constants)
}

let mut shell: Shell<_, _, _, _, LargeInputConfig> = Shell::new(&TREE, handlers, io);
```

### Problem: Output Buffer Overflow (Async)

**Symptom:** `BufferFull` errors, truncated responses

**Solution:** Increase output buffer in CharIo implementation
```rust
struct MyIo {
    tx_buffer: heapless::Vec<u8, 512>,  // Increase from 256
}
```

### Problem: High RAM Usage

**Symptom:** Stack overflow, allocation failures

**Solutions:**
1. Use `MinimalConfig` instead of `DefaultConfig`
   ```rust
   let mut shell: Shell<_, _, _, _, MinimalConfig> = Shell::new(&TREE, handlers, io);
   ```
2. Create custom config with reduced buffer sizes
   ```rust
   struct TinyConfig;
   impl ShellConfig for TinyConfig {
       const MAX_INPUT: usize = 64;        // Reduced from 128
       const MAX_PATH_DEPTH: usize = 4;    // Reduced from 8
       const MAX_ARGS: usize = 8;          // Reduced from 16
       const MAX_PROMPT: usize = 32;       // Reduced from 64
       const MAX_RESPONSE: usize = 128;    // Reduced from 256
       const HISTORY_SIZE: usize = 4;      // Reduced from 10
       // ... (include all required message constants)
   }
   ```
3. Disable history entirely: `cargo build --no-default-features --features authentication,completion`

### Problem: PathTooDeep Errors

**Symptom:** Deep paths return errors

**Solution:** Create custom config with increased MAX_PATH_DEPTH
```rust
struct DeepPathConfig;

impl ShellConfig for DeepPathConfig {
    const MAX_INPUT: usize = 128;
    const MAX_PATH_DEPTH: usize = 12;  // Increased from 8
    const MAX_ARGS: usize = 16;
    const MAX_PROMPT: usize = 64;
    const MAX_RESPONSE: usize = 256;
    const HISTORY_SIZE: usize = 10;

    // ... (include all required message constants)
}

let mut shell: Shell<_, _, _, _, DeepPathConfig> = Shell::new(&TREE, handlers, io);
```

### Problem: Characters Dropped on UART

**Symptom:** Missing characters in input

**Solutions:**
1. Increase ISR buffer size: `Deque<u8, 128>` instead of 64
2. Process input more frequently (reduce loop delay)
3. Use hardware flow control (RTS/CTS)

### Problem: USB Disconnect Issues

**Symptom:** CLI stops responding after USB reconnect

**Solution:** Implement disconnect detection and CLI restart
```rust
fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
    if self.disconnected {
        return Err(Error::Disconnected);  // Force restart
    }
    // ... normal logic
}
```

### Problem: Async Commands Block CLI

**Symptom:** Can't type while async command runs

**Expected behavior:** CLI blocks on async commands by design. Commands should:
1. Implement their own timeouts
2. Return quickly and spawn background tasks if needed
3. Provide progress feedback via periodic responses

```rust
async fn long_operation_async<C: ShellConfig>(args: &[&str]) -> Result<Response<C>, CliError> {
    // Option 1: Timeout
    embassy_time::with_timeout(Duration::from_secs(10), work()).await?;

    // Option 2: Spawn background task
    spawner.spawn(background_task()).ok();
    Ok(Response::success("Operation started in background"))
}
```

### Problem: Characters Echoed Multiple Times

**Symptom:** Typing `a` shows `aaa`

**Cause:** Both terminal and CLI echoing

**Solution:** Disable local echo in terminal:
```bash
stty -echo -F /dev/ttyACM0
screen /dev/ttyACM0 115200,cs8,-parenb,-cstopb,-echo
```

### Problem: Backspace Not Working

**Symptom:** Backspace shows `^H` or `^?`

**Cause:** Terminal sending wrong control code

**Solution:** CLI handles both `0x08` (BS) and `0x7F` (DEL). Configure terminal:
```bash
stty erase ^H
```

### Problem: High CPU Usage (Async Platforms)

**Symptom:** Executor using excessive CPU

**Cause:** Polling instead of awaiting

**Solution:** Use `.await`, not polling loops:
```rust
// ✅ RIGHT
loop {
    let c = read_char().await;  // Task suspends - zero CPU
    shell.process_char(c)?;
}

// ❌ WRONG
loop {
    if let Some(c) = try_read_char() {  // Wastes CPU
        shell.process_char(c)?;
    }
}
```

---

## CharIo Implementation Patterns

### Bare-Metal UART

Basic implementation using `embedded-hal` traits:

```rust
use embedded_hal::serial::{Read, Write};

struct UartIo<UART> {
    uart: UART,
}

impl<UART> CharIo for UartIo<UART>
where
    UART: Read<u8> + Write<u8>,
{
    type Error = UART::Error;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        match self.uart.read() {
            Ok(byte) => Ok(Some(byte as char)),
            Err(nb::Error::WouldBlock) => Ok(None),
            Err(nb::Error::Other(e)) => Err(e),
        }
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        nb::block!(self.uart.write(c as u8))?;
        Ok(())
    }
}
```

**Usage:**
```rust
let uart = setup_uart(); // Platform-specific initialization
let io = UartIo { uart };
let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&ROOT, handlers, io);

loop {
    if let Ok(Some(c)) = io.get_char() {
        shell.process_char(c).ok();
    }
}
```

### Embassy USB CDC (Async)

Implementation with buffering for async USB:

```rust
struct EmbassyUsbIo<'d, D: embassy_usb::driver::Driver<'d>> {
    class: CdcAcmClass<'d, D>,
    output_buffer: heapless::Vec<u8, 256>,
}

impl<'d, D> CharIo for EmbassyUsbIo<'d, D> {
    type Error = core::convert::Infallible;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        Ok(None)  // Reading happens externally
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.output_buffer.push(c as u8).ok();
        Ok(())
    }
}

impl<'d, D> EmbassyUsbIo<'d, D> {
    pub async fn flush(&mut self) -> Result<(), D::EndpointError> {
        if !self.output_buffer.is_empty() {
            self.class.write_packet(&self.output_buffer).await?;
            self.output_buffer.clear();
        }
        Ok(())
    }
}
```

**Usage:**
```rust
#[embassy_executor::task]
async fn shell_task(usb: CdcAcmClass<'static, Driver<'static>>) {
    let mut io = EmbassyUsbIo { class: usb, output_buffer: Vec::new() };
    let mut shell: Shell<_, _, _, _, DefaultConfig> = Shell::new(&ROOT, handlers, io);

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

### Handling Shared State

Use static or captured references for command access to system state:

```rust
use core::sync::atomic::{AtomicBool, Ordering};

static LED_STATE: AtomicBool = AtomicBool::new(false);

fn led_toggle_fn<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    let new_state = !LED_STATE.load(Ordering::Relaxed);
    LED_STATE.store(new_state, Ordering::Relaxed);

    let msg = if new_state { "LED on" } else { "LED off" };
    Ok(Response::success(msg))
}
```

**With captured state (via handlers):**
```rust
struct MyHandlers<'a> {
    system_state: &'a SystemState,
}

impl<'a, C: ShellConfig> CommandHandlers<C> for MyHandlers<'a> {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError> {
        match name {
            "status" => {
                let info = self.system_state.get_status();
                Ok(Response::success(&info))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}
```

### Long-Running Commands (Async Platforms)

For operations that take significant time, use async commands with natural `.await`:

```rust
// Marked as async in metadata
const FLASH_WRITE: CommandMeta<Level> = CommandMeta {
    kind: CommandKind::Async,
    // ... other fields
};

// Handler with natural async/await
impl<C: ShellConfig> CommandHandlers<C> for MyHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError> {
        Err(CliError::CommandNotFound)
    }

    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError> {
        match name {
            "flash-write" => {
                for i in 0..100 {
                    write_flash_page(i).await;
                    embassy_time::Timer::after_millis(10).await;
                }
                Ok(Response::success("Flash write complete"))
            }
            _ => Err(CliError::CommandNotFound),
        }
    }
}
```

No manual task spawning or global state needed!

---

## Best Practices and Anti-Patterns

### ✅ Best Practices

**Use the buffering model for async:**
```rust
// Good - batch output, flush once
for byte in batch {
    shell.process_char(byte as char)?;
}
io.flush().await?;
```

**Use async commands for long operations:**
```rust
// Good - non-blocking
async fn execute_async<C: ShellConfig>(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError> {
    match name {
        "long-op" => {
            for _ in 0..1000 {
                do_work().await;
            }
            Ok(Response::success("Done"))
        }
        _ => Err(CliError::CommandNotFound),
    }
}
```

**Use heapless for collections:**
```rust
// Good - no_std compatible
fn command<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    let mut data: heapless::Vec<u8, 32> = heapless::Vec::new();
    data.push(42).ok();
    Ok(Response::success("Done"))
}
```

**Await properly in async code:**
```rust
// Good - task suspends, zero CPU usage
loop {
    let c = read_char().await;
    shell.process_char_async(c).await?;
}
```

### ❌ Anti-Patterns to Avoid

**Don't make CharIo async:**
```rust
// AVOID - adds complexity without benefit
trait CharIo {
    async fn put_char(&mut self, c: char) -> Result<()>;
}
```

**Why:** Use the buffering model instead - sync trait, async flush externally.

**Don't flush after every character:**
```rust
// AVOID - inefficient for async platforms
for byte in batch {
    shell.process_char(byte as char)?;
    io.flush().await?;  // Too many I/O operations!
}
```

**Why:** Batch output and flush once after processing entire batch.

**Don't block in sync commands:**
```rust
// AVOID - blocks entire CLI
fn bad_command<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    for _ in 0..1000000 { }  // Blocks everything!
    Ok(Response::success("Done"))
}
```

**Why:** Use async commands for long-running operations.

**Don't allocate on heap:**
```rust
// AVOID - no_std incompatible
fn bad_command<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    let data = vec![1, 2, 3];  // Won't compile!
    Ok(Response::success("Done"))
}
```

**Why:** Use `heapless` collections instead.

**Don't poll in async loops:**
```rust
// AVOID - wastes CPU
loop {
    if let Some(c) = try_read_char() {  // Busy loop!
        shell.process_char(c)?;
    }
}
```

**Why:** Use `.await` to suspend the task when no data available.

---
