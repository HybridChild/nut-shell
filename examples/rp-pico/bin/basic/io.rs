//! USB CDC CharIo implementation for basic (non-async) example

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use rp2040_hal::usb::UsbBus;
use usb_device::prelude::*;
use usbd_serial::SerialPort;

use nut_shell::io::CharIo;

// =============================================================================
// USB CDC CharIo Implementation
// =============================================================================

/// Global USB device and serial port protected by Mutex
static USB_DEVICE: Mutex<RefCell<Option<UsbDevice<'static, UsbBus>>>> = Mutex::new(RefCell::new(None));
static USB_SERIAL: Mutex<RefCell<Option<SerialPort<'static, UsbBus>>>> = Mutex::new(RefCell::new(None));

/// Initialize USB device and serial port
///
/// This must be called once during startup with the initialized USB bus.
pub fn init_usb(
    usb_device: UsbDevice<'static, UsbBus>,
    serial: SerialPort<'static, UsbBus>,
) {
    cortex_m::interrupt::free(|cs| {
        USB_DEVICE.borrow(cs).replace(Some(usb_device));
        USB_SERIAL.borrow(cs).replace(Some(serial));
    });
}

/// Poll USB device (must be called regularly from main loop)
///
/// This handles USB events and keeps the connection alive.
/// Call this frequently (every 1-10ms) from your main loop.
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
                // Write in chunks if needed
                let mut bytes = s.as_bytes();
                while !bytes.is_empty() {
                    match serial.write(bytes) {
                        Ok(n) if n > 0 => bytes = &bytes[n..],
                        _ => break, // Stop if write fails or no data written
                    }
                }
            }
            Ok(())
        })
    }
}
