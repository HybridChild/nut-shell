# nut-shell — NUCLEO-H753ZI Embassy Example

Demonstrates **nut-shell** on the STM32H753ZIT6 (NUCLEO-H753ZI board) using the
[Embassy](https://embassy.dev/) async executor and USB CDC serial communication.

This is the Embassy counterpart to `examples/stm32h753/`, which uses bare-metal
`stm32h7xx-hal`. Both examples expose the same command tree on the same hardware.

## Hardware

- **Board**: NUCLEO-H753ZI (MB1364)
- **MCU**: STM32H753ZIT6 (Cortex-M7, 2 MB Flash, 1 MB RAM)
- **Interface**: USB CDC via CN13 (OTG_FS, 12 Mbps full-speed)

### Solder bridges (default — no modification required)

| Bridge | State | Function |
|--------|-------|----------|
| SB21   | ON    | PA11 → CN13 DM |
| SB22   | ON    | PA12 → CN13 DP |
| SB23   | ON    | PA9 → CN13 VBUS sense |
| SB76   | ON    | PG7 overcurrent alarm |
| SB77   | ON    | PD10 → U18 USB power switch |

## Setup

### Dependencies

This example uses a local Embassy checkout. The `[patch.crates-io]` section in
`Cargo.toml` points to `../../../embassy/` (three directories above this folder).
Adjust the paths if your checkout is elsewhere.

### Flash

```bash
# Connect ST-LINK (CN1) first, then CN13
cargo run --release
```

The board enumerates as a USB CDC serial device. Connect with any terminal at any
baud rate (e.g. `screen /dev/ttyACM0` or PuTTY on Windows).

## Clock configuration

| Clock      | Frequency | Source |
|------------|-----------|--------|
| HSI        | 64 MHz    | Internal RC |
| SYSCLK     | 200 MHz   | PLL1 (HSI/4 × 50 / 4), VOS1 |
| AHB / HCLK | 200 MHz  | SYSCLK / 1 |
| APB1–APB4  | 100 MHz   | AHB / 2 |
| USB kernel | 48 MHz    | PLL3Q (HSI/4 × 12 / 4) |

HSE (SB45) is not used — HSI is reliable across all board revisions.
`Usbsel::Hsi48` is not used — it panics on H753ZI if HSI48 is not running.

## Task architecture

```
embassy_executor (single-threaded, thread mode)
│
├── usb_task    — drives UsbDevice::run() (USB state machine)
└── shell_task  — owns CDC ACM class; reads USB packets, feeds each byte to
                  shell.process_char_async(), flushes buffered TX to USB
```

`UsbCharIo` buffers output into a static `TxBuffer`; the shell task flushes it
to USB in 64-byte chunks after each packet.

## Command tree

```
/
├── system/
│   ├── info         — Board and firmware information
│   ├── uptime       — System uptime (from embassy_time::Instant)
│   ├── meminfo      — Static RAM and flash usage
│   ├── benchmark    — CPU performance test
│   ├── flash        — Flash size and firmware footprint
│   └── crash        — Trigger panic [Admin only]
└── hardware/
    ├── get/
    │   ├── chipid   — 96-bit unique device ID
    │   ├── clocks   — Active clock frequencies
    │   ├── core     — Cortex-M7 CPUID register
    │   └── bootreason — Last reset flags (RCC_RSR)
    └── set/
        └── led <1|2|3> <on|off|toggle>
```

## Features

| Feature          | Default | Description |
|------------------|---------|-------------|
| `completion`     | yes     | Tab completion |
| `history`        | yes     | Up/down arrow command history |
| `authentication` | no      | Login with username/password |
| `async`          | no      | Async command execution support |

With `authentication` enabled, default credentials are:
- `admin` / `admin123` (Admin)
- `user` / `user123` (User)

> **Security note**: Hardcoded credentials are for demonstration only.

## Differences from `examples/stm32h753/`

| Aspect | stm32h753 (bare-metal) | stm32h753-embassy |
|--------|----------------------|-------------------|
| Executor | None (bare loop) | Embassy async |
| HAL | stm32h7xx-hal | embassy-stm32 |
| USB driver | usb-device + usbd-serial | embassy-usb CDC ACM |
| USB peripheral | USB2 (OTG2_HS regs) | USB_OTG_FS (OTG1_FS regs) |
| USB clock | HSI48 | PLL3Q |
| Uptime source | SysTick ISR counter | embassy_time::Instant |
| Concurrency | Polling loop | Async tasks |
