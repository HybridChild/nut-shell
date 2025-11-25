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
//! # Building
//! ```bash
//! cd examples/rp-pico
//! cargo build --release --bin embassy_uart_cli
//! ```
//!
//! # Flashing
//! ```bash
//! # Using picotool
//! picotool load -x target/thumbv6m-none-eabi/release/embassy_uart_cli
//!
//! # Or using elf2uf2-rs
//! elf2uf2-rs target/thumbv6m-none-eabi/release/embassy_uart_cli embassy_uart_cli.uf2
//! # Then copy embassy_uart_cli.uf2 to the RPI-RP2 drive
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
//! ```
//!
//! # Features
//! - Embassy async runtime
//! - Async command execution with `process_char_async()`
//! - Buffered UART I/O (deferred flush pattern)
//! - Async delay command demonstration

#![no_std]
#![no_main]

mod handlers;
mod tree;

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
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
    io::CharIo,
    shell::Shell,
};

use rp_pico_examples::{PicoAccessLevel, PicoCredentialProvider};

use crate::handlers::{LedCommand, PicoHandlers};
use crate::tree::ROOT;

// Bind UART interrupt handler
bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

// =============================================================================
// UART CharIo Implementation (Buffered for Embassy)
// =============================================================================

/// Buffered UART I/O adapter for Embassy.
///
/// Implements the deferred flush pattern described in IO_DESIGN.md:
/// - `put_char()` and `write_str()` buffer to memory only
/// - Output is stored in an internal buffer accessed via RefCell
struct BufferedUartCharIo {
    output_buffer: &'static RefCell<heapless::Vec<u8, 512>>,
}

impl BufferedUartCharIo {
    fn new(output_buffer: &'static RefCell<heapless::Vec<u8, 512>>) -> Self {
        Self { output_buffer }
    }

    /// Check if buffer has data to flush
    fn has_data(&self) -> bool {
        !self.output_buffer.borrow().is_empty()
    }

    /// Get buffered data for flushing
    fn take_buffer(&self) -> heapless::Vec<u8, 512> {
        let mut buf = self.output_buffer.borrow_mut();
        let data = buf.clone();
        buf.clear();
        data
    }
}

impl CharIo for BufferedUartCharIo {
    type Error = ();

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Not used in async pattern - read happens externally
        Ok(None)
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Buffer to memory only (deferred flush pattern)
        self.output_buffer.borrow_mut().push(c as u8).ok();
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        // Buffer to memory only (deferred flush pattern)
        let mut buf = self.output_buffer.borrow_mut();
        for c in s.bytes() {
            buf.push(c).ok();
        }
        Ok(())
    }
}

// =============================================================================
// Embassy Tasks
// =============================================================================

/// LED control task.
#[embassy_executor::task]
async fn led_task(
    mut led: Output<'static>,
    channel: &'static Channel<ThreadModeRawMutex, LedCommand, 1>,
) {
    loop {
        match channel.receive().await {
            LedCommand::On => led.set_high(),
            LedCommand::Off => led.set_low(),
        }
    }
}

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

    // Create handlers
    let handlers = PicoHandlers { led_channel };

    // Create credential provider (runtime initialization)
    let provider = PicoCredentialProvider::new();

    // Create shell
    let mut shell: Shell<PicoAccessLevel, BufferedUartCharIo, PicoHandlers, DefaultConfig> =
        Shell::new(&ROOT, handlers, &provider, io);

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
    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Set up onboard LED (GP25)
    let led = Output::new(p.PIN_25, Level::Low);

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
        Irqs,
        tx_buf,
        rx_buf,
        uart::Config::default(),
    );
    let (tx, rx) = uart.split();

    // Spawn tasks
    spawner.spawn(led_task(led, led_channel)).unwrap();
    spawner.spawn(shell_task(tx, rx, led_channel)).unwrap();
}
