//! Command tree definition for the embassy example

use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};
use rp_pico_examples::{PicoAccessLevel, hw_commands, system_commands};

// =============================================================================
// LED Control Command (Embassy channel-based)
// =============================================================================

pub const CMD_LED: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "led",
    name: "led",
    description: "Control onboard LED (on/off)",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

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

pub const CMD_TEMP: CommandMeta<PicoAccessLevel> = CommandMeta {
    id: "hw_temp",
    name: "temp",
    description: "Read internal temperature sensor",
    access_level: PicoAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// Hardware read commands
const HARDWARE_GET_DIR: Directory<PicoAccessLevel> = Directory {
    name: "get",
    children: &[
        Node::Command(&CMD_TEMP),
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
    children: &[Node::Command(&CMD_LED)],
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
        Node::Command(&CMD_DELAY),
    ],
    access_level: PicoAccessLevel::User,
};
