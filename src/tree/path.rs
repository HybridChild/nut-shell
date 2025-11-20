//! Path parsing and navigation.
//!
//! Provides Unix-style path resolution with support for absolute and relative paths,
//! parent navigation (..), and current directory (.).
//!
//! # Example
//!
//! ```rust,ignore
//! use nut_shell::tree::path::Path;
//!
//! // Absolute path
//! let path = Path::parse("/system/reboot")?;
//! assert!(path.is_absolute());
//! assert_eq!(path.segments().collect::<Vec<_>>(), vec!["system", "reboot"]);
//!
//! // Relative path with parent navigation
//! let path = Path::parse("../network/status")?;
//! assert!(!path.is_absolute());
//! assert_eq!(path.segments().collect::<Vec<_>>(), vec!["..", "network", "status"]);
//! ```

use crate::error::CliError;

/// Maximum path depth (directory nesting)
///
/// This matches the default MAX_PATH_DEPTH from ShellConfig.
/// Limited to 8 levels to conserve stack space on embedded systems.
pub const MAX_PATH_DEPTH: usize = 8;

/// Unix-style path parser and representation.
///
/// Handles absolute and relative paths with `.` and `..` navigation.
/// Zero-allocation parsing using string slices.
///
/// # Path Syntax
///
/// - **Absolute paths**: Start with `/` (e.g., `/system/reboot`)
/// - **Relative paths**: No leading `/` (e.g., `network/status`, `../hw`)
/// - **Parent navigation**: `..` goes up one level
/// - **Current directory**: `.` stays at current level
///
/// # Memory
///
/// Uses MAX_PATH_DEPTH (8) to limit nesting depth.
/// All parsing is zero-allocation, working with string slices.
#[derive(Debug, PartialEq)]
pub struct Path<'a> {
    /// Original path string
    _original: &'a str,

    /// Whether this is an absolute path (starts with `/`)
    is_absolute: bool,

    /// Path segments (directories/commands)
    /// Includes `.` and `..` for processing during resolution
    segments: heapless::Vec<&'a str, MAX_PATH_DEPTH>,
}

impl<'a> Path<'a> {
    /// Parse path string into Path structure.
    ///
    /// # Supported Syntax
    ///
    /// - Absolute paths: `/system/reboot`
    /// - Relative paths: `../network/status`, `./cmd`, `cmd`
    /// - Parent navigation: `..` (go up one level)
    /// - Current directory: `.` (stay at current level)
    ///
    /// # Returns
    ///
    /// - `Ok(Path)` - Successfully parsed
    /// - `Err(CliError::InvalidPath)` - Empty path or invalid syntax
    /// - `Err(CliError::PathTooDeep)` - Exceeds MAX_PATH_DEPTH
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let path = Path::parse("/system/reboot")?;
    /// assert!(path.is_absolute());
    ///
    /// let path = Path::parse("../network")?;
    /// assert!(!path.is_absolute());
    /// ```
    pub fn parse(input: &'a str) -> Result<Self, CliError> {
        // Handle empty path
        if input.is_empty() {
            return Err(CliError::InvalidPath);
        }

        // Check if absolute (starts with /)
        let is_absolute = input.starts_with('/');

        // Remove leading slash for parsing
        let path_str = if is_absolute {
            &input[1..]
        } else {
            input
        };

        // Parse segments
        let mut segments = heapless::Vec::new();

        // Empty path after removing leading slash means root directory
        if path_str.is_empty() {
            // Absolute path "/" refers to root
            if is_absolute {
                return Ok(Self {
                    _original: input,
                    is_absolute,
                    segments,
                });
            } else {
                // Relative empty path is invalid
                return Err(CliError::InvalidPath);
            }
        }

        // Split by '/' and filter empty segments
        for segment in path_str.split('/') {
            // Skip empty segments (e.g., from "//" or trailing "/")
            if segment.is_empty() {
                continue;
            }

            // Add segment
            segments
                .push(segment)
                .map_err(|_| CliError::PathTooDeep)?;
        }

        Ok(Self {
            _original: input,
            is_absolute,
            segments,
        })
    }

    /// Check if this is an absolute path (starts with `/`).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// assert!(Path::parse("/system")?.is_absolute());
    /// assert!(!Path::parse("network")?.is_absolute());
    /// ```
    pub fn is_absolute(&self) -> bool {
        self.is_absolute
    }

    /// Get path segments as slice.
    ///
    /// Returns borrowed slice of segment strings. Segments include `.` and `..`
    /// which are processed during path resolution.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let path = Path::parse("/system/network")?;
    /// assert_eq!(path.segments(), &["system", "network"]);
    ///
    /// let path = Path::parse("../hw/led")?;
    /// assert_eq!(path.segments(), &["..", "hw", "led"]);
    /// ```
    pub fn segments(&self) -> &[&'a str] {
        &self.segments
    }

    /// Get number of segments in path.
    ///
    /// Useful for checking path depth before resolution.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestPath<'a> = Path<'a>;

    #[test]
    fn test_empty_path_is_invalid() {
        let result = TestPath::parse("");
        assert_eq!(result, Err(CliError::InvalidPath));
    }

    #[test]
    fn test_absolute_root() {
        let path = TestPath::parse("/").unwrap();
        assert!(path.is_absolute());
        assert_eq!(path.segments(), &[] as &[&str]);
        assert_eq!(path.segment_count(), 0);
    }

    #[test]
    fn test_absolute_single_segment() {
        let path = TestPath::parse("/system").unwrap();
        assert!(path.is_absolute());
        assert_eq!(path.segments(), &["system"]);
        assert_eq!(path.segment_count(), 1);
    }

    #[test]
    fn test_absolute_multiple_segments() {
        let path = TestPath::parse("/system/network/status").unwrap();
        assert!(path.is_absolute());
        assert_eq!(path.segments(), &["system", "network", "status"]);
        assert_eq!(path.segment_count(), 3);
    }

    #[test]
    fn test_relative_single_segment() {
        let path = TestPath::parse("help").unwrap();
        assert!(!path.is_absolute());
        assert_eq!(path.segments(), &["help"]);
    }

    #[test]
    fn test_relative_multiple_segments() {
        let path = TestPath::parse("system/network").unwrap();
        assert!(!path.is_absolute());
        assert_eq!(path.segments(), &["system", "network"]);
    }

    #[test]
    fn test_parent_navigation() {
        let path = TestPath::parse("..").unwrap();
        assert!(!path.is_absolute());
        assert_eq!(path.segments(), &[".."]);

        let path = TestPath::parse("../system").unwrap();
        assert_eq!(path.segments(), &["..", "system"]);

        let path = TestPath::parse("../../hw/led").unwrap();
        assert_eq!(path.segments(), &["..", "..", "hw", "led"]);
    }

    #[test]
    fn test_current_directory() {
        let path = TestPath::parse(".").unwrap();
        assert!(!path.is_absolute());
        assert_eq!(path.segments(), &["."]);

        let path = TestPath::parse("./cmd").unwrap();
        assert_eq!(path.segments(), &[".", "cmd"]);
    }

    #[test]
    fn test_mixed_navigation() {
        let path = TestPath::parse("../system/./network").unwrap();
        assert_eq!(path.segments(), &["..", "system", ".", "network"]);
    }

    #[test]
    fn test_trailing_slash_ignored() {
        let path = TestPath::parse("/system/").unwrap();
        assert!(path.is_absolute());
        assert_eq!(path.segments(), &["system"]);

        let path = TestPath::parse("network/").unwrap();
        assert_eq!(path.segments(), &["network"]);
    }

    #[test]
    fn test_double_slash_treated_as_single() {
        let path = TestPath::parse("/system//network").unwrap();
        assert_eq!(path.segments(), &["system", "network"]);

        let path = TestPath::parse("//system").unwrap();
        assert_eq!(path.segments(), &["system"]);
    }

    #[test]
    fn test_path_too_deep() {
        // Build a path that exceeds MAX_PATH_DEPTH (8 for DefaultConfig)
        let deep_path = "a/b/c/d/e/f/g/h/i/j/k";
        let result = TestPath::parse(deep_path);
        assert_eq!(result, Err(CliError::PathTooDeep));
    }

    #[test]
    fn test_max_depth_exactly() {
        // MAX_PATH_DEPTH = 8 for DefaultConfig
        let path = TestPath::parse("a/b/c/d/e/f/g/h").unwrap();
        assert_eq!(path.segment_count(), 8);
    }

    #[test]
    fn test_absolute_path_with_parent() {
        let path = TestPath::parse("/../system").unwrap();
        assert!(path.is_absolute());
        assert_eq!(path.segments(), &["..", "system"]);
    }

    #[test]
    fn test_complex_real_world_paths() {
        // Real navigation scenarios
        let path = TestPath::parse("/system/network/config").unwrap();
        assert!(path.is_absolute());
        assert_eq!(path.segments(), &["system", "network", "config"]);

        let path = TestPath::parse("../../debug/memory").unwrap();
        assert!(!path.is_absolute());
        assert_eq!(path.segments(), &["..", "..", "debug", "memory"]);

        let path = TestPath::parse("./status").unwrap();
        assert!(!path.is_absolute());
        assert_eq!(path.segments(), &[".", "status"]);
    }
}
