//! RP2040 (Raspberry Pi Pico) Embassy USB CDC example with async support
//!
//! This example demonstrates nut-shell running on RP2040 hardware with Embassy async runtime,
//! showcasing async command execution with USB CDC serial communication.
//!
//! # Hardware Setup
//! - USB connection provides serial interface
//! - No external UART adapter needed
//! - Connect Pico directly to computer via USB
//!
//! # Features
//! - Embassy async runtime
//! - Async command execution with `process_char_async()`
//! - Buffered USB I/O (deferred flush pattern)
//! - Async delay command demonstration

#![no_std]
#![no_main]

mod handler;
mod hw_setup;
mod hw_state;
mod io;
mod tasks;
mod tree;

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_rp::{
    peripherals::USB,
    usb::{Driver, InterruptHandler},
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Builder, Config};
use heapless;
use panic_halt as _;
use static_cell::StaticCell;

use nut_shell::{config::DefaultConfig, shell::Shell};

use rp_pico_examples::{PicoAccessLevel, init_boot_time, init_chip_id, init_reset_reason};

#[cfg(feature = "authentication")]
use rp_pico_examples::PicoCredentialProvider;

use crate::handler::{LedCommand, PicoHandler};
use crate::io::BufferedCharIo;
use crate::tree::ROOT;

// Bind USB interrupt handler
embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

// =============================================================================
// Shell Task
// =============================================================================

/// Shell task with async command processing and USB CDC transport.
#[embassy_executor::task]
async fn shell_task(
    mut usb_class: CdcAcmClass<'static, Driver<'static, USB>>,
    led_channel: &'static Channel<ThreadModeRawMutex, LedCommand, 1>,
) {
    // Create output buffer wrapped in RefCell for interior mutability
    static OUTPUT_BUFFER: StaticCell<RefCell<heapless::Vec<u8, 512>>> = StaticCell::new();
    let output_buffer = OUTPUT_BUFFER.init(RefCell::new(heapless::Vec::new()));

    // Create buffered I/O (we'll create two references to the same buffer)
    let io = BufferedCharIo::new(output_buffer);
    let io_flush = BufferedCharIo::new(output_buffer); // Second reference for flushing

    // Create handler
    let handler = PicoHandler { led_channel };

    // Create credential provider (must live as long as shell)
    #[cfg(feature = "authentication")]
    let provider = PicoCredentialProvider::new();

    // Create shell (with or without authentication based on feature flag)
    #[cfg(feature = "authentication")]
    let mut shell: Shell<PicoAccessLevel, BufferedCharIo, PicoHandler, DefaultConfig> =
        Shell::new(&ROOT, handler, &provider, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell: Shell<PicoAccessLevel, BufferedCharIo, PicoHandler, DefaultConfig> =
        Shell::new(&ROOT, handler, io);

    // Wait for USB connection
    usb_class.wait_connection().await;

    // Activate shell
    shell.activate().ok();

    // Flush initial output (welcome message)
    if io_flush.has_data() {
        let data = io_flush.take_buffer();
        // Write all data, splitting into 64-byte packets as needed
        let mut offset = 0;
        while offset < data.len() {
            let chunk_size = (data.len() - offset).min(64);
            let chunk = &data[offset..offset + chunk_size];
            match usb_class.write_packet(chunk).await {
                Ok(_) => offset += chunk_size,
                Err(_) => break,
            }
        }
    }

    // Main async loop
    let mut usb_buf = [0u8; 64];
    loop {
        // Read from USB (async)
        match usb_class.read_packet(&mut usb_buf).await {
            Ok(n) if n > 0 => {
                // Process each character
                for &byte in &usb_buf[..n] {
                    let c = byte as char;

                    // Process character (async)
                    shell.process_char_async(c).await.ok();

                    // Flush buffered output after each character (deferred flush pattern)
                    if io_flush.has_data() {
                        let data = io_flush.take_buffer();
                        // Write all data, splitting into 64-byte packets as needed
                        let mut offset = 0;
                        while offset < data.len() {
                            let chunk_size = (data.len() - offset).min(64);
                            let chunk = &data[offset..offset + chunk_size];
                            match usb_class.write_packet(chunk).await {
                                Ok(_) => offset += chunk_size,
                                Err(_) => break, // Stop on error
                            }
                        }
                    }
                }
            }
            _ => {
                // USB error or disconnection
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

    // Initialize Embassy runtime (default config includes USB clock setup)
    let p = embassy_rp::init(Default::default());

    // Initialize hardware status (chip ID must be read after HAL initialization)
    init_chip_id();

    // Initialize hardware peripherals (ADC, temperature sensor, LED)
    let hw_config = hw_setup::init_hardware(p.ADC, p.ADC_TEMP_SENSOR, p.PIN_25);

    // Create LED command channel
    static LED_CHANNEL: StaticCell<Channel<ThreadModeRawMutex, LedCommand, 1>> = StaticCell::new();
    let led_channel = LED_CHANNEL.init(Channel::new());

    // Create USB driver
    let driver = Driver::new(p.USB, Irqs);

    // Create embassy-usb DeviceBuilder
    let mut config = Config::new(0x16c0, 0x27dd);
    config.manufacturer = Some("Raspberry Pi");
    config.product = Some("Pico");
    config.serial_number = Some("nut-shell");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Set device release version
    config.device_release = 0x0100;

    // Required buffers for USB descriptor building
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

    let mut builder = Builder::new(
        driver,
        config,
        CONFIG_DESC.init([0; 256]),
        BOS_DESC.init([0; 256]),
        &mut [], // no msos descriptors
        CONTROL_BUF.init([0; 64]),
    );

    // Create CDC-ACM class (serial port)
    static STATE: StaticCell<State> = StaticCell::new();
    let state = STATE.init(State::new());
    let usb_class = CdcAcmClass::new(&mut builder, state, 64);

    // Build the USB device
    let usb = builder.build();

    // Spawn USB task first to start enumeration
    spawner.spawn(embassy_usb_task(usb)).unwrap();

    // Small delay to allow USB enumeration to start
    Timer::after(Duration::from_millis(100)).await;

    // Spawn other tasks
    spawner
        .spawn(tasks::led_task(hw_config.led, led_channel))
        .unwrap();
    spawner
        .spawn(tasks::temperature_monitor(
            hw_config.adc,
            hw_config.temp_channel,
        ))
        .unwrap();
    spawner.spawn(shell_task(usb_class, led_channel)).unwrap();
}

/// USB device task (handles USB protocol)
#[embassy_executor::task]
async fn embassy_usb_task(mut usb: embassy_usb::UsbDevice<'static, Driver<'static, USB>>) -> ! {
    usb.run().await
}
