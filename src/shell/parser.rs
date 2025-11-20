//! Input parser for escape sequences and special keys.
//!
//! Provides state machine for parsing ANSI escape sequences (arrow keys, etc.)
//! and handling double-ESC clear functionality.
//!
//! See [INTERNALS.md](../../docs/INTERNALS.md) for state machine details.

// Placeholder - will be implemented in Phase 6

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
