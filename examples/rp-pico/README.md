# Raspberry Pi Pico Examples

Examples for Raspberry Pi Pico (RP2040) demonstrating **nut-shell** CLI framework on embedded hardware.

- **[basic](#basic)** - Complete interactive command-line interface over UART with optional authentication, command navigation, and embedded-optimized features (synchronous/bare-metal).
- **[embassy](#embassy)** - Embassy async runtime example with async command execution, buffered UART I/O, and optional authentication (demonstrates async feature).

## Hardware Setup

### UART Connections

The examples use UART0 for serial communication:

- **TX (Output)**: GPIO0
- **RX (Input)**: GPIO1
- **Baud Rate**: 115200
- **Ground**: Connect GND pin

### Connection Diagram

```
Raspberry Pi Pico          USB-to-Serial Adapter
┌─────────────┐            ┌──────────────┐
│          GP0├────────────┤RX            │
│          GP1├────────────┤TX            │
│          GND├────────────┤GND           │
└─────────────┘            └──────────────┘
```

**Note**: Connect Pico GP0 (TX) to adapter RX, and Pico GP1 (RX) to adapter TX (crossover).

## Building and Flashing

### Prerequisites

- Rust toolchain with `thumbv6m-none-eabi` target
- probe-rs (recommended) or elf2uf2-rs for flashing
- USB-to-Serial adapter (e.g., FTDI, CP2102)
- Serial terminal (screen, minicom, PuTTY)

### Install Target

```bash
rustup target add thumbv6m-none-eabi
```

### Install Tools

```bash
# probe-rs (recommended - supports debugging)
cargo install probe-rs --features cli

# OR elf2uf2-rs (for UF2 bootloader flashing)
cargo install elf2uf2-rs
```

### Build

From the `examples/rp-pico` directory:

```bash
cargo build --release --bin <example_name>
```

### Flash

**Method 1: probe-rs (requires debug probe)**

```bash
cargo run --release --bin <example_name>
```

**Method 2: UF2 Bootloader (no debug probe needed)**

```bash
# Build and convert to UF2
cargo build --release --bin <example_name>
elf2uf2-rs target/thumbv6m-none-eabi/release/<example_name> <example_name>.uf2

# Flash: Hold BOOTSEL button while connecting USB
# Copy the .uf2 file to the RPI-RP2 drive that appears
cp <example_name>.uf2 /Volumes/RPI-RP2/  # macOS
cp <example_name>.uf2 /media/$USER/RPI-RP2/  # Linux
# Or drag-and-drop in File Explorer (Windows)
```

### Connect to Serial

After flashing, connect to the serial port:

```bash
# Linux
screen /dev/ttyUSB0 115200

# macOS
screen /dev/tty.usbserial-* 115200

# Windows (PowerShell)
# Use PuTTY or another terminal emulator
```

**Exit screen**: Press `Ctrl+A` then `K` then `Y`

## Examples

### basic

A complete interactive command-line interface demonstrating nut-shell on embedded hardware.

**Features:**
- UART communication at 115200 baud
- Optional user authentication with password hashing (SHA-256)
- Hierarchical command tree navigation (`cd`, `..`)
- Optional command execution with access control
- Global commands (`?`, `ls`, `clear`, `logout`)
- Interactive prompt showing current directory (and user when authenticated)
- Minimal memory footprint (no heap allocation)

**Commands Available:**

```
/
├── system/
│   └── info    - Show device information (User)
└── led <on|off> - Toggle LED (User)

Global:
  ?      - Show help
  ls     - List directory contents
  clear  - Clear screen
  logout - End session (returns to login)
```

**Authentication:**

When built with the `authentication` feature enabled:
- **admin:pico123** (Admin access - full control)
- **user:pico456** (User access - limited commands)

Login format: `username:password` (no spaces)

Without authentication, the CLI starts directly at the prompt.

**What you'll learn:**
- How to implement CharIo trait for UART
- How to define command trees for embedded systems
- How to integrate authentication in resource-constrained environments
- How to build interactive CLIs without heap allocation
- UART configuration and initialization on RP2040

**Memory Usage:**
- Flash: ~15KB (with all features)
- RAM: <2KB static allocation
- No heap allocation (pure `no_std`)

**Run:**

```bash
cargo run --release --bin basic
```

**Expected Output:**

```
nut-shell v0.1.0 - Embedded CLI Framework
Type '?' for help

Login (username:password): admin:pico123
admin@/> ?
  ?         - List global commands
  ls        - Detail items in current directory
  logout    - Exit current session
  clear     - Clear screen
  ESC ESC   - Clear input buffer

admin@/> ls
Directories:
  system/        (User)       System commands

Commands:
  led            (User)       Toggle onboard LED

admin@/> cd system
admin@/system> ls
Commands:
  info           (User)       Show device information

admin@/system> info
Device: Raspberry Pi Pico
Chip: RP2040
Firmware: nut-shell v0.1.0
UART: GP0(TX)/GP1(RX) @ 115200

admin@/system>
```

### embassy

An Embassy-based async runtime example demonstrating nut-shell with async command execution.

**Features:**
- Embassy async executor for concurrent task execution
- Async UART I/O with buffered output (deferred flush pattern)
- Async command execution using `process_char_async()`
- LED control via async channel communication
- Async delay command demonstration
- Optional user authentication with password hashing (SHA-256)
- Minimal memory footprint with static allocation

**Commands Available:**

```
/
├── system/
│   ├── info    - Show device information (User)
│   └── delay <seconds> - Async delay demo (User) [ASYNC]
└── led <on|off> - Toggle LED (User)

Global:
  ?      - Show help
  ls     - List directory contents
  clear  - Clear screen
  logout - End session (returns to login)
```

**Authentication:**

When built with the `authentication` feature enabled:
- **admin:pico123** (Admin access - full control)
- **user:pico456** (User access - limited commands)

Without authentication, the CLI starts directly at the prompt.

**What you'll learn:**
- How to integrate nut-shell with Embassy async runtime
- Async I/O buffering patterns (deferred flush)
- Async command execution with `process_char_async()`
- Concurrent task spawning with Embassy executor
- Inter-task communication using Embassy channels
- RefCell pattern for shared buffer access

**Memory Usage:**
- Flash: ~26KB (with async + authentication)
- RAM: ~5.7KB (including executor stack)
- No heap allocation (pure `no_std`)

**Build and Run:**

```bash
# Build with embassy feature
cargo build --release --bin embassy --features embassy

# Flash using probe-rs
cargo run --release --bin embassy --features embassy

# Or flash using UF2 bootloader
elf2uf2-rs target/thumbv6m-none-eabi/release/embassy embassy.uf2
cp embassy.uf2 /Volumes/RPI-RP2/
```

**Try the async delay command:**

```
Login (username:password): user:pico456
user@/> cd system
user@/system> delay 3
[... waits 3 seconds ...]
Delayed for 3 second(s)
user@/system>
```

**Architecture Highlights:**

- **Deferred Flush Pattern**: Output is buffered to memory during `process_char_async()`, then flushed to UART after processing completes (see `IO_DESIGN.md`)
- **RefCell for Interior Mutability**: Shared buffer accessed through RefCell to enable both Shell and flush logic to access the same buffer
- **Dual I/O References**: Two `BufferedUartCharIo` instances reference the same buffer - one owned by Shell, one for flushing

## Building with Different Feature Combinations

The examples support various feature combinations to customize functionality. The target (`thumbv6m-none-eabi`) is automatically configured in `.cargo/config.toml`.

### Available Features

- **`authentication`** - User authentication with password hashing (SHA-256)
- **`completion`** - Tab completion for commands and directories
- **`history`** - Command history with arrow key navigation
- **`embassy`** - Embassy async runtime (required for embassy example)

### Default Configuration

By default, examples build with `completion` and `history` enabled, but **without** authentication:

```bash
# Basic example (default features)
cargo build --release --bin basic

# Embassy example (default features + async)
cargo build --release --bin embassy --features embassy
```

### With Authentication

To enable user authentication and login:

```bash
# Basic example with authentication
cargo build --release --bin basic --features authentication

# Embassy example with authentication
cargo build --release --bin embassy --features embassy,authentication

# With all features enabled
cargo build --release --bin basic --features authentication,completion,history
```

**Default credentials:**
- `admin:pico123` (Admin access)
- `user:pico456` (User access)

### Without Any Optional Features

Minimal build with only core functionality:

```bash
# Basic example (no optional features)
cargo build --release --bin basic --no-default-features

# Embassy example (only async, no other features)
cargo build --release --bin embassy --no-default-features --features embassy
```

### Custom Feature Combinations

Mix and match features as needed:

```bash
# Authentication + history, no completion
cargo build --release --bin basic --no-default-features --features authentication,history

# Completion only
cargo build --release --bin basic --no-default-features --features completion

# Embassy with all features
cargo build --release --bin embassy --no-default-features --features embassy,authentication,completion,history
```

### Behavior Differences

**With authentication enabled:**
- Shows login prompt on startup
- Requires `username:password` to access commands
- Access control based on user level
- `logout` command available

**Without authentication:**
- Starts directly at command prompt
- No login required
- All commands accessible
- No `logout` command

## Troubleshooting

### "No device found" with probe-rs

- Ensure debug probe is connected and recognized
- Try `probe-rs list` to verify probe detection
- Use UF2 bootloader method if no debug probe available

### Serial port not appearing

- Check USB-to-Serial adapter connection
- Verify UART wiring (TX/RX crossover, GND connected)
- Try different USB port or cable
- Check adapter is recognized: `ls /dev/tty*` (Linux/macOS)

### "Permission denied" on serial port (Linux)

```bash
sudo usermod -a -G dialout $USER
# Log out and back in for changes to take effect
```

### Garbled output on serial terminal

- Verify baud rate is set to 115200
- Check 8N1 configuration (8 data bits, no parity, 1 stop bit)
- Ensure ground connection between Pico and serial adapter

### Build errors about missing target

```bash
rustup target add thumbv6m-none-eabi
```

## Hardware Verification

These examples have been compiled and verified to build correctly for the thumbv6m-none-eabi target. Hardware testing on physical Raspberry Pi Pico devices is pending.

If you test these examples on hardware, please report your results!

## License

Same as parent nut-shell project (MIT OR Apache-2.0).
