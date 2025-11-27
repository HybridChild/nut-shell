//! UART CharIo implementation for STM32F072

use stm32f0xx_hal::{
    pac::USART2,
    prelude::*,
    serial::{Tx, Rx},
};

use nut_shell::io::CharIo;

// =============================================================================
// UART CharIo Implementation
// =============================================================================

pub type UartTx = Tx<USART2>;
pub type UartRx = Rx<USART2>;

pub struct UartCharIo {
    tx: UartTx,
    rx: UartRx,
}

impl UartCharIo {
    pub fn new(tx: UartTx, rx: UartRx) -> Self {
        Self { tx, rx }
    }
}

impl CharIo for UartCharIo {
    type Error = ();

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Non-blocking read - return None if no data available
        match self.rx.read() {
            Ok(byte) => Ok(Some(byte as char)),
            Err(nb::Error::WouldBlock) => Ok(None),
            Err(_) => Err(()),
        }
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Blocking write for simplicity
        nb::block!(self.tx.write(c as u8)).map_err(|_| ())?;
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.chars() {
            self.put_char(c)?;
        }
        Ok(())
    }
}
