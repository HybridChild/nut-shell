//! UART CharIo implementation for basic (non-async) example

use rp2040_hal::{
    gpio::{FunctionUart, Pin, PullDown},
    pac,
    uart::UartPeripheral,
};

use nut_shell::io::CharIo;

// =============================================================================
// UART CharIo Implementation
// =============================================================================

pub type UartPins = (
    Pin<rp2040_hal::gpio::bank0::Gpio0, FunctionUart, PullDown>,
    Pin<rp2040_hal::gpio::bank0::Gpio1, FunctionUart, PullDown>,
);
pub type UartType = UartPeripheral<rp2040_hal::uart::Enabled, pac::UART0, UartPins>;

pub struct UartCharIo {
    uart: UartType,
}

impl UartCharIo {
    pub fn new(uart: UartType) -> Self {
        Self { uart }
    }
}

impl CharIo for UartCharIo {
    type Error = ();

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Non-blocking read
        if self.uart.uart_is_readable() {
            let mut buf = [0u8; 1];
            match self.uart.read_raw(&mut buf) {
                Ok(n) if n > 0 => Ok(Some(buf[0] as char)),
                Ok(_) => Ok(None),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Blocking write for simplicity
        self.uart.write_full_blocking(&[c as u8]);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.uart.write_full_blocking(s.as_bytes());
        Ok(())
    }
}
