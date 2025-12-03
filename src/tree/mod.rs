//! Command tree data structures.
//!
//! Provides the core tree structure for organizing commands and directories.
//! All tree structures are const-initializable and live in ROM.

use crate::auth::AccessLevel;

// Sub-modules
pub mod completion;
pub mod path;

/// Command kind marker (sync or async).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CommandKind {
    /// Synchronous command
    Sync,

    /// Asynchronous command (requires `async` feature)
    #[cfg(feature = "async")]
    Async,
}

/// Command metadata (const-initializable, no execution logic).
/// Execution via `CommandHandler` trait enables sync and async commands with const-initialization.
/// Unique `id` field allows duplicate names across directories.
#[derive(Debug, Clone)]
pub struct CommandMeta<L: AccessLevel> {
    /// Unique identifier for handler dispatch (must be unique across entire tree).
    /// Convention: use path-like IDs (e.g., "system_reboot", "network_reboot").
    pub id: &'static str,

    /// Command name (display name, can duplicate across directories)
    pub name: &'static str,

    /// Command description (shown by ls command)
    pub description: &'static str,

    /// Minimum access level required
    pub access_level: L,

    /// Command kind (sync or async marker)
    pub kind: CommandKind,

    /// Minimum number of arguments
    pub min_args: usize,

    /// Maximum number of arguments
    pub max_args: usize,
}

/// Directory node containing child nodes (const-initializable, stored in ROM).
/// Organizes commands hierarchically.
#[derive(Debug, Clone)]
pub struct Directory<L: AccessLevel> {
    /// Directory name
    pub name: &'static str,

    /// Child nodes (commands and subdirectories)
    pub children: &'static [Node<L>],

    /// Minimum access level required to access this directory
    pub access_level: L,
}

/// Tree node (command or directory).
///
/// Enables zero-cost dispatch via pattern matching instead of vtables.
#[derive(Debug, Clone)]
pub enum Node<L: AccessLevel> {
    /// Command node (metadata only)
    Command(&'static CommandMeta<L>),

    /// Directory node
    Directory(&'static Directory<L>),
}

impl<L: AccessLevel> Node<L> {
    /// Check if this node is a command.
    pub fn is_command(&self) -> bool {
        matches!(self, Node::Command(_))
    }

    /// Check if this node is a directory.
    pub fn is_directory(&self) -> bool {
        matches!(self, Node::Directory(_))
    }

    /// Get node name.
    pub fn name(&self) -> &'static str {
        match self {
            Node::Command(cmd) => cmd.name,
            Node::Directory(dir) => dir.name,
        }
    }

    /// Get node access level.
    pub fn access_level(&self) -> L {
        match self {
            Node::Command(cmd) => cmd.access_level,
            Node::Directory(dir) => dir.access_level,
        }
    }
}

impl<L: AccessLevel> Directory<L> {
    /// Find child node by name (no access control, returns `None` if not found).
    pub fn find_child(&self, name: &str) -> Option<&Node<L>> {
        self.children.iter().find(|child| child.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AccessLevel;

    // Mock access level for testing
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum TestAccessLevel {
        Guest = 0,
        User = 1,
    }

    impl AccessLevel for TestAccessLevel {
        fn from_str(s: &str) -> Option<Self> {
            match s {
                "Guest" => Some(Self::Guest),
                "User" => Some(Self::User),
                _ => None,
            }
        }

        fn as_str(&self) -> &'static str {
            match self {
                Self::Guest => "Guest",
                Self::User => "User",
            }
        }
    }

    #[test]
    fn test_command_kind() {
        assert_eq!(CommandKind::Sync, CommandKind::Sync);

        #[cfg(feature = "async")]
        assert_ne!(CommandKind::Sync, CommandKind::Async);
    }

    #[test]
    fn test_node_type_checking() {
        const CMD: CommandMeta<TestAccessLevel> = CommandMeta {
            id: "test",
            name: "test",
            description: "Test command",
            access_level: TestAccessLevel::User,
            kind: CommandKind::Sync,
            min_args: 0,
            max_args: 0,
        };

        let node = Node::Command(&CMD);
        assert!(node.is_command());
        assert!(!node.is_directory());
        assert_eq!(node.name(), "test");
    }
}
