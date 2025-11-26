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

use core::cell::RefCell;
use cortex_m::delay::Delay;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use embedded_hal_0_2::adc::OneShot;
use embedded_hal_0_2::digital::v2::OutputPin;
use fugit::HertzU32;
use nb;
use panic_halt as _;

// Link in the Boot ROM - required for RP2040
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

use rp2040_hal::{
    Sio,
    adc::Adc,
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

use rp_pico_examples::{PicoAccessLevel, PicoCredentialProvider, hw_commands, init_chip_id, init_reset_reason};

use crate::handlers::PicoHandlers;
use crate::tree::ROOT;

// =============================================================================
// Global LED State
// =============================================================================

type LedPin = Pin<
    rp2040_hal::gpio::bank0::Gpio25,
    rp2040_hal::gpio::FunctionSio<rp2040_hal::gpio::SioOutput>,
    rp2040_hal::gpio::PullDown,
>;

/// Global LED pin protected by a Mutex for safe access from command handlers
static LED_PIN: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

/// Set the LED state (on = true, off = false)
pub fn set_led(on: bool) {
    cortex_m::interrupt::free(|cs| {
        if let Some(led) = LED_PIN.borrow(cs).borrow_mut().as_mut() {
            if on {
                let _ = led.set_high();
            } else {
                let _ = led.set_low();
            }
        }
    });

    // Update GPIO cache to reflect current state
    // Pin 25: output (1), value (0=low or 1=high), no pull (0)
    hw_commands::update_gpio_state(25, 1, if on { 1 } else { 0 }, 0);
}

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
    // Cache reset reason FIRST, before any HAL initialization
    // The WATCHDOG REASON register auto-clears on first read
    init_reset_reason();

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

    // Configure onboard LED (GP25) as output
    let led = pins.gpio25.into_push_pull_output();

    // Store LED in global static
    cortex_m::interrupt::free(|cs| {
        LED_PIN.borrow(cs).replace(Some(led));
    });

    // Update GPIO cache for LED pin (pin 25: output, low, no pull)
    hw_commands::update_gpio_state(25, 1, 0, 0);

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

    // Initialize hardware status cache
    init_chip_id();

    // Initialize clock frequency cache
    hw_commands::update_clocks(
        clocks.system_clock.freq().to_Hz(),
        clocks.usb_clock.freq().to_Hz(),
        clocks.peripheral_clock.freq().to_Hz(),
        clocks.adc_clock.freq().to_Hz(),
    );

    // Register LED control function
    hw_commands::register_led_control(set_led);

    // Initialize ADC for temperature readings
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let mut temp_sensor = adc.take_temp_sensor().unwrap();

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
    let mut ticks: u64 = 0;
    let mut last_temp_update: u64 = 0;

    loop {
        // Poll for incoming characters and process them
        shell.poll().ok();

        // Update temperature cache every 500ms (5000 ticks @ 100us per tick)
        if ticks.wrapping_sub(last_temp_update) >= 5000 {
            // Read temperature sensor (OneShot trait returns nb::Result)
            let read_result: nb::Result<u16, _> = adc.read(&mut temp_sensor);
            if let Ok(adc_value) = read_result {

                // Convert ADC value to temperature (RP2040 formula)
                // T = 27 - (ADC_voltage - 0.706) / 0.001721
                // ADC_voltage = (adc_value * 3.3) / 4096
                let adc_voltage = (adc_value as f32 * 3.3) / 4096.0;
                let temp_celsius = 27.0 - (adc_voltage - 0.706) / 0.001721;

                hw_commands::update_temperature(temp_celsius);
                last_temp_update = ticks;
            }
        }

        ticks = ticks.wrapping_add(1);

        // Small delay to prevent busy-waiting and reduce CPU usage
        delay.delay_us(100u32);
    }
}
