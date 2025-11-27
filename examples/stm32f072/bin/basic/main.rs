//! STM32F072 (NUCLEO-F072RB) UART example
//!
//! This example demonstrates nut-shell running on STM32F072 hardware with UART communication.
//!
//! # Hardware Setup
//! - Board: NUCLEO-F072RB
//! - UART: USART2 (PA2=TX, PA3=RX) connected to ST-LINK VCP
//! - Baud rate: 115200
//! - LED: PA5 (User LED LD2)

#![no_std]
#![no_main]

mod handlers;
mod hw_setup;
mod hw_state;
mod io;
mod tree;

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f0xx_hal::pac;

use nut_shell::{
    config::MinimalConfig,
    shell::Shell,
};

use stm32_examples::Stm32AccessLevel;

use crate::handlers::Stm32Handlers;
use crate::io::UartCharIo;
use crate::tree::ROOT;

// =============================================================================
// Main Entry Point
// =============================================================================

#[entry]
fn main() -> ! {
    // Get peripheral access
    let pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Initialize all hardware (clocks, GPIO, UART, LED)
    let hw_config = hw_setup::init_hardware(pac, core);
    let mut delay = hw_config.delay;

    // Create CharIo wrapper
    let io = UartCharIo::new(hw_config.uart_tx, hw_config.uart_rx);

    // Create handlers
    let handlers = Stm32Handlers;

    // Create shell with minimal configuration for resource-constrained STM32
    // MinimalConfig uses smaller buffers (64-byte input, 128-byte response)
    // to reduce stack usage on devices with limited RAM (16KB on STM32F072)
    let mut shell: Shell<Stm32AccessLevel, UartCharIo, Stm32Handlers, MinimalConfig> =
        Shell::new(&ROOT, handlers, io);

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
