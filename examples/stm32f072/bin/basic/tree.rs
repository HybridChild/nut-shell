//! Command tree definition for the NUCLEO-F072RB example

use stm32_examples::{Stm32AccessLevel, hw_commands, system_commands};
use nut_shell::tree::{CommandKind, CommandMeta, Directory, Node};

// =============================================================================
// System Commands
// =============================================================================

pub const CMD_INFO: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "system_info",
    name: "info",
    description: "Show device information",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM_DIR: Directory<Stm32AccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&CMD_INFO),
        Node::Command(&system_commands::CMD_UPTIME),
        Node::Command(&system_commands::CMD_MEMINFO),
        Node::Command(&system_commands::CMD_BENCHMARK),
        Node::Command(&system_commands::CMD_FLASH),
        Node::Command(&system_commands::CMD_CRASH),
    ],
    access_level: Stm32AccessLevel::User,
};

// =============================================================================
// Hardware Commands
// =============================================================================

pub const CMD_TEMP: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "hw_temp",
    name: "temp",
    description: "Read internal temperature sensor",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// Hardware read commands
const HARDWARE_GET_DIR: Directory<Stm32AccessLevel> = Directory {
    name: "get",
    children: &[
        Node::Command(&CMD_TEMP),
        Node::Command(&hw_commands::CMD_CHIPID),
        Node::Command(&hw_commands::CMD_CLOCKS),
        Node::Command(&hw_commands::CMD_CORE),
        Node::Command(&hw_commands::CMD_BOOTREASON),
    ],
    access_level: Stm32AccessLevel::User,
};

pub const CMD_LED: CommandMeta<Stm32AccessLevel> = CommandMeta {
    id: "hw_led",
    name: "led",
    description: "Control USER LED (on/off)",
    access_level: Stm32AccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 1,
    max_args: 1,
};

// Hardware write/control commands
const HARDWARE_SET_DIR: Directory<Stm32AccessLevel> = Directory {
    name: "set",
    children: &[Node::Command(&CMD_LED)],
    access_level: Stm32AccessLevel::User,
};

const HARDWARE_DIR: Directory<Stm32AccessLevel> = Directory {
    name: "hardware",
    children: &[
        Node::Directory(&HARDWARE_GET_DIR),
        Node::Directory(&HARDWARE_SET_DIR),
    ],
    access_level: Stm32AccessLevel::User,
};

// =============================================================================
// Root Directory
// =============================================================================

pub const ROOT: Directory<Stm32AccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Directory(&SYSTEM_DIR),
        Node::Directory(&HARDWARE_DIR),
    ],
    access_level: Stm32AccessLevel::User,
};
