//! Command Context structures.
//!
//! Provides the traits and structure for information to be passed from the command to the handler.

#[cfg(feature = "context-path")]
use core::fmt::Write;
#[cfg(feature = "context-path")]
use heapless::String;

/// Default implementation of Command Context
/// This represents the information passed from the command node back to the handler
#[derive(Debug)]
pub struct CommandContext<'a> {
    /// Unique identifier for handler dispatch
    /// (If the path parameter is not used then this must be unique across entire tree).
    /// Convention: use path-like IDs (e.g., "system_reboot", "network_reboot").
    pub id: &'static str,

    /// Arguments passed to the command
    pub args: &'a [&'a str],

    /// Get the directory path hfof the command
    #[cfg(feature = "context-path")]
    pub path: &'a [&'a str],
}

impl<'a> CommandContext<'a> {
    /// Constructor for the command context
    pub const fn new(
        id: &'static str,
        args: &'a [&'a str],
        #[cfg(feature = "context-path")] path: &'a [&'a str],
    ) -> Self {
        Self {
            id,
            args,
            #[cfg(feature = "context-path")]
            path: path,
        }
    }

    /// Get the path in the form of a string with a path seperator
    #[cfg(feature = "context-path")]
    pub fn get_path_withsep(&self) -> Result<String<128>, core::fmt::Error> {
        let mut result = String::<128>::new();
        if self.path.is_empty() {
            write!(result, "/")?;
        } else {
            for part in self.path {
                write!(result, "/{}", part)?;
            }
        }
        Ok(result)
    }
}
