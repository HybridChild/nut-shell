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

Complete interactive CLI over UART demonstrating **nut-shell** on STM32 hardware.

**Commands Available:**

```
/
├── system/
│   ├── info       - Show device information
│   ├── uptime     - Show system uptime
│   ├── meminfo    - Display memory usage statistics
│   ├── benchmark  - Run CPU performance benchmark
│   ├── flash      - Show flash memory information
│   └── crash      - Trigger controlled panic (Admin only!)
│
└── hardware/
    ├── get/
    │   ├── temp       - Read internal temperature sensor
    │   ├── chipid     - Display unique device ID (96-bit)
    │   ├── clocks     - Show clock frequencies
    │   ├── core       - Display CPU core information
    │   └── bootreason - Show last reset reason
    │
    └── set/
        └── led        - Control USER LED (on/off)

Global:
  ?      - Show help
  ls     - List directory contents
  clear  - Clear screen
  logout - End session (authentication only)
```

**Authentication** (when enabled):
- `admin:admin123` - Admin access
- `user:user123` - User access

**Memory Usage (Release Build):**
- Flash: ~35KB (minimal) to ~45KB (all features)
- RAM: <2KB static allocation
- No heap allocation (pure `no_std`)

Feature impact on flash:
- Base (no features): 35KB
- +completion: +1KB
- +history: +2KB
- +authentication: +7KB

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

# Minimal (no optional features - debug build supported)
cargo build --bin basic --no-default-features

# Custom combinations
cargo build --release --bin basic --features completion,history
cargo build --release --bin basic --features authentication,completion,history
```

Available features: `authentication`, `completion`, `history`

**Important:** Due to the STM32F072's limited flash memory (128KB), **any build with optional features enabled requires release mode**. Debug builds overflow flash memory with any feature combination except the minimal no-features configuration. Release builds with all features fit comfortably (~42KB flash).

---

## Hardware Verification

This example has been verified to build and run successfully on physical NUCLEO-F072RB hardware with all feature combinations.

## License

Same as parent **nut-shell** project (MIT OR Apache-2.0).
