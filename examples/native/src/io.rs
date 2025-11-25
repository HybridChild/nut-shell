//! Shared I/O implementation for native examples

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use nut_shell::io::CharIo;
use std::io::{self, Read};

// =============================================================================
// Terminal Raw Mode Guard
// =============================================================================

/// RAII guard that enables raw terminal mode on creation and restores on drop.
///
/// This ensures the terminal is always restored, even on panic or error.
/// Raw mode provides:
/// - No local echo (shell controls all echoing for password masking)
/// - No line buffering (process characters immediately)
/// - No special key processing by terminal (Tab, arrows passed to shell)
pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // Always try to restore terminal mode
        let _ = disable_raw_mode();
    }
}

// =============================================================================
// I/O Implementation
// =============================================================================

/// Standard I/O implementation for native examples using stdin/stdout.
///
/// This implementation uses blocking reads, which is appropriate for
/// examples running in a terminal. For async examples, the async behavior
/// comes from `process_char_async()`, not from the I/O itself.
pub struct StdioCharIo {
    stdin: io::Stdin,
}

impl StdioCharIo {
    pub fn new() -> Self {
        Self { stdin: io::stdin() }
    }
}

impl Default for StdioCharIo {
    fn default() -> Self {
        Self::new()
    }
}

impl CharIo for StdioCharIo {
    type Error = io::Error;

    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        let mut buf = [0u8; 1];
        let mut handle = self.stdin.lock();

        // Non-blocking read would require platform-specific code
        // For these examples, we use blocking reads
        match handle.read(&mut buf) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                // Simple ASCII to char conversion
                // For a production CLI, you'd want proper UTF-8 handling
                Ok(Some(buf[0] as char))
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        print!("{}", c);
        use std::io::Write;
        std::io::stdout().flush()?;
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        print!("{}", s);
        use std::io::Write;
        std::io::stdout().flush()?;
        Ok(())
    }
}
