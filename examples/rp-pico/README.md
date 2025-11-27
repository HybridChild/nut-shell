# Raspberry Pi Pico Examples

Examples for Raspberry Pi Pico (RP2040) demonstrating **nut-shell** CLI framework on embedded hardware.

- **[basic](#basic)** - Complete interactive command-line interface over USB CDC with optional authentication, command navigation, and embedded-optimized features (synchronous/bare-metal).
- **[embassy](#embassy)** - Embassy async runtime example with async command execution, buffered USB I/O, and optional authentication (demonstrates async feature).

## Hardware Setup

### USB Connection

Both examples use **USB CDC (Communications Device Class)** for serial communication - no external UART adapter needed!

- **Connection**: Simply connect the Pico's USB port directly to your computer
- **Serial Port**: Appears as `/dev/tty.usbmodemnut_shell1` (macOS/Linux) or `COMx` (Windows)
- **Baud Rate**: Not applicable (USB Full Speed - 12 Mbps)

## Building and Flashing

### Prerequisites

- Rust toolchain with `thumbv6m-none-eabi` target
- probe-rs (recommended) or elf2uf2-rs for flashing
- Serial terminal (screen, minicom, PuTTY)
- **No USB-to-Serial adapter needed** - uses built-in USB!

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

After flashing, the Pico will enumerate as a USB serial device. Connect using:

```bash
# Linux
screen /dev/tty.usbmodemnut_shell1 115200

# macOS
screen /dev/tty.usbmodemnut_shell1 115200

# Windows (PowerShell)
# Use PuTTY or another terminal emulator - look for "USB Serial Device (COMx)"
```

**Exit screen**: Press `Ctrl+A` then `K` then `Y`

## Examples

### basic

A complete interactive command-line interface demonstrating nut-shell on embedded hardware over USB CDC.

**Features:**
- USB CDC serial communication (no external adapter needed)
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
│   ├── info       - Show device information (User)
│   ├── uptime     - Show system uptime (User)
│   ├── meminfo    - Display memory information (User)
│   ├── benchmark  - Run performance benchmark (User)
│   ├── flash      - Display flash memory info (User)
│   └── crash      - Trigger panic for testing (Admin)
└── hardware/
    ├── get/
    │   ├── temp       - Read temperature sensor (User)
    │   ├── chipid     - Show flash unique ID (User)
    │   ├── clocks     - Display clock frequencies (User)
    │   ├── core       - Show CPU core ID (User)
    │   ├── bootreason - Show last reset reason (User)
    │   └── gpio <pin> - Show GPIO pin status (User)
    └── set/
        └── led <on|off> - Control onboard LED (User)

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
- How to implement CharIo trait for USB CDC
- How to define command trees for embedded systems
- How to integrate authentication in resource-constrained environments
- How to build interactive CLIs without heap allocation
- USB device configuration on RP2040

**Memory Usage:**
- Flash: ~18KB (with all features)
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
  hardware/      (User)       Hardware control

admin@/> cd hardware/get
admin@/hardware/get> temp
Temperature: 24.5°C

admin@/hardware/get> cd ../set
admin@/hardware/set> led on
LED turned on

admin@/hardware/set>
```

### embassy

An Embassy-based async runtime example demonstrating nut-shell with async command execution over USB CDC.

**Features:**
- Embassy async executor for concurrent task execution
- USB CDC serial communication (no external adapter needed)
- Async USB I/O with buffered output (deferred flush pattern)
- Async command execution using `process_char_async()`
- LED control via async channel communication
- Background temperature monitoring task
- Async delay command demonstration
- Optional user authentication with password hashing (SHA-256)
- Minimal memory footprint with static allocation

**Commands Available:**

```
/
├── system/
│   ├── info       - Show device information (User)
│   ├── uptime     - Show system uptime (User)
│   ├── meminfo    - Display memory information (User)
│   ├── benchmark  - Run performance benchmark (User)
│   ├── flash      - Display flash memory info (User)
│   ├── crash      - Trigger panic for testing (Admin)
│   └── delay <seconds> - Async delay demo (User) [ASYNC]
└── hardware/
    ├── get/
    │   ├── temp       - Read temperature sensor (User)
    │   ├── chipid     - Show flash unique ID (User)
    │   ├── clocks     - Display clock frequencies (User)
    │   ├── core       - Show CPU core ID (User)
    │   ├── bootreason - Show last reset reason (User)
    │   └── gpio <pin> - Show GPIO pin status (User)
    └── set/
        └── led <on|off> - Control onboard LED (User)

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
- USB CDC device implementation with embassy-usb
- Async I/O buffering patterns (deferred flush)
- Handling large output with packet chunking
- Async command execution with `process_char_async()`
- Concurrent task spawning with Embassy executor
- Inter-task communication using Embassy channels
- RefCell pattern for shared buffer access

**Memory Usage:**
- Flash: ~30KB (with async + authentication)
- RAM: ~6KB (including executor stack)
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

- **USB CDC Implementation**: Uses embassy-usb for full-speed USB device with CDC-ACM class
- **Packet Chunking**: Automatically splits output larger than 64 bytes into multiple USB packets
- **Deferred Flush Pattern**: Output is buffered to memory during `process_char_async()`, then flushed to USB after processing completes (see `IO_DESIGN.md`)
- **RefCell for Interior Mutability**: Shared buffer accessed through RefCell to enable both Shell and flush logic to access the same buffer
- **Dual I/O References**: Two `BufferedCharIo` instances reference the same buffer - one owned by Shell, one for flushing

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

### USB serial port not appearing

- Check USB cable connection (ensure it's a data cable, not charge-only)
- Try a different USB port on your computer
- Wait a few seconds after flashing for USB enumeration
- Check if device appears in system:
  - **macOS**: `ls /dev/tty.usb*`
  - **Linux**: `ls /dev/ttyACM*` or `dmesg | tail`
  - **Windows**: Device Manager → Ports (COM & LPT)

### Embassy example not enumerating (no USB device)

This was a bug that has been fixed. Ensure you have the latest code with:
- Correct USB descriptor buffers (config, BOS, control)
- Proper task spawning order (USB task first)
- Output packet chunking for data >64 bytes

### "Permission denied" on serial port (Linux)

```bash
sudo usermod -a -G dialout $USER
# Log out and back in for changes to take effect
```

### Commands execute but no output visible

This was a bug in the embassy example where output >64 bytes was truncated. Fixed by chunking output into multiple USB packets. Ensure you have the latest code.

### Build errors about missing target

```bash
rustup target add thumbv6m-none-eabi
```

## Hardware Verification

These examples have been tested and verified working on physical Raspberry Pi Pico hardware:
- ✅ Basic example: USB CDC communication working
- ✅ Embassy example: USB CDC with async working
- ✅ All commands functional
- ✅ Authentication working
- ✅ Tab completion working
- ✅ Command history working

## License

Same as parent nut-shell project (MIT OR Apache-2.0).
