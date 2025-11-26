//! Command tree definition for the basic example

use rp_pico_examples::{PicoAccessLevel, hw_commands, system_commands};
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};

// =============================================================================
// System Commands
// =============================================================================

pub const CMD_INFO: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "system_info",
    name: "info",
    description: "Show device information",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM_DIR: Directory<PicoAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_INFO),
        Node::Command(&system_commands::CMD_UPTIME),
        Node::Command(&system_commands::CMD_MEMINFO),
        Node::Command(&system_commands::CMD_BENCHMARK),
        Node::Command(&system_commands::CMD_FLASH),
        Node::Command(&system_commands::CMD_CRASH),
    ],
    access_level: PicoAccessLevel::User,
};

// =============================================================================
// Hardware Commands
// =============================================================================

// Hardware read commands
const HARDWARE_GET_DIR: Directory<PicoAccessLevel> = Directory {
    name: "get",
    children: &[
        Node::Command(&hw_commands::CMD_TEMP),
        Node::Command(&hw_commands::CMD_CHIPID),
        Node::Command(&hw_commands::CMD_CLOCKS),
        Node::Command(&hw_commands::CMD_CORE),
        Node::Command(&hw_commands::CMD_BOOTREASON),
        Node::Command(&hw_commands::CMD_GPIO),
    ],
    access_level: PicoAccessLevel::User,
};

// Hardware write/control commands
const HARDWARE_SET_DIR: Directory<PicoAccessLevel> = Directory {
    name: "set",
    children: &[Node::Command(&hw_commands::CMD_LED)],
    access_level: PicoAccessLevel::User,
};

const HARDWARE_DIR: Directory<PicoAccessLevel> = Directory {
    name: "hardware",
    children: &[
        Node::Directory(&HARDWARE_GET_DIR),
        Node::Directory(&HARDWARE_SET_DIR),
    ],
    access_level: PicoAccessLevel::User,
};

// =============================================================================
// Root Directory
// =============================================================================

pub const ROOT: Directory<PicoAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Directory(&SYSTEM_DIR),
        Node::Directory(&HARDWARE_DIR),
    ],
    access_level: PicoAccessLevel::User,
};
