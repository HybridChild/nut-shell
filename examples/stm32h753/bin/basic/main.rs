//! STM32H753ZI (NUCLEO-H753ZI) USB CDC example
//!
//! Demonstrates nut-shell on STM32H753ZI with USB CDC serial communication
//! via the CN13 user USB connector (OTG2_HS, embedded FS PHY).
//!
//! # Hardware Setup
//! - Power the board via CN1 (ST-LINK USB) BEFORE connecting CN13
//! - Connect CN13 (USB Micro-AB, user USB) to your PC
//! - A new CDC serial port appears — open at any baud rate
//!
//! # Default solder bridges (no modification required)
//! - SB21/SB22: PA11/PA12 routed to CN13 (DM/DP)
//! - SB23: PA9 connected to CN13 VBUS sense
//! - SB76/SB77: Overcurrent and power-switch signals connected
//!
//! # Clock configuration
//! - HSI: 64 MHz (internal; HSE/SB45 not required)
//! - SYSCLK: 200 MHz (PLL1, VOS1)
//! - USB kernel clock: 48 MHz (HSI48)

#![no_std]
#![no_main]

mod handler;
mod hw_setup;
mod hw_state;
mod io;
mod systick;
mod tree;

use cortex_m_rt::{entry, exception};
use panic_halt as _;
use stm32h7xx_hal::pac;

use nut_shell::{config::DefaultConfig, shell::Shell};
use stm32h753zi_examples::H753AccessLevel;

use crate::handler::H753Handler;
use crate::io::UsbCharIo;
use crate::tree::ROOT;

// =============================================================================
// Entry point
// =============================================================================

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let core = cortex_m::Peripherals::take().unwrap();

    // Initialize clocks, GPIO, USB, and SysTick
    hw_setup::init_hardware(dp, core);

    let io = UsbCharIo::new();
    let handler = H753Handler;

    #[cfg(feature = "authentication")]
    let provider = stm32h753zi_examples::create_h753_provider();

    #[cfg(feature = "authentication")]
    let mut shell: Shell<H753AccessLevel, UsbCharIo, H753Handler, DefaultConfig> =
        Shell::new(&ROOT, handler, &provider, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell: Shell<H753AccessLevel, UsbCharIo, H753Handler, DefaultConfig> =
        Shell::new(&ROOT, handler, io);

    shell.activate().ok();

    loop {
        // Drive USB state machine — must be called frequently
        io::poll_usb();

        // Process any received character through the shell
        shell.poll().ok();
    }
}

// =============================================================================
// Interrupt handlers
// =============================================================================

/// Increment the millisecond counter at 1 kHz.
#[exception]
fn SysTick() {
    systick::increment_millis();
}
