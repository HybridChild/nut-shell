//! Build-time credentials example for Raspberry Pi Pico
//!
//! Demonstrates using nut-shell with credentials generated at build time
//! from a TOML configuration file. Credentials are hashed during build
//! and compiled into the binary as const data.
//!
//! # Build Flow
//! 1. build.rs runs nut-shell-credgen on credentials.toml
//! 2. Generated credentials.rs written to OUT_DIR
//! 3. This code includes credentials.rs at compile time
//! 4. Credentials are const-initializable (no heap, no runtime init)
//!
//! # Hardware Setup
//! - USB connection provides serial interface
//! - No external UART adapter needed
//! - Connect Pico directly to computer via USB
//!
//! # Default Credentials
//! See credentials.toml:
//! - admin:admin123 (Admin access)
//! - user:user123 (User access)

#![no_std]
#![no_main]

mod handler;
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

use crate::handler::PicoHandler;
use crate::io::UsbCharIo;
use crate::tree::ROOT;

// =============================================================================
// Build-Time Generated Credentials
// =============================================================================

// Include generated credentials module from build script
mod credentials {
    include!(concat!(env!("OUT_DIR"), "/credentials.rs"));
}

// Import the access level type from library
use rp_pico_buildtime::PicoAccessLevel;

// =============================================================================
// Main Entry Point
// =============================================================================

#[entry]
fn main() -> ! {
    // Get peripheral access
    let pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Initialize all hardware (clocks, GPIO, USB)
    let hw_config = hw_setup::init_hardware(pac, core);
    let mut delay = hw_config.delay;

    // Create CharIo wrapper
    let io = UsbCharIo::new();

    // Create handler
    let handler = PicoHandler;

    // Create credential provider using build-time generated credentials
    // This provider is created from const data - no heap allocation!
    let provider = credentials::create_provider();

    // Create shell with authentication enabled
    let mut shell: Shell<PicoAccessLevel, UsbCharIo, PicoHandler, DefaultConfig> =
        Shell::new(&ROOT, handler, &provider, io);

    // Activate shell (show welcome and prompt)
    shell.activate().ok();

    // Main polling loop
    loop {
        // Poll USB device to handle USB events
        io::poll_usb();

        // Poll for incoming characters and process them
        shell.poll().ok();

        // Small delay to prevent busy-waiting
        delay.delay_us(100u32);
    }
}
