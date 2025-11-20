//! Input parser for escape sequences and special keys.
//!
//! Provides state machine for parsing ANSI escape sequences (arrow keys, etc.)
//! and handling double-ESC clear functionality.
//!
//! See [INTERNALS.md](../../docs/INTERNALS.md) for state machine details.

use crate::error::CliError;

/// Parser state for escape sequence handling.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParserState {
    /// Normal input mode
    Normal,

    /// Saw first ESC character
    EscapeStart,

    /// Saw ESC [ (start of escape sequence)
    EscapeSequence,
}

/// Parse event result.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseEvent {
    /// No event (accumulating sequence)
    None,

    /// Regular character to add to buffer
    Character(char),

    /// Backspace key
    Backspace,

    /// Enter key (submit command)
    Enter,

    /// Tab key (completion)
    Tab,

    /// Up arrow key (history previous)
    UpArrow,

    /// Down arrow key (history next)
    DownArrow,

    /// Double ESC pressed (clear buffer)
    ClearAndRedraw,
}

/// Terminal input parser with escape sequence handling.
///
/// Processes characters one at a time, maintaining state for multi-character
/// escape sequences (arrow keys, etc.). Supports double-ESC buffer clearing.
///
/// # Example
///
/// ```rust,ignore
/// use nut_shell::shell::parser::{InputParser, ParseEvent};
///
/// let mut parser = InputParser::new();
/// let mut buffer = heapless::String::<128>::new();
///
/// // Process regular character
/// let event = parser.process_char('a', &mut buffer).unwrap();
/// assert_eq!(event, ParseEvent::Character('a'));
/// assert_eq!(buffer.as_str(), "a");
///
/// // Process backspace
/// let event = parser.process_char('\x7f', &mut buffer).unwrap();
/// assert_eq!(event, ParseEvent::Backspace);
/// assert_eq!(buffer.as_str(), "");
/// ```
#[derive(Debug)]
pub struct InputParser {
    /// Current parser state
    state: ParserState,
}

impl InputParser {
    /// Create new parser in Normal state.
    pub fn new() -> Self {
        Self {
            state: ParserState::Normal,
        }
    }

    /// Process single character through state machine.
    ///
    /// Updates buffer based on event type and returns the parse event.
    /// Generic over buffer size (N) to support any configuration.
    ///
    /// # Arguments
    ///
    /// * `c` - Character to process
    /// * `buffer` - Current input buffer to update
    ///
    /// # Returns
    ///
    /// * `Ok(ParseEvent)` - Event indicating what happened
    /// * `Err(CliError::BufferFull)` - Buffer capacity exceeded
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Normal character
    /// let event = parser.process_char('h', &mut buffer)?;
    /// assert_eq!(event, ParseEvent::Character('h'));
    ///
    /// // Backspace
    /// let event = parser.process_char('\x7f', &mut buffer)?;
    /// assert_eq!(event, ParseEvent::Backspace);
    ///
    /// // Up arrow (ESC [ A)
    /// parser.process_char('\x1b', &mut buffer)?;
    /// parser.process_char('[', &mut buffer)?;
    /// let event = parser.process_char('A', &mut buffer)?;
    /// assert_eq!(event, ParseEvent::UpArrow);
    /// ```
    pub fn process_char<const N: usize>(
        &mut self,
        c: char,
        buffer: &mut heapless::String<N>,
    ) -> Result<ParseEvent, CliError> {
        match self.state {
            ParserState::Normal => self.process_normal(c, buffer),
            ParserState::EscapeStart => self.process_escape_start(c, buffer),
            ParserState::EscapeSequence => self.process_escape_sequence(c),
        }
    }

    /// Process character in Normal state.
    fn process_normal<const N: usize>(
        &mut self,
        c: char,
        buffer: &mut heapless::String<N>,
    ) -> Result<ParseEvent, CliError> {
        match c {
            // ESC - start of escape sequence
            '\x1b' => {
                self.state = ParserState::EscapeStart;
                Ok(ParseEvent::None)
            }

            // Enter - line feed or carriage return
            '\n' | '\r' => Ok(ParseEvent::Enter),

            // Tab
            '\t' => Ok(ParseEvent::Tab),

            // Backspace - ASCII BS (0x08) or DEL (0x7F)
            '\x08' | '\x7f' => {
                if !buffer.is_empty() {
                    buffer.pop();
                }
                Ok(ParseEvent::Backspace)
            }

            // Control characters (except those handled above) - ignore
            c if c.is_control() => Ok(ParseEvent::None),

            // Regular printable character
            _ => {
                buffer
                    .push(c)
                    .map_err(|_| CliError::BufferFull)?;
                Ok(ParseEvent::Character(c))
            }
        }
    }

    /// Process character after seeing ESC.
    fn process_escape_start<const N: usize>(
        &mut self,
        c: char,
        buffer: &mut heapless::String<N>,
    ) -> Result<ParseEvent, CliError> {
        match c {
            // Second ESC = double-ESC clear
            '\x1b' => {
                self.state = ParserState::Normal;
                buffer.clear();
                Ok(ParseEvent::ClearAndRedraw)
            }

            // '[' - start of escape sequence (arrow keys, etc.)
            '[' => {
                self.state = ParserState::EscapeSequence;
                Ok(ParseEvent::None)
            }

            // Any other character after ESC - treat as regular character
            // This handles ESC followed by non-sequence characters
            _ => {
                self.state = ParserState::Normal;
                // Process the character normally
                self.process_normal(c, buffer)
            }
        }
    }

    /// Process character in escape sequence (after ESC [).
    fn process_escape_sequence(&mut self, c: char) -> Result<ParseEvent, CliError> {
        // Return to normal state
        self.state = ParserState::Normal;

        match c {
            // Arrow keys
            'A' => Ok(ParseEvent::UpArrow),
            'B' => Ok(ParseEvent::DownArrow),

            // Future: could add C (right arrow), D (left arrow), H (home), F (end)
            // For Phase 6, only up/down arrows are implemented
            // See PHILOSOPHY.md "Recommended Additions"

            // Unknown sequence - ignore
            _ => Ok(ParseEvent::None),
        }
    }

    /// Reset parser state to Normal.
    ///
    /// Useful after handling special events or errors.
    pub fn reset(&mut self) {
        self.state = ParserState::Normal;
    }

    /// Get current parser state (for testing/debugging).
    #[cfg(test)]
    pub fn state(&self) -> ParserState {
        self.state
    }
}

impl Default for InputParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Basic Parser State Tests
    // ========================================

    #[test]
    fn test_parser_state() {
        assert_eq!(ParserState::Normal, ParserState::Normal);
        assert_ne!(ParserState::Normal, ParserState::EscapeStart);
    }

    #[test]
    fn test_parse_event() {
        assert_eq!(ParseEvent::Enter, ParseEvent::Enter);
        assert_ne!(ParseEvent::Tab, ParseEvent::Enter);
    }

    #[test]
    fn test_parser_new() {
        let parser = InputParser::new();
        assert_eq!(parser.state(), ParserState::Normal);
    }

    #[test]
    fn test_parser_default() {
        let parser = InputParser::default();
        assert_eq!(parser.state(), ParserState::Normal);
    }

    #[test]
    fn test_parser_reset() {
        let mut parser = InputParser::new();
        parser.state = ParserState::EscapeStart;
        parser.reset();
        assert_eq!(parser.state(), ParserState::Normal);
    }

    // ========================================
    // Regular Character Processing
    // ========================================

    #[test]
    fn test_regular_characters() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        let event = parser.process_char('h', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::Character('h'));
        assert_eq!(buffer.as_str(), "h");

        let event = parser.process_char('i', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::Character('i'));
        assert_eq!(buffer.as_str(), "hi");
    }

    #[test]
    fn test_unicode_characters() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        let event = parser.process_char('ø', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::Character('ø'));
        assert_eq!(buffer.as_str(), "ø");

        let event = parser.process_char('£', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::Character('£'));
        assert_eq!(buffer.as_str(), "ø£");
    }

    #[test]
    fn test_spaces() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        parser.process_char('h', &mut buffer).unwrap();
        let event = parser.process_char(' ', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::Character(' '));
        parser.process_char('w', &mut buffer).unwrap();

        assert_eq!(buffer.as_str(), "h w");
    }

    // ========================================
    // Special Key Tests
    // ========================================

    #[test]
    fn test_enter_linefeed() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        let event = parser.process_char('\n', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::Enter);
        assert_eq!(buffer.as_str(), "");
    }

    #[test]
    fn test_enter_carriage_return() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        let event = parser.process_char('\r', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::Enter);
        assert_eq!(buffer.as_str(), "");
    }

    #[test]
    fn test_tab() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        let event = parser.process_char('\t', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::Tab);
        assert_eq!(buffer.as_str(), "");
    }

    // ========================================
    // Backspace Tests
    // ========================================

    #[test]
    fn test_backspace_ascii_bs() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        buffer.push_str("hello").unwrap();
        let event = parser.process_char('\x08', &mut buffer).unwrap();

        assert_eq!(event, ParseEvent::Backspace);
        assert_eq!(buffer.as_str(), "hell");
    }

    #[test]
    fn test_backspace_del() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        buffer.push_str("hello").unwrap();
        let event = parser.process_char('\x7f', &mut buffer).unwrap();

        assert_eq!(event, ParseEvent::Backspace);
        assert_eq!(buffer.as_str(), "hell");
    }

    #[test]
    fn test_backspace_on_empty_buffer() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        let event = parser.process_char('\x7f', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::Backspace);
        assert_eq!(buffer.as_str(), "");
    }

    #[test]
    fn test_backspace_multiple() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        buffer.push_str("hello").unwrap();

        parser.process_char('\x7f', &mut buffer).unwrap();
        parser.process_char('\x7f', &mut buffer).unwrap();
        parser.process_char('\x7f', &mut buffer).unwrap();

        assert_eq!(buffer.as_str(), "he");
    }

    // ========================================
    // Escape Sequence Tests
    // ========================================

    #[test]
    fn test_single_esc_no_sequence() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        // ESC should transition to EscapeStart
        let event = parser.process_char('\x1b', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::None);
        assert_eq!(parser.state(), ParserState::EscapeStart);
        assert_eq!(buffer.as_str(), "");
    }

    #[test]
    fn test_double_esc_clears_buffer() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        buffer.push_str("some input").unwrap();

        // First ESC
        let event = parser.process_char('\x1b', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::None);
        assert_eq!(buffer.as_str(), "some input"); // Not cleared yet

        // Second ESC - should clear
        let event = parser.process_char('\x1b', &mut buffer).unwrap();
        assert_eq!(event, ParseEvent::ClearAndRedraw);
        assert_eq!(buffer.as_str(), ""); // Cleared!
        assert_eq!(parser.state(), ParserState::Normal);
    }

    #[test]
    fn test_esc_bracket_starts_sequence() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        // ESC [
        parser.process_char('\x1b', &mut buffer).unwrap();
        let event = parser.process_char('[', &mut buffer).unwrap();

        assert_eq!(event, ParseEvent::None);
        assert_eq!(parser.state(), ParserState::EscapeSequence);
        assert_eq!(buffer.as_str(), ""); // Not added to buffer
    }

    #[test]
    fn test_up_arrow() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        // ESC [ A
        parser.process_char('\x1b', &mut buffer).unwrap();
        parser.process_char('[', &mut buffer).unwrap();
        let event = parser.process_char('A', &mut buffer).unwrap();

        assert_eq!(event, ParseEvent::UpArrow);
        assert_eq!(parser.state(), ParserState::Normal);
    }

    #[test]
    fn test_down_arrow() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        // ESC [ B
        parser.process_char('\x1b', &mut buffer).unwrap();
        parser.process_char('[', &mut buffer).unwrap();
        let event = parser.process_char('B', &mut buffer).unwrap();

        assert_eq!(event, ParseEvent::DownArrow);
        assert_eq!(parser.state(), ParserState::Normal);
    }

    #[test]
    fn test_unknown_escape_sequence() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        // ESC [ X (unknown)
        parser.process_char('\x1b', &mut buffer).unwrap();
        parser.process_char('[', &mut buffer).unwrap();
        let event = parser.process_char('X', &mut buffer).unwrap();

        assert_eq!(event, ParseEvent::None);
        assert_eq!(parser.state(), ParserState::Normal);
    }

    #[test]
    fn test_esc_followed_by_regular_char() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        // ESC followed by 'a' (not a sequence)
        parser.process_char('\x1b', &mut buffer).unwrap();
        let event = parser.process_char('a', &mut buffer).unwrap();

        assert_eq!(event, ParseEvent::Character('a'));
        assert_eq!(buffer.as_str(), "a");
        assert_eq!(parser.state(), ParserState::Normal);
    }

    // ========================================
    // Buffer Overflow Tests
    // ========================================

    #[test]
    fn test_buffer_overflow() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<8>::new();

        // Fill buffer
        for _ in 0..8 {
            parser.process_char('a', &mut buffer).unwrap();
        }

        // Try to overflow
        let result = parser.process_char('x', &mut buffer);
        assert_eq!(result, Err(CliError::BufferFull));
    }

    // ========================================
    // Control Character Tests
    // ========================================

    #[test]
    fn test_control_characters_ignored() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        // Various control characters (except handled ones)
        for c in ['\x00', '\x01', '\x02', '\x03', '\x04', '\x05', '\x06', '\x07'] {
            let event = parser.process_char(c, &mut buffer).unwrap();
            assert_eq!(event, ParseEvent::None);
        }

        assert_eq!(buffer.as_str(), "");
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_complex_input_sequence() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        // Type "hello"
        parser.process_char('h', &mut buffer).unwrap();
        parser.process_char('e', &mut buffer).unwrap();
        parser.process_char('l', &mut buffer).unwrap();
        parser.process_char('l', &mut buffer).unwrap();
        parser.process_char('o', &mut buffer).unwrap();
        assert_eq!(buffer.as_str(), "hello");

        // Backspace once
        parser.process_char('\x7f', &mut buffer).unwrap();
        assert_eq!(buffer.as_str(), "hell");

        // Add space and more text
        parser.process_char(' ', &mut buffer).unwrap();
        parser.process_char('w', &mut buffer).unwrap();
        parser.process_char('o', &mut buffer).unwrap();
        parser.process_char('r', &mut buffer).unwrap();
        parser.process_char('l', &mut buffer).unwrap();
        parser.process_char('d', &mut buffer).unwrap();
        assert_eq!(buffer.as_str(), "hell world");
    }

    #[test]
    fn test_double_esc_with_content() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        // Type something
        parser.process_char('t', &mut buffer).unwrap();
        parser.process_char('e', &mut buffer).unwrap();
        parser.process_char('s', &mut buffer).unwrap();
        parser.process_char('t', &mut buffer).unwrap();
        assert_eq!(buffer.as_str(), "test");

        // Double ESC to clear
        parser.process_char('\x1b', &mut buffer).unwrap();
        parser.process_char('\x1b', &mut buffer).unwrap();
        assert_eq!(buffer.as_str(), "");

        // Can type again
        parser.process_char('n', &mut buffer).unwrap();
        parser.process_char('e', &mut buffer).unwrap();
        parser.process_char('w', &mut buffer).unwrap();
        assert_eq!(buffer.as_str(), "new");
    }

    #[test]
    fn test_arrow_keys_dont_modify_buffer() {
        let mut parser = InputParser::new();
        let mut buffer = heapless::String::<128>::new();

        buffer.push_str("test").unwrap();

        // Up arrow
        parser.process_char('\x1b', &mut buffer).unwrap();
        parser.process_char('[', &mut buffer).unwrap();
        parser.process_char('A', &mut buffer).unwrap();

        assert_eq!(buffer.as_str(), "test"); // Unchanged

        // Down arrow
        parser.process_char('\x1b', &mut buffer).unwrap();
        parser.process_char('[', &mut buffer).unwrap();
        parser.process_char('B', &mut buffer).unwrap();

        assert_eq!(buffer.as_str(), "test"); // Unchanged
    }
}
