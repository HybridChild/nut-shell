//! Command history with up/down arrow navigation.
//!
//! Uses stub type pattern - struct always exists, but behavior is feature-gated.

#![cfg_attr(not(feature = "history"), allow(unused_variables))]

#[cfg(not(feature = "history"))]
use core::marker::PhantomData;

/// Command history storage.
///
/// When `history` feature is enabled, stores commands in a ring buffer.
/// When disabled, zero-size stub that no-ops all operations.
#[derive(Debug)]
pub struct CommandHistory<const N: usize, const INPUT_SIZE: usize> {
    #[cfg(feature = "history")]
    buffer: heapless::Vec<heapless::String<INPUT_SIZE>, N>,

    #[cfg(feature = "history")]
    position: Option<usize>,

    #[cfg(not(feature = "history"))]
    _phantom: PhantomData<[u8; INPUT_SIZE]>,
}

impl<const N: usize, const INPUT_SIZE: usize> CommandHistory<N, INPUT_SIZE> {
    /// Create new command history.
    #[cfg(feature = "history")]
    pub fn new() -> Self {
        Self {
            buffer: heapless::Vec::new(),
            position: None,
        }
    }

    /// Create new command history (stub version).
    #[cfg(not(feature = "history"))]
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Add command to history.
    #[cfg(feature = "history")]
    pub fn add(&mut self, cmd: &str) {
        // Don't add empty commands or duplicates
        if cmd.is_empty() {
            return;
        }

        // Don't add if same as most recent
        if let Some(last) = self.buffer.last()
            && last.as_str() == cmd
        {
            return;
        }

        let mut entry = heapless::String::new();
        if entry.push_str(cmd).is_ok() {
            // Ring buffer behavior - remove oldest if full
            if self.buffer.is_full() {
                self.buffer.remove(0);
            }
            let _ = self.buffer.push(entry);
        }

        // Reset position
        self.position = None;
    }

    /// Add command to history (stub version - no-op).
    #[cfg(not(feature = "history"))]
    pub fn add(&mut self, _cmd: &str) {
        // No-op
    }

    /// Navigate to previous command (up arrow).
    #[cfg(feature = "history")]
    pub fn previous_command(&mut self) -> Option<heapless::String<INPUT_SIZE>> {
        if self.buffer.is_empty() {
            return None;
        }

        let pos = match self.position {
            None => self.buffer.len() - 1,
            Some(0) => 0, // Already at oldest
            Some(p) => p - 1,
        };

        self.position = Some(pos);
        self.buffer.get(pos).cloned()
    }

    /// Navigate to previous command (stub version - returns None).
    #[cfg(not(feature = "history"))]
    pub fn previous_command(&mut self) -> Option<heapless::String<INPUT_SIZE>> {
        None
    }

    /// Navigate to next command (down arrow).
    #[cfg(feature = "history")]
    pub fn next_command(&mut self) -> Option<heapless::String<INPUT_SIZE>> {
        match self.position {
            None => None, // Not navigating
            Some(p) if p >= self.buffer.len() - 1 => {
                // At newest - go to empty
                self.position = None;
                None
            }
            Some(p) => {
                let pos = p + 1;
                self.position = Some(pos);
                self.buffer.get(pos).cloned()
            }
        }
    }

    /// Navigate to next command (stub version - returns None).
    #[cfg(not(feature = "history"))]
    pub fn next_command(&mut self) -> Option<heapless::String<INPUT_SIZE>> {
        None
    }

    /// Reset navigation position.
    #[cfg(feature = "history")]
    pub fn reset_position(&mut self) {
        self.position = None;
    }

    /// Reset navigation position (stub version - no-op).
    #[cfg(not(feature = "history"))]
    pub fn reset_position(&mut self) {
        // No-op
    }
}

impl<const N: usize, const INPUT_SIZE: usize> Default for CommandHistory<N, INPUT_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "history")]
    fn test_add_and_navigate() {
        let mut history = CommandHistory::<5, 128>::new();

        history.add("cmd1");
        history.add("cmd2");
        history.add("cmd3");

        // Navigate backwards
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd3");
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd2");
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd1");

        // At oldest - should stay
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd1");

        // Navigate forward
        assert_eq!(history.next_command().unwrap().as_str(), "cmd2");
        assert_eq!(history.next_command().unwrap().as_str(), "cmd3");

        // At newest - should return None
        assert!(history.next_command().is_none());
    }

    #[test]
    #[cfg(not(feature = "history"))]
    fn test_stub_behavior() {
        let mut history = CommandHistory::<5, 128>::new();

        history.add("cmd1");
        assert!(history.previous_command().is_none());
        assert!(history.next_command().is_none());
    }

    #[test]
    #[cfg(feature = "history")]
    fn test_ring_buffer_behavior() {
        let mut history = CommandHistory::<3, 128>::new();

        // Fill buffer to capacity
        history.add("cmd1");
        history.add("cmd2");
        history.add("cmd3");

        // Add one more - should remove oldest (cmd1)
        history.add("cmd4");

        // Navigate to oldest (should be cmd2 now)
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd4");
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd3");
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd2");

        // cmd1 should be gone
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd2"); // Stay at oldest
    }

    #[test]
    #[cfg(feature = "history")]
    fn test_empty_commands_ignored() {
        let mut history = CommandHistory::<5, 128>::new();

        history.add("");
        history.add("cmd1");
        history.add("");
        history.add("cmd2");

        // Only cmd1 and cmd2 should be in history
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd2");
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd1");
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd1"); // At oldest
    }

    #[test]
    #[cfg(feature = "history")]
    fn test_duplicate_commands_ignored() {
        let mut history = CommandHistory::<5, 128>::new();

        history.add("cmd1");
        history.add("cmd1"); // Duplicate - should be ignored
        history.add("cmd2");
        history.add("cmd2"); // Duplicate - should be ignored
        history.add("cmd1"); // Different from last - should be added

        // Should have: cmd1, cmd2, cmd1
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd1");
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd2");
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd1");
    }

    #[test]
    #[cfg(feature = "history")]
    fn test_navigation_without_adding() {
        let mut history = CommandHistory::<5, 128>::new();

        // Try to navigate when empty
        assert!(history.previous_command().is_none());
        assert!(history.next_command().is_none());
    }

    #[test]
    #[cfg(feature = "history")]
    fn test_reset_position() {
        let mut history = CommandHistory::<5, 128>::new();

        history.add("cmd1");
        history.add("cmd2");
        history.add("cmd3");

        // Navigate backwards
        history.previous_command();
        history.previous_command();

        // Reset position
        history.reset_position();

        // Next previous should return most recent
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd3");
    }

    #[test]
    #[cfg(feature = "history")]
    fn test_position_resets_on_add() {
        let mut history = CommandHistory::<5, 128>::new();

        history.add("cmd1");
        history.add("cmd2");

        // Navigate backwards
        history.previous_command();

        // Add new command - position should reset
        history.add("cmd3");

        // Next previous should return most recent
        assert_eq!(history.previous_command().unwrap().as_str(), "cmd3");
    }

    #[test]
    #[cfg(feature = "history")]
    fn test_default() {
        let history = CommandHistory::<5, 128>::default();
        let mut history2 = history;
        assert!(history2.previous_command().is_none());
    }

    #[test]
    #[cfg(not(feature = "history"))]
    fn test_stub_reset_position() {
        let mut history = CommandHistory::<5, 128>::new();
        history.reset_position(); // Should not panic
    }
}
