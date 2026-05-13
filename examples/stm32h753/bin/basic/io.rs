//! USB CDC CharIo implementation for NUCLEO-H753ZI
//!
//! Uses OTG2_HS (embedded full-speed PHY) via CN13 (USB Micro-AB).
//! Global Mutex/RefCell pattern identical to the RP2040 basic example.

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use stm32h7xx_hal::usb_hs::UsbBus;
use usb_device::prelude::*;
use usbd_serial::SerialPort;

use nut_shell::io::CharIo;

// =============================================================================
// USB peripheral type alias
// =============================================================================

// The OTG2_HS peripheral wrapper from stm32h7xx-hal
use stm32h7xx_hal::usb_hs::USB2;

type UsbBusType = UsbBus<USB2>;

// =============================================================================
// Global USB state
// =============================================================================

static USB_DEVICE: Mutex<RefCell<Option<UsbDevice<'static, UsbBusType>>>> =
    Mutex::new(RefCell::new(None));
static USB_SERIAL: Mutex<RefCell<Option<SerialPort<'static, UsbBusType>>>> =
    Mutex::new(RefCell::new(None));

// =============================================================================
// Initialization
// =============================================================================

/// Store the initialized USB device and serial port into globals.
///
/// Must be called once from `hw_setup::init_hardware()` before the main loop.
pub fn init_usb(
    usb_device: UsbDevice<'static, UsbBusType>,
    serial: SerialPort<'static, UsbBusType>,
) {
    cortex_m::interrupt::free(|cs| {
        USB_DEVICE.borrow(cs).replace(Some(usb_device));
        USB_SERIAL.borrow(cs).replace(Some(serial));
    });
}

// =============================================================================
// USB polling
// =============================================================================

/// Drive USB events. Must be called frequently from the main loop (every 1–10 ms).
///
/// This processes incoming USB packets and keeps the device enumerated.
pub fn poll_usb() {
    cortex_m::interrupt::free(|cs| {
        if let (Some(device), Some(serial)) = (
            USB_DEVICE.borrow(cs).borrow_mut().as_mut(),
            USB_SERIAL.borrow(cs).borrow_mut().as_mut(),
        ) {
            device.poll(&mut [serial]);
        }
    });
}

// =============================================================================
// CharIo implementation
// =============================================================================

pub struct UsbCharIo;

impl UsbCharIo {
    pub fn new() -> Self {
        Self
    }
}

impl CharIo for UsbCharIo {
    type Error = ();

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        cortex_m::interrupt::free(|cs| {
            if let Some(serial) = USB_SERIAL.borrow(cs).borrow_mut().as_mut() {
                let mut buf = [0u8; 1];
                match serial.read(&mut buf) {
                    Ok(n) if n > 0 => Ok(Some(buf[0] as char)),
                    _ => Ok(None),
                }
            } else {
                Ok(None)
            }
        })
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.write_str(core::str::from_utf8(&[c as u8]).unwrap_or(""))
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        cortex_m::interrupt::free(|cs| {
            if let Some(serial) = USB_SERIAL.borrow(cs).borrow_mut().as_mut() {
                let mut bytes = s.as_bytes();
                while !bytes.is_empty() {
                    match serial.write(bytes) {
                        Ok(n) if n > 0 => bytes = &bytes[n..],
                        _ => break,
                    }
                }
            }
            Ok(())
        })
    }
}
