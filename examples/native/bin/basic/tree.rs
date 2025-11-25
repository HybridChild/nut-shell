//! Command tree definition for the basic example

use native_examples::ExampleAccessLevel;
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};

// =============================================================================
// System Commands
// =============================================================================

pub const CMD_REBOOT: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "system_reboot",
    name: "reboot",
    description: "Reboot the system (simulated)",
    access_level: ExampleAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_STATUS: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "system_status",
    name: "status",
    description: "Show system status",
    access_level: ExampleAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_VERSION: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "system_version",
    name: "version",
    description: "Show version information",
    access_level: ExampleAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_REBOOT),
        Node::Command(&CMD_STATUS),
        Node::Command(&CMD_VERSION),
    ],
    access_level: ExampleAccessLevel::Guest,
};

// =============================================================================
// Config Commands
// =============================================================================

pub const CMD_CONFIG_GET: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "config_get",
    name: "get",
    description: "Get configuration value",
    access_level: ExampleAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

pub const CMD_CONFIG_SET: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "config_set",
    name: "set",
    description: "Set configuration value",
    access_level: ExampleAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 2,
    max_args: 2,
};

const CONFIG_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "config",
    children: &[
        Node::Command(&CMD_CONFIG_GET),
        Node::Command(&CMD_CONFIG_SET),
    ],
    access_level: ExampleAccessLevel::User,
};

// =============================================================================
// Coffee Commands
// =============================================================================

pub const CMD_MAKE_COFFEE: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "coffee_make",
    name: "make",
    description: "Brew coffee",
    access_level: ExampleAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const COFFEE_DIR: Directory<ExampleAccessLevel> = Directory {
    name: "coffee",
    children: &[Node::Command(&CMD_MAKE_COFFEE)],
    access_level: ExampleAccessLevel::User,
};

// =============================================================================
// Root-Level Commands
// =============================================================================

pub const CMD_ECHO: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "echo",
    name: "echo",
    description: "Echo arguments back",
    access_level: ExampleAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 16,
};

pub const CMD_UPTIME: CommandMeta<ExampleAccessLevel> = CommandMeta {
    id: "uptime",
    name: "uptime",
    description: "Show system uptime (simulated)",
    access_level: ExampleAccessLevel::Guest,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// =============================================================================
// Root Directory
// =============================================================================

pub const ROOT: Directory<ExampleAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Directory(&SYSTEM_DIR),
        Node::Directory(&CONFIG_DIR),
        Node::Directory(&COFFEE_DIR),
        Node::Command(&CMD_ECHO),
        Node::Command(&CMD_UPTIME),
    ],
    access_level: ExampleAccessLevel::Guest,
};
