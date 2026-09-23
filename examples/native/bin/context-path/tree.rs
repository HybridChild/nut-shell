//! Command tree definition for the basic example

use native_examples::ExampleAccessLevel;
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};

pub const CMD_SHOWPATH: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "shared_showpath",
    name: "showpath",
    description: "Show the path of the current command",
    access_level: ExampleAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const DIR1_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "dir1",
    children: &[
        Node::Directory(&DIR3_DIR),
        Node::Directory(&DIR4_DIR),
        Node::Command(&CMD_SHOWPATH),
    ],
    access_level: ExampleAccessLevel::Guest,
};

const DIR2_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "dir2",
    children: &[
        Node::Directory(&DIR3_DIR),
        Node::Directory(&DIR4_DIR),
        Node::Command(&CMD_SHOWPATH),
    ],
    access_level: ExampleAccessLevel::Guest,
};

const DIR3_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "dir3",
    children: &[Node::Command(&CMD_SHOWPATH)],
    access_level: ExampleAccessLevel::Guest,
};

const DIR4_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "dir4",
    children: &[Node::Command(&CMD_SHOWPATH)],
    access_level: ExampleAccessLevel::Guest,
};

// =============================================================================
// Root Directory
// =============================================================================

pub const ROOT: Directory<ExampleAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Directory(&DIR1_DIR),
        Node::Directory(&DIR2_DIR),
        Node::Command(&CMD_SHOWPATH),
    ],
    access_level: ExampleAccessLevel::Guest,
};
