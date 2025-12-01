# Native Platform Examples

Examples demonstrating **nut-shell** CLI framework on native platforms (Linux, macOS, Windows).

- **[basic](#basic)** - Complete interactive CLI demonstrating all features
- **[async](#async)** - Async command execution using Tokio runtime

## Examples

### basic

Complete interactive CLI demonstrating all nut-shell features on native platforms.

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

**Authentication** (when enabled):
- `admin:admin123` - Admin access
- `user:user123` - User access
- `guest:guest123` - Guest access

**Key implementation details:**
- `CharIo` trait implementation for stdin/stdout
- Raw terminal mode mimics embedded UART behavior (no local echo, character-at-a-time)
- Main loop polls stdin and feeds characters to shell
- Ctrl+C (0x03) used for graceful exit

**Run:** `cargo run --release --bin basic`

---

### async

Async command execution using Tokio runtime, demonstrating `process_char_async()` and mixing sync/async commands.

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

**Authentication** (when enabled):
- `admin:admin123` - Admin access
- `user:user123` - User access
- `guest:guest123` - Guest access

**Key implementation details:**
- Commands marked `CommandKind::Async` in metadata
- Async commands implement `execute_async()` trait method
- `Shell` uses `process_char_async()` instead of `process_char()`
- Tokio runtime required

**Run:** `cargo run --release --bin async --features async`

---

## Feature Configuration

Both examples support flexible feature combinations:

```bash
# With authentication
cargo run --release --bin basic --features authentication

# Minimal (no optional features)
cargo run --release --bin basic --no-default-features

# Custom combinations
cargo run --release --bin basic --features completion,history
```

Available features: `authentication`, `completion`, `history`, `async`

---

## License

Same as parent nut-shell project (MIT OR Apache-2.0).
