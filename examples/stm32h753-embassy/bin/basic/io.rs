//! Buffered CharIo for NUCLEO-H753ZI Embassy example
//!
//! Implements the deferred flush pattern used by the RP2040 Embassy example:
//!
//! - `get_char()` always returns `None` — input is fed directly via
//!   `shell.process_char_async()` in the shell task, not through CharIo
//! - `put_char()` and `write_str()` buffer bytes into a static TX buffer
//! - The shell task flushes the TX buffer to USB after each processed packet
//!
//! Both the shell and the shell task share access to the same static buffers
//! through a `&'static RefCell<TxBuffer>` reference.

use core::cell::RefCell;
use nut_shell::io::CharIo;

// =============================================================================
// TX buffer
// =============================================================================

pub struct TxBuffer(pub heapless::Vec<u8, 512>);

impl TxBuffer {
    pub const fn new() -> Self {
        Self(heapless::Vec::new())
    }

    pub fn take(&mut self) -> heapless::Vec<u8, 512> {
        let data = self.0.clone();
        self.0.clear();
        data
    }
}

// =============================================================================
// CharIo implementation
// =============================================================================

pub struct UsbCharIo {
    tx: &'static RefCell<TxBuffer>,
}

impl UsbCharIo {
    pub fn new(tx: &'static RefCell<TxBuffer>) -> Self {
        Self { tx }
    }
}

impl CharIo for UsbCharIo {
    type Error = ();

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Not used — input fed directly via process_char_async() in shell task
        Ok(None)
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        self.tx.borrow_mut().0.push(c as u8).ok();
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let mut tx = self.tx.borrow_mut();
        for b in s.bytes() {
            tx.0.push(b).ok();
        }
        Ok(())
    }
}
