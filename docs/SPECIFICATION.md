# cli-service - Behavioral Specification

This document specifies the exact behavior of the cli-service library, serving as the authoritative reference for implementation.

## Terminal I/O Behavior

### Character Echo

The CLI echoes all printable characters back to the terminal as they are typed, enabling interactive line editing.

**Echo behavior:**
- Regular characters: echoed immediately
- Password input: characters after `:` delimiter are masked with `*`
- Backspace: sends `\b \b` sequence (backspace, space, backspace) to erase character
- Enter: sends `\r\n` (carriage return + line feed)

### Control Characters

**Recognized control characters:**

| Character | Hex Value | Behavior |
|-----------|-----------|----------|
| `BACKSPACE_BS` | 0x08 | Delete last character from buffer, echo `\b \b` |
| `BACKSPACE_DEL` | 0x7F | Same as BACKSPACE_BS (ASCII DEL) |
| `ENTER_LF` | 0x0A | Submit input if buffer non-empty |
| `ENTER_CR` | 0x0D | Submit input if buffer non-empty |
| `TAB` | 0x09 | Trigger tab completion (only when logged in) |
| `ESC` | 0x1B | Begin escape sequence (or clear if followed by ESC) |
| `ESC ESC` | 0x1B 0x1B | Clear input buffer and exit history navigation |

**Backspace behavior:**
- Removes last character from input buffer
- Sends `\b \b` to terminal (move back, overwrite with space, move back)
- No effect if buffer is empty

**Enter behavior:**
- Processes input only if buffer is non-empty
- Empty input (just pressing enter) is ignored
- Echoes `\r\n` to terminal

### Escape Sequences

The CLI recognizes ANSI escape sequences for terminal navigation.

**Supported sequences:**
- `ESC [ A` - Up arrow (previous command in history)
- `ESC [ B` - Down arrow (next command in history)
- `ESC ESC` - Double-ESC: clear input buffer and exit history navigation

**Escape sequence processing:**
- Enters escape mode on receiving `ESC` (0x1B)
- Buffers subsequent characters (max 16 characters)
- Validates CSI (Control Sequence Introducer) format: `[` followed by command character
- Arrow keys only functional when logged in
- Unrecognized sequences are discarded after alphabetic terminator
- Buffer overflow protection: resets to normal mode when buffer reaches 16 characters (discards incomplete sequence)

**Double-ESC clear behavior:**
- `ESC ESC` clears the input buffer and redraws the prompt
- Exits history navigation mode if active (returns to empty prompt)
- Does not interfere with escape sequences (ESC followed by `[` begins sequence as normal)
- Single ESC followed by non-`[` character: clears buffer, then processes the character
- Useful for quickly abandoning long input or exiting history navigation

**State machine:**
1. **Normal mode**: Process characters normally
2. **Escape mode**: Triggered by first ESC, wait for next character
   - If next char is ESC → clear buffer and return to normal mode
   - If next char is `[` → enter sequence mode
   - If next char is other → clear buffer, process char in normal mode
3. **Sequence mode**: Buffer subsequent chars for escape sequence
4. **Sequence completion**: After 2+ chars, check for valid sequence
5. **Termination**: Alphabetic character ends sequence (recognized or discarded)

## Authentication Flow

### Authentication Feature Status

Authentication can be disabled at compile time via Cargo features (see ARCHITECTURE.md). The behavior differs based on this configuration:

**When authentication is enabled (default):**
- System starts in logged-out state requiring login
- Welcome message displayed on activation
- Login prompt shown: `> `
- User must authenticate before accessing commands
- Access control enforced based on user permissions

**When authentication is disabled:**
- System starts in logged-in state with implicit system access
- Welcome message displayed without login instruction
- Command prompt shown immediately: `@/> ` (no username before `@`)
- All commands accessible without login
- No access control enforcement

### Login Process (Authentication Enabled)

**Input format:** `username:password`

**Validation rules:**
1. Must contain exactly one `:` delimiter (additional colons are part of password)
2. Username must be non-empty (characters before first `:`)
3. Password must be non-empty (characters after first `:`)
4. Both fields are trimmed of leading/trailing whitespace

**Valid examples:**
```
admin:secretPass          → username="admin", password="secretPass"
user@domain.com:P@ss:123  → username="user@domain.com", password="P@ss:123"
admin:P@ssw0rd!          → username="admin", password="P@ssw0rd!"
```

**Invalid examples:**
```
admin                     → Missing delimiter
:password                 → Empty username
admin:                    → Empty password
```

### Password Masking

**Masking behavior:**
- Characters typed before `:` are echoed normally
- The `:` character itself is echoed
- All characters after `:` are echoed as `*`
- Backspace works normally but removes masked characters from display

**Example terminal output:**
```
Input:  a d m i n : p a s s
Output: a d m i n : * * * *
```

### Authentication States

**State transitions (authentication enabled):**

```
Inactive → LoggedOut (on activate)
LoggedOut → LoggedIn (valid credentials)
LoggedIn → LoggedOut (logout command)
LoggedIn → Inactive (exit command)
```

**State transitions (authentication disabled):**

```
Inactive → LoggedIn (on activate)
LoggedIn → Inactive (exit command)
```

**State-dependent behavior:**
- **LoggedOut** (auth enabled only): Only accepts login attempts, password masking active, no tab/history
- **LoggedIn**: Full command access, tab completion, history navigation
- **Inactive**: CLI not processing input

### Session Messages

**Default messages (authentication enabled):**
```
Welcome: "Welcome to CLI Service. Please login."
Logged in: "Logged in. Type 'help' for help."
Logged out: "Logged out."
Exit: "Exiting CLI Service."
Invalid login: "Invalid login attempt. Please enter <username>:<password>"
```

**Default messages (authentication disabled):**
```
Welcome: "Welcome to CLI Service. Type 'help' for help."
Exit: "Exiting CLI Service."
```

All messages are customizable via configuration.

## Tab Completion

### Completion Behavior

Tab completion assists with command and directory name entry.

**Triggering:**
- Press `TAB` (0x09) while logged in
- Current input buffer is analyzed for completion

**Completion algorithm:**
1. Parse input as path (may be partial)
2. Determine target directory for completion:
   - Absolute path: start from root
   - Relative path: start from current directory
3. Extract the partial name to complete (last path segment)
4. Find all accessible nodes matching the prefix
5. Apply access control filtering

**Completion results:**

| Scenario | Behavior |
|----------|----------|
| **No matches** | No action, buffer unchanged |
| **Single match (command)** | Auto-complete, add space |
| **Single match (directory)** | Auto-complete, add `/` |
| **Multiple matches** | Display all options, buffer unchanged |
| **Exact match + other options** | Display all options (user may want longer match) |

**Multi-match display format:**
```
> sys<TAB>
  sysinfo
  system

> system/
```

After displaying options, the original input remains in the buffer for further editing.

**Path completion examples:**
```
Input: sy<TAB>
Match: system (directory)
Result: system/

Input: system/re<TAB>
Match: reboot (command)
Result: system/reboot

Input: s<TAB>
Matches: system/, status
Result: Display both, no change

Input: hw/led/<TAB>
Matches: get, set
Result: Display both, no change
```

### Access Control in Completion

Only nodes accessible at the current user's access level are shown.

**Filtering:**
- Nodes with higher required access level are invisible
- Directories are only shown if user can access them
- Commands only shown if user's access level permits execution

**Example:**
```
User (access level: User)
> system/<TAB>
  network    (User level)
  status     (User level)
  [reboot not shown - requires Admin]

Admin (access level: Admin)
> system/<TAB>
  network
  reboot
  status
```

## Command History

### History Buffer

Commands are stored in a circular buffer for recall via arrow keys.

**Buffer properties:**
- Configurable size (default: 10 entries)
- Circular/ring buffer structure
- O(1) add, previous, next operations
- Only successfully executed commands are stored (invalid input/paths not stored)

**Storage rules:**
- Only completed commands are added (after pressing Enter)
- Empty inputs are not stored
- Login attempts are not stored in history
- Commands with parse errors or invalid paths are not stored
- Successfully executed commands are stored regardless of their result (success/error)

### Navigation

**Arrow key behavior:**

| Key | Action |
|-----|--------|
| Up arrow | Navigate to previous command (older) |
| Down arrow | Navigate to next command (newer) |

**Navigation rules:**
1. Up arrow from current input: save current buffer, show most recent history
2. Subsequent up arrows: move backward through history
3. Down arrow: move forward through history
4. Down arrow past newest: restore original buffer (before history navigation began)
5. Typing new input: exit history navigation, resume normal editing
6. Double-ESC (ESC ESC): exit history navigation and clear buffer completely

**State tracking:**
- Current position in history buffer
- Original buffer content (before entering history mode)
- Wraparound behavior when reaching buffer boundaries

**Example session:**
```
History: [cmd1, cmd2, cmd3]
Buffer: new_cmd<UP>      → Buffer shows: cmd3
<UP>                     → Buffer shows: cmd2
<UP>                     → Buffer shows: cmd1
<DOWN>                   → Buffer shows: cmd2
<DOWN>                   → Buffer shows: cmd3
<DOWN>                   → Buffer shows: new_cmd (original)
<ESC><ESC>               → Buffer cleared (exits history mode)
```

**Example: Double-ESC for quick clear:**
```
user@/> system/network/wifi/configure --long-argument<ESC><ESC>
user@/> _                                                        # Cleared!

user@/> test<UP>         → Shows: previous_command
user@/> previous_command<ESC><ESC>
user@/> _                                                        # Cleared, exited history
```

## Response Formatting

### Response Structure

Commands return `CLIResponse` with message and formatting flags.

**Status codes:**
- `Success` - Command completed successfully
- `Error` - Command failed (generic error)
- `InvalidArguments` - Wrong argument count or format
- `InvalidPath` - Path does not exist, is malformed, or user lacks permission (security: inaccessible nodes appear non-existent)

**Formatting flags:**

| Flag | Default | Effect |
|------|---------|--------|
| `showPrompt` | true | Display prompt after response |
| `indentMessage` | true | Indent message with 2 spaces |
| `prefixNewLine` | true | Add newline before message |
| `postfixNewLine` | true | Add newline after message |
| `inlineMessage` | false | Display on same line as prompt |

**Default formatting:**
```
admin@/system> reboot
<prefix newline>
  <indent>System rebooting...
<postfix newline>
admin@/system>
```

### Error Messages

**Standard error messages:**

| Error Type | Message | When Used |
|------------|---------|-----------|
| Invalid path | "Invalid path" | Path does not exist OR user lacks access (nodes invisible to user) |
| Invalid arguments | "Command takes no arguments" (or specific count) | Wrong argument count or format |
| Invalid login | "Invalid login attempt. Please enter <username>:<password>" | Authentication failure |

**Note:** "Access denied" is not used to maintain security - inaccessible nodes appear non-existent.

**Custom error messages:**
Commands can return custom error strings with appropriate status codes.

**Example error responses:**
```
> invalid/path
  Invalid path

> system/reboot
  Invalid path
  [Note: Same error whether path doesn't exist or user lacks access]

> hw/led/set 255
  Invalid argument count. Expected 4 arguments, got 1.

> hw/led/set abc def ghi jkl
  Invalid value: abc ... valid values: 0 .. 255
```

### Prompt Format

**Prompt structure (authentication enabled):** `username@path> `

**Examples (authentication enabled):**
```
> (logged out, no username yet)
user@/>
admin@/system>
guest@/hw/sensors>
```

**Prompt structure (authentication disabled):** `@path> `

**Examples (authentication disabled):**
```
@/>
@/system>
@/hw/sensors>
```

**Components:**
- Username: Current authenticated user (empty when auth disabled)
- `@`: Separator between username and path (always present)
- Path: Current location in directory tree (always starts with `/`)
- `> `: Command prompt indicator (space after `>`)

**Path display:**
- Root directory: `/`
- Subdirectories: `/parent/child` (no trailing slash)

## Global Commands

Reserved keywords that function at any location in the directory tree.

### `?` - Context Help

Display contents of current directory with descriptions.

**Format:**
```
> ?

  <name> - <description>
  <name> - <description>
  ...
```

**Behavior:**
- Lists all accessible nodes in current directory
- Includes both commands and subdirectories
- Filters by access level
- Indented output (2 spaces)
- Each item on separate line
- Items sorted alphabetically (implementation-defined)
- If no accessible nodes: displays empty list (no output after newline)

**Example:**
```
admin@/system> ?

  debug - Debug system commands
  network - Network configuration
  reboot - Reboot the system
  status - Show system status
```

### `help` - Global Help

List all global commands available.

**Format (authentication enabled):**
```
> help

  help      - List global commands
  ?         - Detail items in current directory
  logout    - Exit current session
  clear     - Clear screen
  ESC ESC   - Clear input buffer
```

**Format (authentication disabled):**
```
> help

  help      - List global commands
  ?         - Detail items in current directory
  clear     - Clear screen
  ESC ESC   - Clear input buffer
```

**Behavior:**
- Always available when logged in
- Lists global commands only (not tree-specific commands)
- `logout` command only shown when authentication feature enabled
- ESC ESC is not a command but a keyboard shortcut (always shown for discoverability)
- Indented output format
- Brief descriptions

### `logout` - End Session

Return to login prompt. **Only available when authentication feature is enabled.**

**Behavior (authentication enabled):**
- Available when logged in
- Clears current user
- Transitions to LoggedOut state
- Displays logout message
- Clears command history (implementation-defined)

**Behavior (authentication disabled):**
- Command not available
- Not listed in `help` output

**Example:**
```
admin@/system> logout

  Logged out.

>
```

### `clear` - Clear Screen

Clear terminal screen (platform-dependent).

**Behavior:**
- Sends terminal clear sequence if supported
- Implementation-defined (may be no-op on some platforms)
- Does not affect CLI state or history

## Command Execution

### Argument Parsing

Input is split into path and arguments using whitespace delimiters.

**Parsing rules:**
1. Split input on whitespace (space, tab)
2. First token is command path
3. Remaining tokens are arguments
4. No quote parsing or escape sequences
5. Multiple consecutive spaces = multiple delimiters (empty tokens ignored)

**Examples:**
```
Input: "reboot"
Path: "reboot"
Args: []

Input: "hw/led/set 1 255 0 0"
Path: "hw/led/set"
Args: ["1", "255", "0", "0"]

Input: "command    arg1    arg2"  (multiple spaces)
Path: "command"
Args: ["arg1", "arg2"]
```

### Path Resolution

Both absolute and relative paths are supported.

**Path types:**
- **Absolute**: Start with `/` (e.g., `/system/reboot`)
- **Relative**: No leading `/` (e.g., `system`, `../status`)

**Special path segments:**
- `.` - Current directory (no-op)
- `..` - Parent directory (up one level)
- `name` - Child node by name

**Resolution algorithm:**
1. If absolute: start from root
2. If relative: start from current directory
3. Process each segment left-to-right
4. Validate segment exists and is accessible
5. Return final node (Command or Directory)

**Navigation vs Execution:**
- If final node is Directory → navigate to it
- If final node is Command → execute with provided arguments

**Example resolutions:**
```
Current: /system

Input: "network"         → /system/network (navigate)
Input: "../hw/sensors"   → /hw/sensors (navigate)
Input: "/system/reboot"  → /system/reboot (execute)
Input: "reboot"          → /system/reboot (execute)
```

### Access Control

Every node has an associated access level requirement.

**Enforcement:**
- Check user's access level against node's requirement
- User level must be >= node level
- Applied during path resolution
- Inaccessible nodes are invisible (like non-existent)

**Access check points:**
1. Path resolution (each segment validated)
2. Command execution
3. Tab completion (filtered list)
4. Directory listing (`?` command)

**Denial behavior:**
- Inaccessible nodes are treated as non-existent
- Return "Invalid path" error (same as non-existent nodes)
- Do not reveal node existence through error messages

## Example Command Trees

### Minimal Tree (No Authentication)

For development or unsecured environments.

**Structure:**
```
/
├── info              # Show system info
├── reboot            # Reboot system
└── config/
    ├── get           # Get config value
    └── set           # Set config value
```

**Rust implementation:**
```rust
const INFO_CMD: Command<AccessLevel> = Command {
    name: "info",
    description: "Show system information",
    execute: info_fn,
    access_level: AccessLevel::Public,
    min_args: 0,
    max_args: 0,
};

const REBOOT_CMD: Command<AccessLevel> = Command {
    name: "reboot",
    description: "Reboot the device",
    execute: reboot_fn,
    access_level: AccessLevel::Public,
    min_args: 0,
    max_args: 0,
};

const CONFIG_GET: Command<AccessLevel> = Command {
    name: "get",
    description: "Get config value",
    execute: config_get_fn,
    access_level: AccessLevel::Public,
    min_args: 1,
    max_args: 1,
};

const CONFIG_SET: Command<AccessLevel> = Command {
    name: "set",
    description: "Set config value",
    execute: config_set_fn,
    access_level: AccessLevel::Public,
    min_args: 2,
    max_args: 2,
};

const CONFIG_DIR: Directory<AccessLevel> = Directory {
    name: "config",
    children: &[
        Node::Command(&CONFIG_GET),
        Node::Command(&CONFIG_SET),
    ],
    access_level: AccessLevel::Public,
};

const ROOT: Directory<AccessLevel> = Directory {
    name: "root",
    children: &[
        Node::Command(&INFO_CMD),
        Node::Command(&REBOOT_CMD),
        Node::Directory(&CONFIG_DIR),
    ],
    access_level: AccessLevel::Public,
};
```

### Full-Featured Tree (With Authentication)

**Structure:**
```
/
├── system/          (Admin only)
│   ├── reboot       (Admin only) - Reboot the device
│   └── heap         (Admin only) - Get heap statistics
└── hw/              (User)
    ├── pot/         (User)
    │   └── get      (User) - Read potentiometer value
    ├── rgb/         (User)
    │   └── set      (Admin) - Set RGB LED - Args: <id> <R> <G> <B>
    └── toggle/      (User)
        └── get      (User) - Read toggle switch state
```

**Access levels:**
- User: Can access hw/ tree and read sensors
- Admin: Full access including system/ tree and RGB LED control

**Example session:**
```
Welcome to CLI Service. Please login.

> user:********

  Logged in. Type 'help' for help.

user@/> ?

  hw - Hardware interface commands

user@/> system

  Invalid path

user@/> hw
user@/hw> ?

  pot - Potentiometer interface
  rgb - RGB LED interface
  toggle - Toggle switch interface

user@/hw> rgb
user@/hw/rgb> ?

user@/hw/rgb> set 1 255 0 0

  Invalid path

user@/hw> pot
user@/hw/pot> get

  Potentiometer value: 512

user@/hw/pot> /hw/toggle/get

  Toggle switch state: ON

user@/hw/pot> logout

  Logged out.

> admin:**********

  Logged in. Type 'help' for help.

admin@/> system
admin@/system> ?

  heap - Get heap statistics
  reboot - Reboot the device

admin@/system> /hw/rgb/set 1 255 0 0

  RGB LED 1 set to: 255 0 0

admin@/> logout

  Logged out.
```

## Implementation Requirements

### No-std Compatibility

The library must work in `no_std` environments.

**Requirements:**
- No heap allocation (use fixed-size buffers)
- No standard library dependencies
- Platform-agnostic I/O abstraction
- Const-initializable data structures

**Buffer sizes (configurable):**
- Input buffer: 128-256 bytes
- Command history: 10 entries (default)
- Max path depth: 8 levels
- Max arguments: 16 (default)
- Escape sequence buffer: 16 bytes

### Static Allocation

All memory must be determinable at compile time.

**Requirements:**
- Directory trees in ROM (const-initialized)
- Fixed-size input buffers (heapless::String)
- Fixed-size history buffer (heapless::Vec)
- Stack-allocated temporaries only

### Zero-Cost Abstractions

Generic traits should compile to efficient code.

**Requirements:**
- I/O trait monomorphized per platform
- No vtables or dynamic dispatch
- Inline-friendly designs
- Const evaluation where possible

### Platform Requirements

**Minimum requirements:**
- 8KB RAM (for buffers and stack)
- 32KB flash (for code and const data)
- Serial I/O capability (UART, USB-CDC, or similar)
- Single-threaded execution model

**Target platforms:**
- RP2040 (Raspberry Pi Pico) - primary target
- Native (testing and development)
- Other embedded ARM Cortex-M microcontrollers
