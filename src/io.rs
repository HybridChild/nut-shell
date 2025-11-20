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

#[cfg(test)]
mod tests {
    // Placeholder for tests (will be expanded in Phase 2)
}
