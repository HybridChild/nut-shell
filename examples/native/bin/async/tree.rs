//! Command tree definition for the async example

use native_examples::ExampleAccessLevel;
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};

// =============================================================================
// Async Commands
// =============================================================================

pub const CMD_DELAY: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "async_delay",
    name: "delay",
    description: "Async delay for N seconds (max 30)",
    access_level: ExampleAccessLevel::Guest,
    kind: CommandKind::Async,
    min_args: 1,
    max_args: 1,
};

pub const CMD_FETCH: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "async_fetch",
    name: "fetch",
    description: "Simulate async HTTP fetch",
    access_level: ExampleAccessLevel::User,
    kind: CommandKind::Async,
    min_args: 1,
    max_args: 1,
};

pub const CMD_COMPUTE: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "async_compute",
    name: "compute",
    description: "Simulate async computation",
    access_level: ExampleAccessLevel::User,
    kind: CommandKind::Async,
    min_args: 0,
    max_args: 0,
};

const ASYNC_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "async",
    children: &[
        Node::Command(&CMD_DELAY),
        Node::Command(&CMD_FETCH),
        Node::Command(&CMD_COMPUTE),
    ],
    access_level: ExampleAccessLevel::Guest,
};

// =============================================================================
// Sync Commands
// =============================================================================

pub const CMD_ECHO: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "sync_echo",
    name: "echo",
    description: "Echo arguments back",
    access_level: ExampleAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 16,
};

pub const CMD_INFO: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "sync_info",
    name: "info",
    description: "Show system information",
    access_level: ExampleAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_REBOOT: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "sync_reboot",
    name: "reboot",
    description: "Reboot the system (simulated)",
    access_level: ExampleAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "system",
    children: &[Node::Command(&CMD_REBOOT), Node::Command(&CMD_INFO)],
    access_level: ExampleAccessLevel::Guest,
};

// =============================================================================
// Root Directory
// =============================================================================

pub const ROOT: Directory<ExampleAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Directory(&SYSTEM_DIR),
        Node::Directory(&ASYNC_DIR),
        Node::Command(&CMD_ECHO),
    ],
    access_level: ExampleAccessLevel::Guest,
};
