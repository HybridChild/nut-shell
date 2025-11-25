//! Command tree definition for the embassy_uart_cli example

use rp_pico_examples::PicoAccessLevel;
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};

// =============================================================================
// System Commands
// =============================================================================

pub const CMD_REBOOT: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_reboot",
    name: "reboot",
    description: "Reboot the device",
    access_level: PicoAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_INFO: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_info",
    name: "info",
    description: "Show device information",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

pub const CMD_DELAY: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_delay",
    name: "delay",
    description: "Async delay demonstration (seconds)",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Async,
    min_args: 1,
    max_args: 1,
};

const SYSTEM_DIR: Directory<PicoAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_REBOOT),
        Node::Command(&CMD_INFO),
        Node::Command(&CMD_DELAY),
    ],
    access_level: PicoAccessLevel::User,
};

// =============================================================================
// Root-Level Commands
// =============================================================================

pub const CMD_LED: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "led",
    name: "led",
    description: "Toggle onboard LED",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

// =============================================================================
// Root Directory
// =============================================================================

pub const ROOT: Directory<PicoAccessLevel> = Directory {
    name: "/",
    children: &[Node::Directory(&SYSTEM_DIR), Node::Command(&CMD_LED)],
    access_level: PicoAccessLevel::User,
};
