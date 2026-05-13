//! STM32H753ZI (NUCLEO-H753ZI) Embassy USB CDC example
//!
//! Demonstrates nut-shell on STM32H753ZI using the Embassy async executor
//! and USB CDC serial via the CN13 user USB connector (OTG_FS, 12 Mbps FS).
//!
//! # Task structure
//!
//! - `usb_task`: drives the USB device state machine (never returns)
//! - `shell_task`: owns the CDC ACM class, reads USB packets, feeds each byte
//!   to the shell via `process_char_async()`, and flushes buffered output back
//!   to USB after each packet
//!
//! Output is written through `UsbCharIo` into a static TX buffer; the shell
//! task flushes that buffer to USB in 64-byte chunks after each packet.
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
//! - USB kernel clock: 48 MHz (PLL3Q)

#![no_std]
#![no_main]

mod handler;
mod hw_setup;
mod hw_state;
mod io;
mod tree;

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::USB_OTG_FS;
use embassy_stm32::usb::{Driver, InterruptHandler};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::Timer;
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use panic_halt as _;
use static_cell::StaticCell;

use nut_shell::{config::DefaultConfig, shell::Shell};
use stm32h753zi_embassy_examples::H753AccessLevel;

use crate::handler::H753Handler;
use crate::io::{TxBuffer, UsbCharIo};
use crate::tree::ROOT;

// =============================================================================
// Interrupt binding
// =============================================================================

bind_interrupts!(struct Irqs {
    OTG_FS => InterruptHandler<peripherals::USB_OTG_FS>;
});

// =============================================================================
// Tasks
// =============================================================================

#[embassy_executor::task]
async fn usb_task(mut device: embassy_usb::UsbDevice<'static, Driver<'static, USB_OTG_FS>>) -> ! {
    device.run().await
}

#[embassy_executor::task]
async fn shell_task(
    mut usb_class: CdcAcmClass<'static, Driver<'static, USB_OTG_FS>>,
    tx_buf: &'static RefCell<TxBuffer>,
) {
    let io = UsbCharIo::new(tx_buf);
    let handler = H753Handler;

    #[cfg(feature = "authentication")]
    let provider = stm32h753zi_embassy_examples::create_h753_provider();

    #[cfg(feature = "authentication")]
    let mut shell: Shell<H753AccessLevel, UsbCharIo, H753Handler, DefaultConfig> =
        Shell::new(&ROOT, handler, &provider, io);

    #[cfg(not(feature = "authentication"))]
    let mut shell: Shell<H753AccessLevel, UsbCharIo, H753Handler, DefaultConfig> =
        Shell::new(&ROOT, handler, io);

    let mut usb_buf = [0u8; 64];

    loop {
        usb_class.wait_connection().await;
        shell.activate().ok();

        // Flush activation output (welcome message / prompt)
        flush_tx(&mut usb_class, tx_buf).await;

        loop {
            match usb_class.read_packet(&mut usb_buf).await {
                Ok(n) => {
                    for &byte in &usb_buf[..n] {
                        shell.process_char_async(byte as char).await.ok();
                    }
                    flush_tx(&mut usb_class, tx_buf).await;
                }
                Err(_) => break, // Disconnected
            }
        }
    }
}

/// Write all buffered TX bytes to USB in 64-byte chunks.
async fn flush_tx(
    usb_class: &mut CdcAcmClass<'static, Driver<'static, USB_OTG_FS>>,
    tx_buf: &'static RefCell<TxBuffer>,
) {
    let data = tx_buf.borrow_mut().take();
    let mut offset = 0;
    while offset < data.len() {
        let end = (offset + 64).min(data.len());
        if usb_class.write_packet(&data[offset..end]).await.is_err() {
            break;
        }
        offset = end;
    }
}

// =============================================================================
// Entry point
// =============================================================================

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // ------------------------------------------------------------------
    // 1. Initialize clocks (200 MHz SYSCLK, 48 MHz USB via PLL3Q)
    // ------------------------------------------------------------------
    let p = embassy_stm32::init(hw_setup::rcc_config());

    // ------------------------------------------------------------------
    // 2. GPIO — LEDs and USB power enable
    //    LD1 green: PB0, LD2 yellow: PE1, LD3 red: PB14
    //    USB power switch: PD10 (U18, SB77 ON = default)
    // ------------------------------------------------------------------
    let led1 = Output::new(p.PB0, Level::Low, Speed::Low);
    let led2 = Output::new(p.PE1, Level::Low, Speed::Low);
    let led3 = Output::new(p.PB14, Level::Low, Speed::Low);
    hw_state::init_leds(led1, led2, led3);

    let _usb_pwr_en = Output::new(p.PD10, Level::High, Speed::Low);

    // ------------------------------------------------------------------
    // 3. USB OTG_FS — PA11 (DM, AF10), PA12 (DP, AF10) → CN13
    //
    //    All buffers are allocated via StaticCell so the driver, device,
    //    and CDC class all carry 'static lifetimes and can be moved into
    //    tasks. ep_out_buffer must be in AXI SRAM (0x24000000 per
    //    memory.x) — accessible by USB DMA. DTCM must NOT be used.
    // ------------------------------------------------------------------
    static EP_OUT_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
    let ep_out_buffer = EP_OUT_BUFFER.init([0u8; 256]);

    let mut usb_config = embassy_stm32::usb::Config::default();
    usb_config.vbus_detection = false;

    let driver = Driver::new_fs(
        p.USB_OTG_FS,
        Irqs,
        p.PA12,
        p.PA11,
        ep_out_buffer,
        usb_config,
    );

    // ------------------------------------------------------------------
    // 4. Build USB device and CDC ACM class
    //    All descriptor buffers must be 'static so that the resulting
    //    UsbDevice and CdcAcmClass carry 'static lifetimes for task use.
    // ------------------------------------------------------------------
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    static CDC_STATE: StaticCell<embassy_usb::class::cdc_acm::State<'static>> = StaticCell::new();

    let mut usb_dev_config = embassy_usb::Config::new(0x16c0, 0x27dd);
    usb_dev_config.manufacturer = Some("nut-shell");
    usb_dev_config.product = Some("NUCLEO-H753ZI");
    usb_dev_config.serial_number = Some("embassy");

    let mut builder = Builder::new(
        driver,
        usb_dev_config,
        CONFIG_DESC.init([0; 256]),
        BOS_DESC.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64]),
    );

    let cdc_class = CdcAcmClass::new(
        &mut builder,
        CDC_STATE.init(embassy_usb::class::cdc_acm::State::new()),
        64,
    );

    let usb_device = builder.build();

    // ------------------------------------------------------------------
    // 5. Static TX buffer shared between UsbCharIo and shell_task
    // ------------------------------------------------------------------
    static TX_BUF: StaticCell<RefCell<TxBuffer>> = StaticCell::new();
    let tx_buf = TX_BUF.init(RefCell::new(TxBuffer::new()));

    // ------------------------------------------------------------------
    // 6. Spawn tasks
    // ------------------------------------------------------------------
    spawner.spawn(usb_task(usb_device).unwrap());

    // Small delay to let USB enumeration begin before accepting serial data
    Timer::after_millis(100).await;

    spawner.spawn(shell_task(cdc_class, tx_buf).unwrap());
}
