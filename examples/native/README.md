# Native Platform Examples

Examples demonstrating **nut-shell** CLI framework on native platforms (Linux, macOS, Windows).

- **[basic](#basic)** - Complete interactive command-line interface demonstrating all features including authentication, command navigation, history, and tab completion.

## Building and Running

### Prerequisites

- Rust toolchain (stable or nightly)
- Standard platform terminal/console

### Build

From the `examples/native` directory:

```bash
cargo build --release --bin basic
```

### Run

```bash
cargo run --release --bin basic
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

## Building with Different Feature Combinations

The native examples support flexible feature configurations:

### All Features (Default)

```bash
cargo run --release --bin basic
```

Features enabled: authentication, completion, history

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
