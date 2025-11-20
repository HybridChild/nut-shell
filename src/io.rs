//! Character I/O abstraction for platform-agnostic input/output.
//!
//! The `CharIo` trait provides non-blocking character-level I/O operations that
//! can be implemented for any platform (UART, USB CDC, stdio, etc.).
//!
//! See [docs/IO_DESIGN.md](../docs/IO_DESIGN.md) for complete design and buffering requirements.

/// Platform-agnostic character I/O trait.
///
/// Implementations must buffer output internally. See IO_DESIGN.md for buffering requirements:
/// - Async platforms: Buffer to memory, flush externally after `process_char()`
/// - Bare-metal: May flush immediately (blocking acceptable)
/// - `put_char()` and `write_str()` MUST NOT block indefinitely
pub trait CharIo {
    /// Platform-specific error type
    type Error;

    /// Non-blocking character read.
    ///
    /// Returns:
    /// - `Ok(Some(char))` if character available
    /// - `Ok(None)` if no character available (non-blocking)
    /// - `Err(Self::Error)` on I/O error
    fn get_char(&mut self) -> Result<Option<char>, Self::Error>;

    /// Write character to output buffer.
    ///
    /// IMPORTANT: Must buffer internally. Do not block indefinitely.
    /// See IO_DESIGN.md for buffering requirements.
    fn put_char(&mut self, c: char) -> Result<(), Self::Error>;

    /// Write string to output buffer.
    ///
    /// Default implementation uses `put_char()` repeatedly.
    /// Override for more efficient bulk writes if needed.
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.chars() {
            self.put_char(c)?;
        }
        Ok(())
    }
}

/// Standard I/O implementation for testing.
///
/// Uses bare-metal pattern: immediate flush on each `put_char()`.
/// Only available with `std` feature for testing purposes.
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct StdioStream {
    /// Output buffer (for testing/inspection)
    output: std::sync::Arc<std::sync::Mutex<Vec<char>>>,
}

#[cfg(feature = "std")]
impl StdioStream {
    /// Create a new stdio stream with shared output buffer.
    pub fn new() -> Self {
        Self {
            output: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Returns a clone of the Arc pointing to the output buffer.
    ///
    /// This is a cheap operation (increments reference count) that allows
    /// multiple owners to share access to the same output buffer.
    pub fn output(&self) -> std::sync::Arc<std::sync::Mutex<Vec<char>>> {
        self.output.clone()
    }

    /// Get the output as a string (for testing).
    pub fn output_string(&self) -> String {
        self.output
            .lock()
            .unwrap()
            .iter()
            .collect()
    }

    /// Clear the output buffer.
    pub fn clear_output(&mut self) {
        self.output.lock().unwrap().clear();
    }
}

#[cfg(feature = "std")]
impl Default for StdioStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl CharIo for StdioStream {
    type Error = std::io::Error;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        // Non-blocking read not supported in basic stdio
        // Return None (no character available)
        Ok(None)
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        // Bare-metal pattern: immediate flush (write to buffer)
        self.output.lock().unwrap().push(c);
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn test_write_str_default_impl() {
        let mut io = StdioStream::new();

        io.write_str("Hello").unwrap();
        assert_eq!(io.output_string(), "Hello");

        io.write_str(" World").unwrap();
        assert_eq!(io.output_string(), "Hello World");
    }

    #[test]
    fn test_put_char() {
        let mut io = StdioStream::new();

        io.put_char('A').unwrap();
        io.put_char('B').unwrap();
        io.put_char('C').unwrap();

        assert_eq!(io.output_string(), "ABC");
    }

    #[test]
    fn test_clear_output() {
        let mut io = StdioStream::new();

        io.write_str("Test").unwrap();
        assert_eq!(io.output_string(), "Test");

        io.clear_output();
        assert_eq!(io.output_string(), "");
    }

    #[test]
    fn test_get_char_returns_none() {
        let mut io = StdioStream::new();

        // StdioStream doesn't support reading, should return None
        assert_eq!(io.get_char().unwrap(), None);
    }

    #[test]
    fn test_unicode_support() {
        let mut io = StdioStream::new();

        io.write_str("Hello 世界 🦀").unwrap();
        assert_eq!(io.output_string(), "Hello 世界 🦀");
    }

    #[test]
    fn test_special_characters() {
        let mut io = StdioStream::new();

        io.put_char('\n').unwrap();
        io.put_char('\r').unwrap();
        io.put_char('\t').unwrap();
        io.put_char('\x1b').unwrap(); // ESC

        assert_eq!(io.output_string(), "\n\r\t\x1b");
    }
}
