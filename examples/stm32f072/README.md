# STM32 NUCLEO-F072RB Examples

Examples for STM32 NUCLEO-F072RB demonstrating **nut-shell** CLI framework on embedded hardware.

- **[basic](#basic)** - Complete interactive command-line interface over UART with optional authentication, command navigation, and embedded-optimized features (synchronous/bare-metal).

## Hardware Setup

### Board Overview

**NUCLEO-F072RB** development board:
- **MCU**: STM32F072RBT6 (ARM Cortex-M0, 128KB Flash, 16KB RAM)
- **Debug**: Integrated ST-LINK/V2-1 debugger/programmer
- **Virtual COM Port**: USART2 routed through ST-LINK USB

### UART Connections

The examples use USART2 for serial communication, which is connected to the ST-LINK virtual COM port:

- **UART**: USART2
- **TX**: PA2 (connected to ST-LINK VCP)
- **RX**: PA3 (connected to ST-LINK VCP)
- **Baud Rate**: 115200
- **LED**: PA5 (User LED LD2, green)

### Connection Diagram

```
NUCLEO-F072RB Board
┌──────────────────────────────┐
│                              │
│  ST-LINK USB ────┐           │
│  (Virtual COM)   │           │
│         ↓        │           │
│    USART2 (PA2/PA3)          │
│         ↓        │           │
│    Shell CLI     │           │
│         ↓        │           │
│    LED (PA5) ────┘           │
│                              │
└──────────────────────────────┘
```

**Note**: Just connect the NUCLEO board via USB - no external serial adapter needed! The ST-LINK provides both programming and serial communication.

## Building and Flashing

### Prerequisites

- Rust toolchain with `thumbv6m-none-eabi` target
- probe-rs (recommended), OpenOCD, or STM32CubeProgrammer for flashing
- Serial terminal (screen, minicom, PuTTY)

### Install Target

```bash
rustup target add thumbv6m-none-eabi
```

### Install Tools

```bash
# probe-rs (recommended - supports debugging)
cargo install probe-rs-tools

# OR OpenOCD (widely supported)
# macOS: brew install openocd
# Linux: apt-get install openocd gdb-multiarch

# OR STM32CubeProgrammer from ST website
```

### Build

From the `examples/stm32f072` directory:

```bash
cargo build --release --bin basic
```

### Flash

**Method 1: probe-rs (recommended)**

```bash
cargo run --release --bin basic
```

Or explicitly specify the chip:

```bash
probe-rs run --chip STM32F072RBTx --release --bin basic
```

**Method 2: OpenOCD + GDB**

Terminal 1 (start OpenOCD server):
```bash
openocd -f interface/stlink.cfg -f target/stm32f0x.cfg
```

Terminal 2 (flash with GDB):
```bash
arm-none-eabi-gdb target/thumbv6m-none-eabi/release/basic
# In GDB prompt:
(gdb) target extended-remote :3333
(gdb) load
(gdb) monitor reset halt
(gdb) continue
```

**Method 3: STM32CubeProgrammer**

Convert ELF to HEX, then use STM32CubeProgrammer GUI:
```bash
arm-none-eabi-objcopy -O ihex target/thumbv6m-none-eabi/release/basic basic.hex
# Use STM32CubeProgrammer to flash basic.hex
```

### Connect to Serial

After flashing, connect to the ST-LINK virtual COM port:

```bash
# Linux
screen /dev/ttyACM0 115200

# macOS (find the device first)
ls /dev/tty.usbmodem*
screen /dev/tty.usbmodem* 115200

# Windows
# Use PuTTY or Tera Term, select the STMicroelectronics Virtual COM Port at 115200 baud
```

**Exit screen**: Press `Ctrl+A` then `K` then `Y`

## Examples

### basic

A complete interactive command-line interface demonstrating nut-shell on STM32 hardware.

**Features:**
- UART communication at 115200 baud via ST-LINK VCP
- Optional user authentication with password hashing (SHA-256)
- Hierarchical command tree navigation (`cd`, `..`)
- Optional command execution with access control
- Global commands (`?`, `ls`, `clear`, `logout`)
- Interactive prompt showing current directory (and user when authenticated)
- Minimal memory footprint (no heap allocation)

**Commands Available:**

```
/
└── system/
    └── info    - Show device information (User)

Global:
  ?      - Show help
  ls     - List directory contents
  clear  - Clear screen
  logout - End session (returns to login)
```

**Authentication:**

When built with the `authentication` feature enabled:
- **admin:stm32admin** (Admin access - full control)
- **user:stm32user** (User access - limited commands)

Login format: `username:password` (no spaces)

Without authentication, the CLI starts directly at the prompt.

**What you'll learn:**
- How to implement CharIo trait for STM32 UART
- How to define command trees for embedded systems
- How to integrate authentication in resource-constrained environments
- How to build interactive CLIs without heap allocation
- UART configuration and initialization on STM32F0
- Working with STM32 HAL in a no_std environment

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

Login (username:password): admin:stm32admin
admin@/> ?
  ?         - List global commands
  ls        - Detail items in current directory
  logout    - Exit current session
  clear     - Clear screen
  ESC ESC   - Clear input buffer

admin@/> ls
Directories:
  system/        (User)       System commands

admin@/> cd system
admin@/system> ls
Commands:
  info           (User)       Show device information

admin@/system> info
Device: STM32 NUCLEO-F072RB
Chip: STM32F072RBT6
Core: ARM Cortex-M0
Firmware: nut-shell v0.1.0 - UART CLI Example
UART: USART2 (PA2/PA3) @ 115200

admin@/system>
```

## Building with Different Feature Combinations

The examples support various feature combinations to customize functionality. The target (`thumbv6m-none-eabi`) is automatically configured in `.cargo/config.toml`.

### Available Features

- **`authentication`** - User authentication with password hashing (SHA-256)
- **`completion`** - Tab completion for commands and directories
- **`history`** - Command history with arrow key navigation

### Default Configuration

By default, examples build with `completion` and `history` enabled, but **without** authentication:

```bash
# Basic example (default features)
cargo build --release --bin basic
```

### With Authentication

To enable user authentication and login:

```bash
# Basic example with authentication
cargo build --release --bin basic --features authentication

# With all features enabled
cargo build --release --bin basic --features authentication,completion,history
```

**Default credentials:**
- `admin:stm32admin` (Admin access)
- `user:stm32user` (User access)

### Without Any Optional Features

Minimal build with only core functionality:

```bash
# Basic example (no optional features)
cargo build --release --bin basic --no-default-features
```

### Custom Feature Combinations

Mix and match features as needed:

```bash
# Authentication + history, no completion
cargo build --release --bin basic --no-default-features --features authentication,history

# Completion only
cargo build --release --bin basic --no-default-features --features completion

# History only
cargo build --release --bin basic --no-default-features --features history
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

- Ensure NUCLEO board is connected via USB
- Try `probe-rs list` to verify ST-LINK detection
- Update ST-LINK firmware using STM32CubeProgrammer if needed

### Serial port not appearing

- Check USB connection (use the ST-LINK USB port, not the USB user port)
- Try a different USB cable or port
- On Linux: Check device appears with `ls /dev/ttyACM*`
- On macOS: Check device appears with `ls /dev/tty.usbmodem*`
- On Windows: Check Device Manager for "STMicroelectronics Virtual COM Port"

### "Permission denied" on serial port (Linux)

```bash
sudo usermod -a -G dialout $USER
# Log out and back in for changes to take effect
```

### Garbled output on serial terminal

- Verify baud rate is set to 115200
- Check 8N1 configuration (8 data bits, no parity, 1 stop bit)
- Try resetting the board (black reset button)

### Build errors about missing target

```bash
rustup target add thumbv6m-none-eabi
```

### Flash fails with OpenOCD

- Ensure no other debugger is connected
- Try pressing the reset button on the board
- Check ST-LINK firmware is up to date

## Hardware Verification

This example has been compiled and verified to build correctly for the thumbv6m-none-eabi target. Hardware testing on physical NUCLEO-F072RB boards is pending.

If you test this example on hardware, please report your results!

## Development

The examples are organized as a cargo workspace with a shared library:

```
examples/stm32f072/
├── Cargo.toml              # Workspace manifest
├── .cargo/
│   └── config.toml         # Target configuration
├── src/                    # Shared library code
│   ├── lib.rs
│   └── access_level.rs
└── bin/
    └── basic/              # Basic example
        ├── main.rs
        ├── handlers.rs
        ├── hw_setup.rs
        ├── hw_state.rs
        ├── io.rs
        └── tree.rs
```

## License

Same as parent nut-shell project (MIT OR Apache-2.0).
