//! RP2040 (Raspberry Pi Pico) USB CDC example
//!
//! This example demonstrates nut-shell running on RP2040 hardware with USB CDC serial communication.
//!
//! # Hardware Setup
//! - USB connection provides serial interface
//! - No external UART adapter needed
//! - Connect Pico directly to computer via USB

#![no_std]
#![no_main]

mod handlers;
mod hw_setup;
mod hw_state;
mod io;
mod tree;

use cortex_m_rt::entry;
use panic_halt as _;
use rp2040_hal::pac;

// Link in the Boot ROM - required for RP2040
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

use nut_shell::{config::DefaultConfig, shell::Shell};

use rp_pico_examples::{PicoAccessLevel, init_boot_time, init_chip_id, init_reset_reason};

#[cfg(feature = "authentication")]
use rp_pico_examples::PicoCredentialProvider;

use crate::handlers::PicoHandlers;
use crate::io::UsbCharIo;
use crate::tree::ROOT;

// =============================================================================
// Main Entry Point
// =============================================================================

#[entry]
fn main() -> ! {
    // Cache hardware state FIRST, before any HAL initialization
    // The WATCHDOG REASON register auto-clears on first read
    init_reset_reason();
    init_boot_time();

    // Get peripheral access
    let pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Initialize all hardware (clocks, GPIO, ADC, LED, temperature sensor)
    // This also sets up USB and returns the USB bus allocator
    let hw_config = hw_setup::init_hardware(pac, core);
    let mut delay = hw_config.delay;

    // Create CharIo wrapper
    let io = UsbCharIo::new();

    // Initialize hardware status (chip ID must be read after HAL initialization)
    init_chip_id();

    // Create handlers
    let handlers = PicoHandlers;

    // Create credential provider (must live as long as shell)
    #[cfg(feature = "authentication")]
    let provider = PicoCredentialProvider::new();

    // Create shell (with or without authentication based on feature flag)
    #[cfg(feature = "authentication")]
    let mut shell: Shell<PicoAccessLevel, UsbCharIo, PicoHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, &provider, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell: Shell<PicoAccessLevel, UsbCharIo, PicoHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, io);

    // Activate shell (show welcome and prompt)
    shell.activate().ok();

    // Main polling loop
    // The shell.poll() method checks for incoming USB characters and processes them
    loop {
        // Poll USB device to handle USB events
        io::poll_usb();

        // Poll for incoming characters and process them
        shell.poll().ok();

        // Small delay to prevent busy-waiting and reduce CPU usage
        delay.delay_us(100u32);
    }
}
