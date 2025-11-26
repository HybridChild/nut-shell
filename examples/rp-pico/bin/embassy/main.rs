//! RP2040 (Raspberry Pi Pico) Embassy UART example with async support
//!
//! This example demonstrates nut-shell running on RP2040 hardware with Embassy async runtime,
//! showcasing async command execution with UART communication.
//!
//! # Hardware Setup
//! - UART TX: GP0
//! - UART RX: GP1
//! - Baud rate: 115200
//!
//! # Features
//! - Embassy async runtime
//! - Async command execution with `process_char_async()`
//! - Buffered UART I/O (deferred flush pattern)
//! - Async delay command demonstration

#![no_std]
#![no_main]

mod handlers;
mod hw_setup;
mod hw_state;
mod io;
mod tasks;
mod tree;

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{self, BufferedInterruptHandler, BufferedUart, BufferedUartRx, BufferedUartTx},
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_io_async::{Read as AsyncRead, Write as AsyncWrite};
use heapless;
use panic_halt as _;
use static_cell::StaticCell;

use nut_shell::{
    config::DefaultConfig,
    shell::Shell,
};

use rp_pico_examples::{PicoAccessLevel, hw_commands, init_boot_time, init_chip_id, init_reset_reason};

#[cfg(feature = "authentication")]
use rp_pico_examples::PicoCredentialProvider;

use crate::handlers::{LedCommand, PicoHandlers};
use crate::io::BufferedUartCharIo;
use crate::tree::ROOT;

// Bind UART interrupt handler
bind_interrupts!(struct UartIrqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

// =============================================================================
// Shell Task
// =============================================================================

/// Shell task with async command processing.
#[embassy_executor::task]
async fn shell_task(
    mut tx: BufferedUartTx,
    mut rx: BufferedUartRx,
    led_channel: &'static Channel<ThreadModeRawMutex, LedCommand, 1>,
) {
    // Create output buffer wrapped in RefCell for interior mutability
    static OUTPUT_BUFFER: StaticCell<RefCell<heapless::Vec<u8, 512>>> = StaticCell::new();
    let output_buffer = OUTPUT_BUFFER.init(RefCell::new(heapless::Vec::new()));

    // Create buffered I/O (we'll create two references to the same buffer)
    let io = BufferedUartCharIo::new(output_buffer);
    let io_flush = BufferedUartCharIo::new(output_buffer); // Second reference for flushing

    // Initialize hardware status cache (done in main, before spawning tasks)

    // Create handlers
    let handlers = PicoHandlers { led_channel };

    // Create credential provider (must live as long as shell)
    #[cfg(feature = "authentication")]
    let provider = PicoCredentialProvider::new();

    // Create shell (with or without authentication based on feature flag)
    #[cfg(feature = "authentication")]
    let mut shell: Shell<PicoAccessLevel, BufferedUartCharIo, PicoHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, &provider, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell: Shell<PicoAccessLevel, BufferedUartCharIo, PicoHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, io);

    // Activate shell
    shell.activate().ok();

    // Flush initial output (welcome message)
    if io_flush.has_data() {
        let data = io_flush.take_buffer();
        AsyncWrite::write_all(&mut tx, &data).await.ok();
    }

    // Main async loop
    loop {
        // Read character from UART (async)
        let mut buf = [0u8; 1];
        match AsyncRead::read_exact(&mut rx, &mut buf).await {
            Ok(_) => {
                let c = buf[0] as char;

                // Process character (async)
                shell.process_char_async(c).await.ok();

                // Flush buffered output (deferred flush pattern)
                if io_flush.has_data() {
                    let data = io_flush.take_buffer();
                    AsyncWrite::write_all(&mut tx, &data).await.ok();
                }
            }
            Err(_) => {
                // UART error - could log or handle
                Timer::after(Duration::from_millis(100)).await;
            }
        }
    }
}

// =============================================================================
// Main Entry Point
// =============================================================================

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Cache hardware state FIRST, before any HAL initialization
    // The WATCHDOG REASON register auto-clears on first read
    init_reset_reason();
    init_boot_time();

    // Initialize Embassy runtime
    let p = embassy_rp::init(Default::default());

    // Initialize hardware status (chip ID must be read after HAL initialization)
    init_chip_id();

    // Initialize hardware peripherals (ADC, temperature sensor, LED)
    let hw_config = hw_setup::init_hardware(p.ADC, p.ADC_TEMP_SENSOR, p.PIN_25);

    // Register temperature sensor read function
    hw_commands::register_temp_sensor(hw_state::read_temperature);

    // Create LED command channel
    static LED_CHANNEL: StaticCell<Channel<ThreadModeRawMutex, LedCommand, 1>> = StaticCell::new();
    let led_channel = LED_CHANNEL.init(Channel::new());

    // Configure UART on GP0 (TX) and GP1 (RX)
    static TX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    let tx_buf = TX_BUF.init([0u8; 256]);
    let rx_buf = RX_BUF.init([0u8; 256]);

    let uart = BufferedUart::new(
        p.UART0,
        p.PIN_0,  // tx_pin
        p.PIN_1,  // rx_pin
        UartIrqs,
        tx_buf,
        rx_buf,
        uart::Config::default(),
    );
    let (tx, rx) = uart.split();

    // Spawn tasks
    spawner.spawn(tasks::led_task(hw_config.led, led_channel)).unwrap();
    spawner.spawn(tasks::temperature_monitor(hw_config.adc, hw_config.temp_channel)).unwrap();
    spawner.spawn(shell_task(tx, rx, led_channel)).unwrap();
}
