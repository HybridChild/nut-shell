# Raspberry Pi Pico Examples

Examples for Raspberry Pi Pico (RP2040) demonstrating **nut-shell** CLI framework on embedded hardware.

- **[basic](#basic)** - Complete interactive CLI over USB CDC (synchronous/bare-metal)
- **[embassy](#embassy)** - Embassy async runtime with async command execution

## Hardware Setup

**USB Connection:** Both examples use USB CDC (Communications Device Class) - no external UART adapter needed! Simply connect the Pico's USB port directly to your computer.

## Prerequisites

```bash
rustup target add thumbv6m-none-eabi
cargo install probe-rs --features cli  # or elf2uf2-rs for UF2 bootloader
```

## Examples

### basic

Complete interactive CLI over USB CDC demonstrating **nut-shell** on embedded hardware.

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

**Authentication** (when enabled):
- `admin:admin123` - Admin access
- `user:user123` - User access

**Memory Usage:**
- Flash: ~18KB (with all features)
- RAM: <2KB static allocation
- No heap allocation (pure `no_std`)

**Flash and connect:**

```bash
# Flash with probe-rs
cargo run --release --bin basic

# Or use UF2 bootloader (no debug probe needed)
cargo build --release --bin basic
elf2uf2-rs target/thumbv6m-none-eabi/release/basic basic.uf2
# Hold BOOTSEL button while connecting USB, then copy .uf2 file to RPI-RP2 drive

# Connect to serial
screen /dev/tty.usbmodemnut_shell1 115200  # macOS/Linux
```

---

### embassy

Embassy async runtime with async command execution, buffered USB I/O, and concurrent task spawning.

**Commands Available:**

Same as basic example, plus:
- `delay <seconds>` - Async delay demonstration (User) [ASYNC]

**Features:**
- Embassy async executor for concurrent tasks
- Async USB I/O with buffered output (deferred flush pattern)
- Async command execution using `process_char_async()`
- LED control via async channel communication
- Background temperature monitoring task

**Memory Usage:**
- Flash: ~30KB (with async + authentication)
- RAM: ~6KB (including executor stack)
- No heap allocation (pure `no_std`)

**Flash and connect:**

```bash
# Flash with embassy feature
cargo run --release --bin embassy --features embassy

# Connect to serial
screen /dev/tty.usbmodemnut_shell1 115200
```

**Key implementation details:**
- Embassy-usb for full-speed USB CDC device
- Packet chunking for output >64 bytes
- Deferred flush pattern (see docs/CHAR_IO.md)
- RefCell for shared buffer access between `Shell` and flush logic

---

## Feature Configuration

```bash
# With authentication
cargo build --release --bin basic --features authentication

# Minimal (no optional features)
cargo build --release --bin basic --no-default-features

# Embassy with all features
cargo build --release --bin embassy --features embassy,authentication,completion,history
```

Available features: `authentication`, `completion`, `history`, `embassy`

---

## Hardware Verification

These examples have been tested and verified working on physical Raspberry Pi Pico hardware:
- ✅ Basic example: USB CDC communication working
- ✅ Embassy example: USB CDC with async working
- ✅ All commands functional
- ✅ Authentication, tab completion, and command history working

## License

Same as parent **nut-shell** project (MIT OR Apache-2.0).
