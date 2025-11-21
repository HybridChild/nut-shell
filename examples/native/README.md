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

The examples use standard terminal I/O and support:
- **Linux/macOS**: Full ANSI escape sequence support, raw terminal mode
- **Windows**: Basic support via CMD/PowerShell (ANSI support on Windows 10+)

### Terminal Configuration

For the best experience on Unix-like systems, ensure your terminal is in raw mode. The example uses blocking reads for simplicity. For production use, consider non-blocking I/O or async runtime integration.

### Terminal Emulators

Tested and working on:
- **Linux**: GNOME Terminal, Konsole, xterm, tmux, screen
- **macOS**: Terminal.app, iTerm2
- **Windows**: Windows Terminal (recommended), PowerShell, CMD

## Troubleshooting

### Input echoing or special characters visible

The example uses a simple I/O implementation. For a production CLI, you may want to:
- Disable terminal echo for password input
- Enable raw terminal mode for better control
- Handle UTF-8 input properly
- Implement proper ANSI escape sequence parsing

### Tab completion or history not working

Ensure you built with the appropriate features enabled:

```bash
cargo build --release --features completion,history
```

### Colors not displaying (Windows)

On Windows, ensure:
- Using Windows Terminal or a terminal with ANSI support
- Running on Windows 10 or later
- VT100 mode is enabled

## License

Same as parent nut-shell project (MIT OR Apache-2.0).
