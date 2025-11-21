# Raspberry Pi Pico Examples

Examples for Raspberry Pi Pico (RP2040) demonstrating **nut-shell** CLI framework on embedded hardware.

- **[uart_cli](#uart_cli)** - Complete interactive command-line interface over UART with authentication, command navigation, and embedded-optimized features.

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

### uart_cli

A complete interactive command-line interface demonstrating nut-shell on embedded hardware.

**Features:**
- UART communication at 115200 baud
- User authentication with password hashing (SHA-256)
- Hierarchical command tree navigation (`cd`, `..`)
- Command execution with access control
- Global commands (`?`, `ls`, `clear`, `logout`)
- Interactive prompt showing current user and path
- Minimal memory footprint (no heap allocation)

**Commands Available:**

```
/
├── system/
│   ├── info    - Show device information (User)
│   └── reboot  - Reboot the device (Admin)
└── led <on|off> - Toggle LED (User)

Global:
  ?      - Show help
  ls     - List directory contents
  clear  - Clear screen
  logout - End session (returns to login)
```

**Authentication:**

When built with authentication feature (default):
- **admin:pico123** (Admin access - full control)
- **user:pico456** (User access - limited commands)

Login format: `username:password` (no spaces)

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
cargo run --release --bin uart_cli
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
  reboot         (Admin)      Reboot the device

admin@/system> info
Device: Raspberry Pi Pico
Chip: RP2040
Firmware: nut-shell v0.1.0
UART: GP0(TX)/GP1(RX) @ 115200

admin@/system>
```

## Building Without Authentication

To build without the authentication feature (open access, no login):

```bash
# Edit Cargo.toml and remove "authentication" from nut-shell features:
nut-shell = { path = "../..", default-features = false }  # No features

# Then build:
cargo build --release --bin uart_cli
```

When authentication is disabled, the CLI starts directly at the prompt without requiring login.

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
