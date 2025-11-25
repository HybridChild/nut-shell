//! RP2040 (Raspberry Pi Pico) UART example
//!
//! This example demonstrates nut-shell running on RP2040 hardware with UART communication.
//!
//! # Hardware Setup
//! - UART TX: GP0
//! - UART RX: GP1
//! - Baud rate: 115200
//!
//! # Building
//! ```bash
//! cd examples/rp-pico
//! cargo build --release --bin uart_cli
//! ```
//!
//! # Flashing
//! ```bash
//! # Using picotool
//! picotool load -x target/thumbv6m-none-eabi/release/uart_cli
//!
//! # Or using elf2uf2-rs
//! elf2uf2-rs target/thumbv6m-none-eabi/release/uart_cli uart_cli.uf2
//! # Then copy uart_cli.uf2 to the RPI-RP2 drive
//! ```
//!
//! # Connecting
//! Connect to the serial port at 115200 baud:
//! ```bash
//! # Linux
//! screen /dev/ttyACM0 115200
//!
//! # macOS
//! screen /dev/tty.usbmodem* 115200

#![no_std]
#![no_main]

mod handlers;
mod tree;

use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use fugit::HertzU32;
use panic_halt as _;

// Link in the Boot ROM - required for RP2040
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

use rp2040_hal::{
    Sio,
    clocks::{Clock, init_clocks_and_plls},
    gpio::{FunctionUart, Pin, PullDown},
    pac,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
};

use nut_shell::{
    config::DefaultConfig,
    io::CharIo,
    shell::Shell,
};

use rp_pico_examples::{PicoAccessLevel, PicoCredentialProvider};

use crate::handlers::PicoHandlers;
use crate::tree::ROOT;

// =============================================================================
// UART CharIo Implementation
// =============================================================================

type UartPins = (
    Pin<rp2040_hal::gpio::bank0::Gpio0, FunctionUart, PullDown>,
    Pin<rp2040_hal::gpio::bank0::Gpio1, FunctionUart, PullDown>,
);
type UartType = UartPeripheral<rp2040_hal::uart::Enabled, pac::UART0, UartPins>;

struct UartCharIo {
    uart: UartType,
}

impl UartCharIo {
    fn new(uart: UartType) -> Self {
        Self { uart }
    }
}

impl CharIo for UartCharIo {
    type Error = ();

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Non-blocking read
        if self.uart.uart_is_readable() {
            let mut buf = [0u8; 1];
            match self.uart.read_raw(&mut buf) {
                Ok(n) if n > 0 => Ok(Some(buf[0] as char)),
                Ok(_) => Ok(None),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Blocking write for simplicity
        self.uart.write_full_blocking(&[c as u8]);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.uart.write_full_blocking(s.as_bytes());
        Ok(())
    }
}

// =============================================================================
// Main Entry Point
// =============================================================================

#[entry]
fn main() -> ! {
    // Get peripheral access
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up watchdog
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // Configure clocks
    let xosc_crystal_freq = 12_000_000; // 12 MHz crystal on Pico
    let clocks = init_clocks_and_plls(
        xosc_crystal_freq,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Set up delay
    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Set up GPIO
    let sio = Sio::new(pac.SIO);
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure UART on GP0 (TX) and GP1 (RX)
    let uart_pins = (
        pins.gpio0.into_function::<FunctionUart>(),
        pins.gpio1.into_function::<FunctionUart>(),
    );

    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(
                HertzU32::from_raw(115200),
                DataBits::Eight,
                None,
                StopBits::One,
            ),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    // Create CharIo wrapper
    let io = UartCharIo::new(uart);

    // Create handlers
    let handlers = PicoHandlers;

    // Create shell with authentication
    let provider = PicoCredentialProvider::new();
    let mut shell: Shell<PicoAccessLevel, UartCharIo, PicoHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, &provider, io);

    // Activate shell (show welcome and prompt)
    shell.activate().ok();

    // Main polling loop
    // The shell.poll() method checks for incoming UART characters and processes them
    loop {
        // Poll for incoming characters and process them
        shell.poll().ok();

        // Small delay to prevent busy-waiting and reduce CPU usage
        delay.delay_us(100u32);
    }
}
