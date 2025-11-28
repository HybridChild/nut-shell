# STM32 NUCLEO-F072RB Examples

Examples for STM32 NUCLEO-F072RB demonstrating **nut-shell** CLI framework on embedded hardware.

- **[basic](#basic)** - Complete interactive CLI over UART with optional authentication

## Hardware Setup

**NUCLEO-F072RB** development board:
- MCU: STM32F072RBT6 (ARM Cortex-M0, 128KB Flash, 16KB RAM)
- UART: USART2 (PA2/PA3) via ST-LINK virtual COM port @ 115200 baud
- Connection: Single USB cable provides both programming and serial communication

## Prerequisites

```bash
rustup target add thumbv6m-none-eabi
cargo install probe-rs-tools  # or use OpenOCD
```

## Examples

### basic

Complete interactive CLI over UART demonstrating nut-shell on STM32 hardware.

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

**Authentication** (when enabled):
- `admin:stm32admin` - Admin access
- `user:stm32user` - User access

**Memory Usage:**
- Flash: ~15KB (with all features)
- RAM: <2KB static allocation
- No heap allocation (pure `no_std`)

**Flash and connect:**

```bash
# Flash with probe-rs
cargo run --release --bin basic

# Connect to serial
screen /dev/ttyACM0 115200  # Linux
screen /dev/tty.usbmodem* 115200  # macOS
```

**Alternative: OpenOCD**

```bash
# Terminal 1
openocd -f interface/stlink.cfg -f target/stm32f0x.cfg

# Terminal 2
arm-none-eabi-gdb target/thumbv6m-none-eabi/release/basic
(gdb) target extended-remote :3333
(gdb) load
(gdb) continue
```

---

## Feature Configuration

```bash
# With authentication
cargo build --release --bin basic --features authentication

# Minimal (no optional features)
cargo build --release --bin basic --no-default-features

# Custom combinations
cargo build --release --bin basic --features completion,history
```

Available features: `authentication`, `completion`, `history`

---

## Hardware Verification

This example has been compiled and verified to build correctly for the thumbv6m-none-eabi target. Hardware testing on physical NUCLEO-F072RB boards is pending.

## License

Same as parent nut-shell project (MIT OR Apache-2.0).
