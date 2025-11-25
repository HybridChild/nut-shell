# Native Platform Examples

Examples demonstrating **nut-shell** CLI framework on native platforms (Linux, macOS, Windows).

- **[basic](#basic)** - Complete interactive command-line interface demonstrating all features including authentication, command navigation, history, and tab completion.
- **[async](#async)** - Async command execution using Tokio runtime, demonstrating how to integrate async operations into your CLI.

## Building and Running

### Prerequisites

- Rust toolchain (stable or nightly)
- Standard platform terminal/console

### Build

From the `examples/native` directory:

```bash
# Build basic example
cargo build --release --bin basic

# Build async example
cargo build --release --bin async --features async
```

### Run

```bash
# Run basic example
cargo run --release --bin basic

# Run async example
cargo run --release --bin async --features async
```

## Examples

### basic

A complete interactive command-line interface demonstrating nut-shell on native platforms.

**Features:**
- User authentication with password hashing (SHA-256)
- Hierarchical command tree navigation (`cd`, `..`)
- Command execution with access control
- Command history (up/down arrows)
- Tab completion for commands and directories
- Global commands (`?`, `ls`, `clear`, `logout`)
- Interactive prompt showing current user and path
- Terminal color support and ANSI escape sequences

**Commands Available:**

```
/
├── system/
│   ├── reboot   - Reboot the system (Admin)
│   ├── status   - Show system status (User)
│   └── version  - Show version information (Guest)
├── config/
│   ├── get <key>      - Get configuration value (User)
│   └── set <key> <val> - Set configuration value (Admin)
├── echo [args...]     - Echo arguments back (Guest)
└── uptime             - Show system uptime (Guest)

Global:
  ?      - Show help
  ls     - List directory contents
  clear  - Clear screen
  logout - End session (returns to login)
```

**Authentication:**

When built with authentication feature (default):
- **admin:admin123** (Admin access - full control)
- **user:user123** (User access - limited commands)
- **guest:guest123** (Guest access - read-only)

Login format: `username:password` (no spaces)

**Interactive Features:**

- **History Navigation**: Use ↑ (up arrow) and ↓ (down arrow) to navigate through command history
- **Tab Completion**: Press Tab to autocomplete commands and directory names
- **Double-ESC**: Press ESC twice to clear the input buffer
- **Ctrl+C**: Exit the shell

**What you'll learn:**
- How to implement CharIo trait for standard I/O
- How to define hierarchical command trees
- How to integrate authentication with password hashing
- How to use command handlers with sync/async support
- How to build feature-rich CLIs with nut-shell
- **How to structure the main loop to match embedded usage patterns**

**Embedded Target Usage Pattern:**

This example is structured to resemble how you would use nut-shell on an embedded target:

```rust
// EMBEDDED TARGET PATTERN:
loop {
    // Poll UART RX buffer for incoming characters
    if let Some(c) = uart_rx_buffer.pop() {
        // Feed character to shell (shell controls echoing)
        shell.process_char(c)?;
    }
}

// NATIVE EXAMPLE PATTERN (matches embedded):
loop {
    // Poll stdin for incoming characters (with raw mode enabled)
    let mut buf = [0u8; 1];
    if stdin.read(&mut buf).is_ok() {
        let c = buf[0] as char;
        // Feed character to shell (shell controls echoing)
        shell.process_char(c)?;
    }
}
```

**Key similarities:**
- Terminal/UART doesn't echo characters (application controls echo)
- Characters processed one at a time as they arrive
- Shell has full control over what appears on screen (enables password masking)
- Special keys (Tab, arrows) passed directly to shell for processing

**Note on Ctrl+C:** In raw mode, Ctrl+C becomes character `0x03` instead of sending a signal. The example detects this and exits gracefully. On an embedded target, you might handle this differently (e.g., ignore it, use it as a command, or implement a different exit mechanism like a dedicated reset button).

**Run:**

```bash
cargo run --release --bin basic
```

**Expected Output:**

```
nut-shell Basic Example
=======================

Authentication enabled. Available credentials:
  admin:admin123  (Admin access)
  user:user123    (User access)
  guest:guest123  (Guest access)

Type '?' for help, 'logout' to exit (with auth), or Ctrl+C to quit.

Login (username:password): admin:admin123
admin@/> ?
  ?         - List global commands
  ls        - Detail items in current directory
  logout    - Exit current session
  clear     - Clear screen
  ESC ESC   - Clear input buffer

admin@/> ls
Directories:
  system/        (Guest)      System commands
  config/        (User)       Configuration commands

Commands:
  echo           (Guest)      Echo arguments back
  uptime         (Guest)      Show system uptime (simulated)

admin@/> cd system
admin@/system> ls
Commands:
  reboot         (Admin)      Reboot the system (simulated)
  status         (User)       Show system status
  version        (Guest)      Show version information

admin@/system> status
System Status:
  CPU Usage: 23%
  Memory: 45% used
  Uptime: 42 hours

admin@/system> ..
admin@/> uptime
System uptime: 42 hours, 13 minutes

admin@/>
```

### async

An async example demonstrating how to use nut-shell with asynchronous command execution using the Tokio runtime.

**Features:**
- Tokio async runtime integration
- Async command execution with `process_char_async()`
- Mix of sync and async commands in the same shell
- Async delays, simulated HTTP fetches, and computations
- Full authentication, history, and completion support

**Commands Available:**

```
/
├── system/
│   ├── reboot   - Reboot the system (Admin)
│   └── info     - Show system information (Guest)
├── async/
│   ├── delay <seconds>  - Async delay for N seconds (Guest)
│   ├── fetch <url>      - Simulate async HTTP fetch (User)
│   └── compute          - Simulate async computation (User)
└── echo [args...]       - Echo arguments back (Guest)

Global:
  ?      - Show help
  ls     - List directory contents
  clear  - Clear screen
  logout - End session (returns to login)
```

**Authentication:**

When built with authentication feature (default):
- **admin:admin123** (Admin access)
- **user:user123** (User access)
- **guest:guest123** (Guest access)

**What you'll learn:**
- How to use `process_char_async()` for async command execution
- How to mix sync and async commands in the same shell
- How to implement async commands with `CommandKind::Async`
- How to integrate Tokio runtime with nut-shell
- Async command patterns for I/O operations and delays

**Async Command Implementation:**

Commands are marked as `CommandKind::Async` and implemented in the `execute_async()` trait method:

```rust
const CMD_DELAY: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "async_delay",
    name: "delay",
    description: "Async delay for N seconds",
    kind: CommandKind::Async,  // Mark as async
    min_args: 1,
    max_args: 1,
    // ... other fields
};

impl CommandHandler<DefaultConfig> for AsyncHandlers {
    async fn execute_async(&self, id: &str, args: &[&str])
        -> Result<Response<DefaultConfig>, CliError>
    {
        match id {
            "async_delay" => {
                let seconds = args[0].parse::<u64>()?;
                sleep(Duration::from_secs(seconds)).await;
                Ok(Response::success("Delay completed"))
            }
            // ... other async commands
        }
    }
}
```

**Run:**

```bash
cargo run --release --bin async --features async,authentication,completion,history
```

**Expected Output:**

```
nut-shell Async Example
=======================
This example demonstrates async command execution using Tokio.

Authentication enabled. Available credentials:
  admin:admin123  (Admin access)
  user:user123    (User access)
  guest:guest123  (Guest access)

Try these async commands in the 'async' directory:
  cd async
  delay 3       - Async delay for 3 seconds
  fetch http://example.com - Simulated async HTTP fetch
  compute       - Simulated async computation

Type '?' for help, 'logout' to exit (with auth), or Ctrl+C to quit.

Login (username:password): user:user123
user@/> cd async
user@/async> delay 3
Delayed for 3 second(s)
user@/async> fetch http://example.com
Fetching 'http://example.com'...
Response: 200 OK
Content-Length: 1234
Fetch completed successfully!
user@/async>
```

## Building with Different Feature Combinations

The native examples support flexible feature configurations:

### All Features (Default)

```bash
# Basic example
cargo run --release --bin basic

# Async example
cargo run --release --bin async --features async
```

Basic features enabled: authentication, completion, history
Async features enabled: async, authentication, completion, history

### Without Authentication

```bash
# Edit Cargo.toml and change nut-shell dependency:
nut-shell = { path = "../..", features = ["completion", "history"] }

# Or run from repository root:
cargo run --example basic --features completion,history
```

When authentication is disabled, the CLI starts directly at the prompt without requiring login.

### Minimal Configuration (No Optional Features)

```bash
# Edit Cargo.toml:
nut-shell = { path = "../.." }  # No features

# Or run from repository root:
cargo run --example basic --no-default-features
```

Minimal configuration removes authentication, history, and tab completion for the smallest footprint.

### Custom Feature Combinations

```bash
# Authentication only (no history or completion):
nut-shell = { path = "../..", features = ["authentication"] }

# History and completion only (no authentication):
nut-shell = { path = "../..", features = ["completion", "history"] }
```

## Platform-Specific Notes

### Terminal Support

The examples automatically configure the terminal in raw mode to resemble embedded target behavior:
- **Linux/macOS**: Full ANSI escape sequence support, raw terminal mode
- **Windows**: Full support via Windows Terminal or Windows 10+ console (ANSI/VT100 mode)

### Raw Mode Configuration

The example uses `crossterm` to configure the terminal in raw mode, which:
- **Disables local echo** - Shell controls all character echoing (enables password masking)
- **Disables line buffering** - Characters processed immediately as typed
- **Disables special key processing** - Tab, arrow keys, etc. passed to shell for completion/history

This configuration **matches embedded target behavior** where UART hardware doesn't echo characters and the application has full control.

### Terminal Emulators

Tested and working on:
- **Linux**: GNOME Terminal, Konsole, xterm, tmux, screen
- **macOS**: Terminal.app, iTerm2
- **Windows**: Windows Terminal (recommended), PowerShell, CMD

## Troubleshooting

### Password characters visible during login

If you see password characters echoing during login, ensure:
- You're using a terminal that supports raw mode (most modern terminals do)
- The `crossterm` crate is properly installed (should be automatic)
- You're not redirecting stdin from a file or pipe

The example automatically enables raw mode to disable echo. If issues persist, try a different terminal emulator.

### Tab completion or history not working

Ensure you built with the appropriate features enabled:

```bash
cargo build --release --features completion,history
```

Also verify your terminal supports ANSI escape sequences for arrow keys (most modern terminals do).

### Colors not displaying (Windows)

On Windows, ensure:
- Using Windows Terminal or a terminal with ANSI support
- Running on Windows 10 or later
- VT100 mode is enabled

## License

Same as parent nut-shell project (MIT OR Apache-2.0).
