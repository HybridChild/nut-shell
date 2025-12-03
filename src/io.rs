//! Character I/O abstraction for platform-agnostic input/output.
//!
//! The `CharIo` trait provides non-blocking character-level I/O operations.

/// Platform-agnostic character I/O trait.
/// Implementations provide non-blocking character I/O with platform-specific buffering.
pub trait CharIo {
    /// Platform-specific error type
    type Error;

    /// Read character if available (non-blocking).
    ///
    /// Returns `Ok(Some(char))` if available, `Ok(None)` otherwise.
    fn get_char(&mut self) -> Result<Option<char>, Self::Error>;

    /// Write character to output buffer (must not block indefinitely).
    fn put_char(&mut self, c: char) -> Result<(), Self::Error>;

    /// Write string to output buffer.
    ///
    /// Default calls `put_char()` per character. Override for efficiency if needed.
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.chars() {
            self.put_char(c)?;
        }
        Ok(())
    }
}
