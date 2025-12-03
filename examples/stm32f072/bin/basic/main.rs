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

mod handler;
mod hw_setup;
mod hw_state;
mod io;
mod systick;
mod tree;

use cortex_m_rt::{entry, exception};
use panic_halt as _;
use stm32f0xx_hal::pac;

use nut_shell::{config::MinimalConfig, shell::Shell};

use stm32_examples::Stm32AccessLevel;
#[cfg(feature = "authentication")]
use stm32_examples::Stm32CredentialProvider;

use crate::handler::Stm32Handler;
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

    // Initialize all hardware (clocks, GPIO, UART, LED, ADC, SysTick)
    let hw_config = hw_setup::init_hardware(pac, core);

    // Capture boot time (after SysTick is running)
    let boot_time_ms = systick::millis();
    stm32_examples::init_boot_time(boot_time_ms);

    // Initialize ADC for temperature sensor
    hw_state::init_adc(hw_config.adc);

    // Create CharIo wrapper
    let io = UartCharIo::new(hw_config.uart_tx, hw_config.uart_rx);

    // Create handler
    let handler = Stm32Handler;

    // Create shell with minimal configuration for resource-constrained STM32
    // MinimalConfig uses smaller buffers (64-byte input, 128-byte response)
    // to reduce stack usage on devices with limited RAM (16KB on STM32F072)
    #[cfg(feature = "authentication")]
    let provider = Stm32CredentialProvider::new();
    #[cfg(feature = "authentication")]
    let mut shell: Shell<Stm32AccessLevel, UartCharIo, Stm32Handler, MinimalConfig> =
        Shell::new(&ROOT, handler, &provider, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell: Shell<Stm32AccessLevel, UartCharIo, Stm32Handler, MinimalConfig> =
        Shell::new(&ROOT, handler, io);

    // Activate shell (show welcome and prompt)
    shell.activate().ok();

    // Main polling loop
    // The shell.poll() method checks for incoming UART characters and processes them
    loop {
        // Poll for incoming characters and process them
        shell.poll().ok();

        // Note: No delay needed - SysTick provides timing, and polling is non-blocking
    }
}

// =============================================================================
// Interrupt Handlers
// =============================================================================

/// SysTick interrupt handler
///
/// Called every 1ms to increment the global millisecond counter.
/// This provides accurate uptime tracking.
#[exception]
fn SysTick() {
    systick::increment_millis();
}
