//! Buffered UART CharIo implementation for Embassy async example

use core::cell::RefCell;
use heapless;
use nut_shell::io::CharIo;

// =============================================================================
// UART CharIo Implementation (Buffered for Embassy)
// =============================================================================

/// Buffered UART I/O adapter for Embassy.
///
/// Implements the deferred flush pattern described in IO_DESIGN.md:
/// - `put_char()` and `write_str()` buffer to memory only
/// - Output is stored in an internal buffer accessed via RefCell
pub struct BufferedUartCharIo {
    output_buffer: &'static RefCell<heapless::Vec<u8, 512>>,
}

impl BufferedUartCharIo {
    pub fn new(output_buffer: &'static RefCell<heapless::Vec<u8, 512>>) -> Self {
        Self { output_buffer }
    }

    /// Check if buffer has data to flush
    pub fn has_data(&self) -> bool {
        !self.output_buffer.borrow().is_empty()
    }

    /// Get buffered data for flushing
    pub fn take_buffer(&self) -> heapless::Vec<u8, 512> {
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
