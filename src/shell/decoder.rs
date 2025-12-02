//! Input decoder for terminal character sequences.
//!
//! Provides state machine for interpreting ANSI escape sequences (arrow keys, etc.)
//! and special key combinations (double-ESC clear).
//!
//! This is a pure decoder - it doesn't manage buffers or I/O. It simply converts
//! raw terminal character sequences into logical input events.
//!
//! See [INTERNALS.md](../../docs/INTERNALS.md) for state machine details.

/// Decoder state for escape sequence handling.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InputState {
    /// Normal input mode
    Normal,

    /// Saw first ESC character
    EscapeStart,

    /// Saw ESC [ (start of escape sequence)
    EscapeSequence,
}

/// Logical input event from terminal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InputEvent {
    /// No event (accumulating sequence)
    None,

    /// Regular character typed
    Char(char),

    /// Backspace key (ASCII BS or DEL)
    Backspace,

    /// Enter key (line feed or carriage return)
    Enter,

    /// Tab key
    Tab,

    /// Up arrow key (history previous)
    UpArrow,

    /// Down arrow key (history next)
    DownArrow,

    /// Double ESC pressed
    DoubleEsc,
}

/// Terminal input decoder with escape sequence handling.
///
/// Decodes raw terminal characters into logical input events. Maintains state
/// for multi-character escape sequences (arrow keys, etc.) and supports
/// double-ESC clear functionality.
///
/// Pure state machine - doesn't manage buffers or perform I/O.
#[derive(Debug)]
pub struct InputDecoder {
    /// Current decoder state
    state: InputState,
}

impl InputDecoder {
    /// Create new decoder in Normal state.
    pub fn new() -> Self {
        Self {
            state: InputState::Normal,
        }
    }

    /// Decode single character into input event.
    ///
    /// Updates internal state machine and returns event representing the
    /// logical input action. Does not manage buffers or perform I/O.
    ///
    /// # Arguments
    ///
    /// * `c` - Character to decode
    ///
    /// # Returns
    ///
    /// Event indicating what input occurred
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Normal character
    /// let event = decoder.decode_char('h');
    /// assert_eq!(event, InputEvent::Char('h'));
    ///
    /// // Backspace
    /// let event = decoder.decode_char('\x7f');
    /// assert_eq!(event, InputEvent::Backspace);
    ///
    /// // Up arrow (ESC [ A)
    /// decoder.decode_char('\x1b');
    /// decoder.decode_char('[');
    /// let event = decoder.decode_char('A');
    /// assert_eq!(event, InputEvent::UpArrow);
    ///
    /// // Double ESC
    /// decoder.decode_char('\x1b');  // First ESC
    /// let event = decoder.decode_char('\x1b');  // Second ESC
    /// assert_eq!(event, InputEvent::DoubleEsc);
    /// ```
    pub fn decode_char(&mut self, c: char) -> InputEvent {
        match self.state {
            InputState::Normal => self.decode_normal(c),
            InputState::EscapeStart => self.decode_escape_start(c),
            InputState::EscapeSequence => self.decode_escape_sequence(c),
        }
    }

    /// Decode character in Normal state.
    fn decode_normal(&mut self, c: char) -> InputEvent {
        match c {
            // ESC - start of escape sequence
            '\x1b' => {
                self.state = InputState::EscapeStart;
                InputEvent::None
            }

            // Enter - line feed or carriage return
            '\n' | '\r' => InputEvent::Enter,

            // Tab
            '\t' => InputEvent::Tab,

            // Backspace - ASCII BS (0x08) or DEL (0x7F)
            '\x08' | '\x7f' => InputEvent::Backspace,

            // Control characters (except those handled above) - ignore
            c if c.is_control() => InputEvent::None,

            // Regular printable character
            _ => InputEvent::Char(c),
        }
    }

    /// Decode character after seeing ESC.
    fn decode_escape_start(&mut self, c: char) -> InputEvent {
        match c {
            // Second ESC = double-ESC
            '\x1b' => {
                self.state = InputState::Normal;
                InputEvent::DoubleEsc
            }

            // '[' - start of escape sequence (arrow keys, etc.)
            '[' => {
                self.state = InputState::EscapeSequence;
                InputEvent::None
            }

            // Any other character after ESC - treat as regular character
            // This handles ESC followed by non-sequence characters
            _ => {
                self.state = InputState::Normal;
                InputEvent::Char(c)
            }
        }
    }

    /// Decode character in escape sequence (after ESC [).
    fn decode_escape_sequence(&mut self, c: char) -> InputEvent {
        // Return to normal state
        self.state = InputState::Normal;

        match c {
            // Arrow keys
            'A' => InputEvent::UpArrow,
            'B' => InputEvent::DownArrow,

            // Future: could add C (right arrow), D (left arrow), H (home), F (end)
            // For Phase 6, only up/down arrows are implemented
            // See PHILOSOPHY.md "Recommended Additions"

            // Unknown sequence - ignore
            _ => InputEvent::None,
        }
    }

    /// Reset decoder state to Normal.
    ///
    /// Useful after handling special events or errors.
    pub fn reset(&mut self) {
        self.state = InputState::Normal;
    }

    /// Get current decoder state (for testing/debugging).
    #[cfg(test)]
    pub fn state(&self) -> InputState {
        self.state
    }
}

impl Default for InputDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Basic Decoder State Tests
    // ========================================

    #[test]
    fn test_decoder_new() {
        let decoder = InputDecoder::new();
        assert_eq!(decoder.state(), InputState::Normal);
    }

    #[test]
    fn test_decoder_default() {
        let decoder = InputDecoder::default();
        assert_eq!(decoder.state(), InputState::Normal);
    }

    #[test]
    fn test_decoder_reset() {
        let mut decoder = InputDecoder::new();
        decoder.state = InputState::EscapeStart;
        decoder.reset();
        assert_eq!(decoder.state(), InputState::Normal);
    }

    // ========================================
    // Regular Character Decoding
    // ========================================

    #[test]
    fn test_regular_characters() {
        let mut decoder = InputDecoder::new();

        let event = decoder.decode_char('h');
        assert_eq!(event, InputEvent::Char('h'));

        let event = decoder.decode_char('i');
        assert_eq!(event, InputEvent::Char('i'));
    }

    #[test]
    fn test_unicode_characters() {
        let mut decoder = InputDecoder::new();

        let event = decoder.decode_char('ø');
        assert_eq!(event, InputEvent::Char('ø'));

        let event = decoder.decode_char('£');
        assert_eq!(event, InputEvent::Char('£'));
    }

    #[test]
    fn test_spaces() {
        let mut decoder = InputDecoder::new();

        let event = decoder.decode_char(' ');
        assert_eq!(event, InputEvent::Char(' '));
    }

    // ========================================
    // Special Key Tests
    // ========================================

    #[test]
    fn test_enter_linefeed() {
        let mut decoder = InputDecoder::new();

        let event = decoder.decode_char('\n');
        assert_eq!(event, InputEvent::Enter);
    }

    #[test]
    fn test_enter_carriage_return() {
        let mut decoder = InputDecoder::new();

        let event = decoder.decode_char('\r');
        assert_eq!(event, InputEvent::Enter);
    }

    #[test]
    fn test_tab() {
        let mut decoder = InputDecoder::new();

        let event = decoder.decode_char('\t');
        assert_eq!(event, InputEvent::Tab);
    }

    // ========================================
    // Backspace Tests
    // ========================================

    #[test]
    fn test_backspace_ascii_bs() {
        let mut decoder = InputDecoder::new();

        let event = decoder.decode_char('\x08');
        assert_eq!(event, InputEvent::Backspace);
    }

    #[test]
    fn test_backspace_del() {
        let mut decoder = InputDecoder::new();

        let event = decoder.decode_char('\x7f');
        assert_eq!(event, InputEvent::Backspace);
    }

    // ========================================
    // Escape Sequence Tests
    // ========================================

    #[test]
    fn test_single_esc_no_sequence() {
        let mut decoder = InputDecoder::new();

        // ESC should transition to EscapeStart
        let event = decoder.decode_char('\x1b');
        assert_eq!(event, InputEvent::None);
        assert_eq!(decoder.state(), InputState::EscapeStart);
    }

    #[test]
    fn test_double_esc() {
        let mut decoder = InputDecoder::new();

        // First ESC
        let event = decoder.decode_char('\x1b');
        assert_eq!(event, InputEvent::None);

        // Second ESC
        let event = decoder.decode_char('\x1b');
        assert_eq!(event, InputEvent::DoubleEsc);
        assert_eq!(decoder.state(), InputState::Normal);
    }

    #[test]
    fn test_esc_bracket_starts_sequence() {
        let mut decoder = InputDecoder::new();

        // ESC [
        decoder.decode_char('\x1b');
        let event = decoder.decode_char('[');

        assert_eq!(event, InputEvent::None);
        assert_eq!(decoder.state(), InputState::EscapeSequence);
    }

    #[test]
    fn test_up_arrow() {
        let mut decoder = InputDecoder::new();

        // ESC [ A
        decoder.decode_char('\x1b');
        decoder.decode_char('[');
        let event = decoder.decode_char('A');

        assert_eq!(event, InputEvent::UpArrow);
        assert_eq!(decoder.state(), InputState::Normal);
    }

    #[test]
    fn test_down_arrow() {
        let mut decoder = InputDecoder::new();

        // ESC [ B
        decoder.decode_char('\x1b');
        decoder.decode_char('[');
        let event = decoder.decode_char('B');

        assert_eq!(event, InputEvent::DownArrow);
        assert_eq!(decoder.state(), InputState::Normal);
    }

    #[test]
    fn test_unknown_escape_sequence() {
        let mut decoder = InputDecoder::new();

        // ESC [ X (unknown)
        decoder.decode_char('\x1b');
        decoder.decode_char('[');
        let event = decoder.decode_char('X');

        assert_eq!(event, InputEvent::None);
        assert_eq!(decoder.state(), InputState::Normal);
    }

    #[test]
    fn test_esc_followed_by_regular_char() {
        let mut decoder = InputDecoder::new();

        // ESC followed by 'a' (not a sequence)
        decoder.decode_char('\x1b');
        let event = decoder.decode_char('a');

        assert_eq!(event, InputEvent::Char('a'));
        assert_eq!(decoder.state(), InputState::Normal);
    }

    // ========================================
    // Control Character Tests
    // ========================================

    #[test]
    fn test_control_characters_ignored() {
        let mut decoder = InputDecoder::new();

        // Various control characters (except handled ones)
        for c in [
            '\x00', '\x01', '\x02', '\x03', '\x04', '\x05', '\x06', '\x07',
        ] {
            let event = decoder.decode_char(c);
            assert_eq!(event, InputEvent::None);
        }
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_complex_input_sequence() {
        let mut decoder = InputDecoder::new();

        // Type "hello"
        assert_eq!(decoder.decode_char('h'), InputEvent::Char('h'));
        assert_eq!(decoder.decode_char('e'), InputEvent::Char('e'));
        assert_eq!(decoder.decode_char('l'), InputEvent::Char('l'));
        assert_eq!(decoder.decode_char('l'), InputEvent::Char('l'));
        assert_eq!(decoder.decode_char('o'), InputEvent::Char('o'));

        // Backspace
        assert_eq!(decoder.decode_char('\x7f'), InputEvent::Backspace);

        // Add space and more text
        assert_eq!(decoder.decode_char(' '), InputEvent::Char(' '));
        assert_eq!(decoder.decode_char('w'), InputEvent::Char('w'));
        assert_eq!(decoder.decode_char('o'), InputEvent::Char('o'));
        assert_eq!(decoder.decode_char('r'), InputEvent::Char('r'));
        assert_eq!(decoder.decode_char('l'), InputEvent::Char('l'));
        assert_eq!(decoder.decode_char('d'), InputEvent::Char('d'));
    }

    #[test]
    fn test_double_esc_then_type() {
        let mut decoder = InputDecoder::new();

        // Double ESC
        decoder.decode_char('\x1b');
        assert_eq!(decoder.decode_char('\x1b'), InputEvent::DoubleEsc);

        // Can type again after clear
        assert_eq!(decoder.decode_char('n'), InputEvent::Char('n'));
        assert_eq!(decoder.decode_char('e'), InputEvent::Char('e'));
        assert_eq!(decoder.decode_char('w'), InputEvent::Char('w'));
    }

    #[test]
    fn test_arrow_keys_sequence() {
        let mut decoder = InputDecoder::new();

        // Up arrow
        decoder.decode_char('\x1b');
        decoder.decode_char('[');
        assert_eq!(decoder.decode_char('A'), InputEvent::UpArrow);

        // Down arrow
        decoder.decode_char('\x1b');
        decoder.decode_char('[');
        assert_eq!(decoder.decode_char('B'), InputEvent::DownArrow);
    }
}
