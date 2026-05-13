//! Command tree definition for the NUCLEO-H753ZI Embassy example

use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};
use stm32h753zi_embassy_examples::{H753AccessLevel, hw_commands, system_commands};

// =============================================================================
// System directory
// =============================================================================

const CMD_INFO: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "system_info",
    name: "info",
    description: "Show device information",
    access_level: H753AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM_DIR: Directory<H753AccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_INFO),
        Node::Command(&system_commands::CMD_UPTIME),
        Node::Command(&system_commands::CMD_MEMINFO),
        Node::Command(&system_commands::CMD_BENCHMARK),
        Node::Command(&system_commands::CMD_FLASH),
        Node::Command(&system_commands::CMD_CRASH),
    ],
    access_level: H753AccessLevel::User,
};

// =============================================================================
// Hardware directory
// =============================================================================

// hardware/get — read-only hardware status
const HARDWARE_GET_DIR: Directory<H753AccessLevel> = Directory {
    name: "get",
    children: &[
        Node::Command(&hw_commands::CMD_CHIPID),
        Node::Command(&hw_commands::CMD_CLOCKS),
        Node::Command(&hw_commands::CMD_CORE),
        Node::Command(&hw_commands::CMD_BOOTREASON),
    ],
    access_level: H753AccessLevel::User,
};

// hardware/set — control
const CMD_LED: CommandMeta<H753AccessLevel> = CommandMeta {
    id: "hw_led",
    name: "led",
    description: "Control a user LED: led <1|2|3> <on|off|toggle>",
    access_level: H753AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 2,
    max_args: 2,
};

const HARDWARE_SET_DIR: Directory<H753AccessLevel> = Directory {
    name: "set",
    children: &[Node::Command(&CMD_LED)],
    access_level: H753AccessLevel::User,
};

const HARDWARE_DIR: Directory<H753AccessLevel> = Directory {
    name: "hardware",
    children: &[
        Node::Directory(&HARDWARE_GET_DIR),
        Node::Directory(&HARDWARE_SET_DIR),
    ],
    access_level: H753AccessLevel::User,
};

// =============================================================================
// Root
// =============================================================================

pub const ROOT: Directory<H753AccessLevel> = Directory {
    name: "/",
    children: &[Node::Directory(&SYSTEM_DIR), Node::Directory(&HARDWARE_DIR)],
    access_level: H753AccessLevel::User,
};
