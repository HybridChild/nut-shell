# STM32 NUCLEO-H753ZI Examples

Examples for STM32 NUCLEO-H753ZI demonstrating **nut-shell** CLI framework on embedded hardware.

- **[basic](#basic)** - Complete interactive CLI over USB CDC with optional authentication

## Hardware Setup

**NUCLEO-H753ZI** development board:
- MCU: STM32H753ZIT6 (ARM Cortex-M7F, 2MB Flash, 1MB RAM)
- USB: OTG2_HS (embedded FS PHY, 12 Mbps) via CN13 (Micro-AB connector)
- No external UART adapter needed — USB CDC appears as a virtual COM port

**Connection order matters:**
1. Connect CN1 (ST-LINK Micro-USB) to power the board and enable flashing
2. Then connect CN13 (user Micro-AB USB) to your PC for the CLI serial port

> CN13 cannot power the board. Always power via CN1 first.

**Default solder bridges** — no modifications required on a stock NUCLEO-H753ZI:
- SB21/SB22: PA11/PA12 routed to CN13 (DM/DP)
- SB23: PA9 connected to CN13 VBUS sense
- SB76/SB77: Overcurrent alarm (PG7) and power switch enable (PD10) connected

## Prerequisites

```bash
rustup target add thumbv7em-none-eabihf
cargo install probe-rs-tools
```

## Examples

### basic

Complete interactive CLI over USB CDC demonstrating **nut-shell** on STM32H7 hardware.

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
    │   ├── chipid     - Display unique device ID (96-bit)
    │   ├── clocks     - Show RCC clock configuration
    │   ├── core       - Display Cortex-M7 CPUID information
    │   └── bootreason - Show last reset reason (RCC_RSR)
    │
    └── set/
        └── led <1|2|3> <on|off|toggle> - Control user LEDs
            LED 1 = LD1 green  (PB0)
            LED 2 = LD2 yellow (PE1)
            LED 3 = LD3 red    (PB14)

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
- Flash: well within 2MB for all feature combinations
- RAM: <4KB static allocation (512KB AXI SRAM available)
- No heap allocation (pure `no_std`)

**Flash and connect:**

```bash
# Flash with probe-rs (ST-LINK on CN1 must be connected)
cargo run --release --bin basic

# Connect to serial (CN13 must also be connected)
screen /dev/ttyACM0 115200        # Linux
screen /dev/tty.usbmodem* 115200  # macOS
```

---

## Feature Configuration

```bash
# Default (completion + history enabled)
cargo build --release --bin basic

# With authentication
cargo build --release --bin basic --features authentication

# Minimal (no optional features)
cargo build --release --bin basic --no-default-features

# All features
cargo build --release --bin basic --features authentication,completion,history
```

Available features: `authentication`, `completion`, `history`

Unlike the STM32F072 example, the H753ZI has sufficient flash and RAM for all feature combinations in both debug and release mode.

---

## Clock Configuration

| Clock | Source | Frequency |
|-------|--------|-----------|
| SYSCLK | HSI 64 MHz → PLL1 | 200 MHz |
| USB kernel | HSI48 (internal RC) | 48 MHz |
| VOS mode | VOS1 (default) | — |

HSI is used instead of HSE (ST-LINK MCO via SB45) because SB45 is not guaranteed to be closed or active across all NUCLEO-H753ZI board revisions. VOS1 is used because stm32h7xx-hal 0.16's CSI oscillator startup sequence is incompatible with the VOS0 transition on this board.

---

## Hardware Verification

This example has been verified to compile and run on NUCLEO-H753ZI hardware. USB CDC enumerates correctly on macOS (appears as `/dev/tty.usbmodembasic1`). All feature combinations compile on `thumbv7em-none-eabihf`.

## License

Same as parent **nut-shell** project (MIT OR Apache-2.0).
