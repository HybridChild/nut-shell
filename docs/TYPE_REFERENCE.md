# Type Reference

**Purpose**: Complete type definitions and method signatures for implementation reference.

**When to use this document**: During implementation when you need exact field names, types, method signatures, or constant values.

**Related Documentation**:
- **[DESIGN.md](DESIGN.md)**: Why these types are designed this way
- **[INTERNALS.md](INTERNALS.md)**: How these types interact at runtime
- **[SPECIFICATION.md](SPECIFICATION.md)**: Behavioral requirements
- **[IMPLEMENTATION.md](IMPLEMENTATION.md)**: Implementation phases and tasks

---

## Table of Contents

1. [Configuration](#configuration)
2. [Core Traits](#core-traits)
3. [Tree Types](#tree-types)
4. [Request/Response Types](#requestresponse-types)
5. [Shell Types](#shell-types)
6. [Parser Types](#parser-types)
7. [Error Types](#error-types)
8. [Method Signatures](#method-signatures)

---

## Configuration

### ShellConfig Trait

Buffer sizes and limits are **user-configurable** via the `ShellConfig` trait. This allows optimization for different embedded targets (tiny MCU vs larger systems) with zero runtime cost.

```rust
/// Configuration for Shell buffer sizes and limits
/// All constants are evaluated at compile time (zero runtime cost)
pub trait ShellConfig {
    /// Maximum input buffer size (characters)
    /// Used for: Command input, path strings
    const MAX_INPUT: usize;

    /// Maximum path depth (directory nesting)
    /// Used for: Path navigation stack
    const MAX_PATH_DEPTH: usize;

    /// Maximum number of command arguments
    /// Used for: Argument parsing
    const MAX_ARGS: usize;

    /// Maximum prompt length
    /// Used for: Prompt generation buffer
    /// Format: "username@/path/to/dir> " with margin
    const MAX_PROMPT: usize;

    /// Maximum response message length
    /// Used for: Command response strings
    const MAX_RESPONSE: usize;

    /// Command history size
    /// Used for: CommandHistory buffer (when history feature enabled)
    const HISTORY_SIZE: usize;
}
```

### Provided Configurations

**DefaultConfig** - Balanced for typical embedded systems:
```rust
/// Default configuration (recommended starting point)
/// Suitable for: RP2040, STM32, ESP32, and similar MCUs
pub struct DefaultConfig;

impl ShellConfig for DefaultConfig {
    const MAX_INPUT: usize = 128;      // ~128 bytes
    const MAX_PATH_DEPTH: usize = 8;   // 8 levels deep
    const MAX_ARGS: usize = 16;        // 16 arguments max
    const MAX_PROMPT: usize = 64;      // ~64 bytes
    const MAX_RESPONSE: usize = 256;   // ~256 bytes
    const HISTORY_SIZE: usize = 10;    // 10 commands
}
```

**MinimalConfig** - Resource-constrained systems:
```rust
/// Minimal configuration (tight memory constraints)
/// Suitable for: Cortex-M0, tiny MCUs with <8KB RAM
pub struct MinimalConfig;

impl ShellConfig for MinimalConfig {
    const MAX_INPUT: usize = 64;       // ~64 bytes
    const MAX_PATH_DEPTH: usize = 4;   // 4 levels deep
    const MAX_ARGS: usize = 8;         // 8 arguments max
    const MAX_PROMPT: usize = 32;      // ~32 bytes
    const MAX_RESPONSE: usize = 128;   // ~128 bytes
    const HISTORY_SIZE: usize = 5;     // 5 commands
}
```

### Custom Configuration

Users can define custom configurations for their specific needs:

```rust
/// Custom configuration for a specific application
struct MyAppConfig;

impl ShellConfig for MyAppConfig {
    const MAX_INPUT: usize = 256;      // Long commands
    const MAX_PATH_DEPTH: usize = 12;  // Deep nesting
    const MAX_ARGS: usize = 32;        // Many arguments
    const MAX_PROMPT: usize = 80;      // Wide terminal
    const MAX_RESPONSE: usize = 512;   // Verbose output
    const HISTORY_SIZE: usize = 20;    // Lots of history
}
```

**Memory calculation example** (DefaultConfig):
```
Input buffer:    128 bytes
Path stack:      8 * 4 = 32 bytes (usize indices)
Prompt buffer:   64 bytes
Response buffer: 256 bytes
History:         10 * 128 = 1280 bytes (with feature)
Parser state:    ~8 bytes
-----------------
Total:           ~1768 bytes (with history)
Total:           ~488 bytes (without history)
```

**Recommended placement**: `src/config.rs` or `src/lib.rs`

---

## Core Traits

### CharIo Trait

```rust
/// Character-based I/O abstraction
/// Platform-specific implementations provide actual I/O
pub trait CharIo {
    /// I/O error type (platform-specific)
    type Error: core::fmt::Debug;

    /// Non-blocking read of a single character
    /// Returns:
    ///   - Ok(Some(c)) - Character available
    ///   - Ok(None) - No character available (non-blocking)
    ///   - Err(e) - I/O error occurred
    fn get_char(&mut self) -> Result<Option<char>, Self::Error>;

    /// Write a single character
    /// Returns:
    ///   - Ok(()) - Character written successfully
    ///   - Err(e) - I/O error occurred
    fn put_char(&mut self, c: char) -> Result<(), Self::Error>;

    /// Optional: Write a string (default implementation uses put_char)
    fn put_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.chars() {
            self.put_char(c)?;
        }
        Ok(())
    }
}
```

**See also**: [IO_DESIGN.md](IO_DESIGN.md) for implementation patterns.

### AccessLevel Trait

```rust
/// Hierarchical access control levels
/// User-defined enum must implement this trait
pub trait AccessLevel: Copy + Clone + PartialOrd + Ord + PartialEq + Eq {
    /// Parse access level from string (for credential storage)
    /// Returns None if string doesn't match any level
    fn from_str(s: &str) -> Option<Self>;

    /// Convert access level to string (for display)
    fn as_str(&self) -> &'static str;
}
```

**Example implementation**:
```rust
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MyAccessLevel {
    Guest = 0,
    User = 1,
    Admin = 2,
}

impl AccessLevel for MyAccessLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Guest" => Some(Self::Guest),
            "User" => Some(Self::User),
            "Admin" => Some(Self::Admin),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Guest => "Guest",
            Self::User => "User",
            Self::Admin => "Admin",
        }
    }
}
```

### CommandHandlers Trait

```rust
/// Command execution dispatcher
/// Maps command names to execution functions
/// User implements this to provide command logic
/// Generic over ShellConfig to match Response buffer sizes
pub trait CommandHandlers<C: ShellConfig> {
    /// Execute synchronous command by name
    /// Called when CommandMeta.kind == CommandKind::Sync
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>;

    /// Execute asynchronous command by name
    /// Called when CommandMeta.kind == CommandKind::Async
    /// Only available when `async` feature enabled
    #[cfg(feature = "async")]
    async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>;
}
```

**Note**: Handler is responsible for mapping command names to actual functions. Shell validates arguments before calling handler.

### CredentialProvider Trait

```rust
/// Credential storage and verification abstraction
/// Enables multiple storage backends (flash, const, external systems)
/// Only available when `authentication` feature enabled
#[cfg(feature = "authentication")]
pub trait CredentialProvider<L: AccessLevel> {
    /// Storage backend error type
    type Error: core::fmt::Debug;

    /// Find user by username
    /// Returns:
    ///   - Ok(Some(user)) - User found with password hash and salt
    ///   - Ok(None) - User not found
    ///   - Err(e) - Storage error
    fn find_user(&self, username: &str) -> Result<Option<User<L>>, Self::Error>;

    /// Verify password against user's stored hash
    /// Uses constant-time comparison to prevent timing attacks
    /// Returns:
    ///   - true - Password matches
    ///   - false - Password incorrect
    fn verify_password(&self, user: &User<L>, password: &str) -> bool;

    /// List all usernames (optional, for admin commands)
    /// Returns:
    ///   - Ok(Vec) - List of usernames
    ///   - Err(e) - Storage error
    fn list_users(&self) -> Result<heapless::Vec<&str, 16>, Self::Error>;
}
```

**Implementation notes**:
- Typically uses `PasswordHasher` trait for password verification
- Multiple storage backends: const (testing), flash (production), build-time env vars
- See [SECURITY.md](SECURITY.md) for implementation patterns and security requirements

### PasswordHasher Trait

```rust
/// Password hashing abstraction
/// Default implementation: SHA-256 with per-user salts
/// Only available when `authentication` feature enabled
#[cfg(feature = "authentication")]
pub trait PasswordHasher {
    /// Hash password with salt
    /// Returns: 32-byte hash (SHA-256)
    fn hash(&self, password: &str, salt: &[u8]) -> [u8; 32];

    /// Verify password against stored hash
    /// MUST use constant-time comparison to prevent timing attacks
    /// Returns: true if password matches
    fn verify(&self, password: &str, salt: &[u8], hash: &[u8; 32]) -> bool;
}
```

**Default implementation (Sha256Hasher)**:
```rust
/// SHA-256 password hasher with constant-time verification
pub struct Sha256Hasher;

impl PasswordHasher for Sha256Hasher {
    fn hash(&self, password: &str, salt: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(salt);  // Salt first
        hasher.update(password.as_bytes());
        hasher.finalize().into()
    }

    fn verify(&self, password: &str, salt: &[u8], hash: &[u8; 32]) -> bool {
        let computed = self.hash(password, salt);
        // Constant-time comparison using subtle crate
        subtle::ConstantTimeEq::ct_eq(&computed, hash).into()
    }
}
```

**Security notes**:
- Uses SHA-256 (not bcrypt/Argon2) for embedded memory constraints
- Per-user 16-byte salts required (prevents rainbow table attacks)
- Constant-time verification prevents timing attacks
- See [SECURITY.md](SECURITY.md) for security considerations and threat model

---

## Tree Types

### Node Enum

```rust
/// Tree node - either a command or directory
/// Zero-cost dispatch via pattern matching
pub enum Node<L: AccessLevel> {
    /// Command leaf node (metadata reference)
    Command(&'static CommandMeta<L>),

    /// Directory node (contains children)
    Directory(&'static Directory<L>),
}

impl<L: AccessLevel> Node<L> {
    /// Check if this node is a command
    pub fn is_command(&self) -> bool {
        matches!(self, Node::Command(_))
    }

    /// Check if this node is a directory
    pub fn is_directory(&self) -> bool {
        matches!(self, Node::Directory(_))
    }

    /// Get node name (works for both variants)
    pub fn name(&self) -> &'static str {
        match self {
            Node::Command(cmd) => cmd.name,
            Node::Directory(dir) => dir.name,
        }
    }

    /// Get node access level
    pub fn access_level(&self) -> &L {
        match self {
            Node::Command(cmd) => &cmd.access_level,
            Node::Directory(dir) => &dir.access_level,
        }
    }
}
```

### CommandMeta Struct

```rust
/// Command metadata (const-initializable)
/// Execution logic provided separately via CommandHandlers trait
pub struct CommandMeta<L: AccessLevel> {
    /// Command name (must be unique within directory)
    pub name: &'static str,

    /// Short description (shown in `ls` output)
    pub description: &'static str,

    /// Minimum access level required to execute
    pub access_level: L,

    /// Command execution type (Sync or Async)
    pub kind: CommandKind,

    /// Minimum number of arguments required
    pub min_args: usize,

    /// Maximum number of arguments allowed
    pub max_args: usize,
}
```

### CommandKind Enum

```rust
/// Command execution type marker
/// Determines which handler method Shell calls
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CommandKind {
    /// Synchronous command (calls execute_sync)
    Sync,

    /// Asynchronous command (calls execute_async)
    /// Requires `async` feature
    Async,
}
```

### Directory Struct

```rust
/// Directory node containing child nodes
pub struct Directory<L: AccessLevel> {
    /// Directory name
    pub name: &'static str,

    /// Child nodes (commands and subdirectories)
    /// Stored as static slice for const initialization
    pub children: &'static [Node<L>],

    /// Minimum access level required to enter directory
    pub access_level: L,
}

impl<L: AccessLevel> Directory<L> {
    /// Find direct child by name
    /// Returns None if not found
    pub fn find_child(&self, name: &str) -> Option<&Node<L>> {
        self.children.iter().find(|node| node.name() == name)
    }

    /// Get all children (for `ls` command)
    pub fn children(&self) -> &[Node<L>] {
        self.children
    }
}
```

### Path Type

```rust
/// Unix-style path parser and representation
/// Handles absolute and relative paths with `.` and `..` navigation
/// Zero-allocation parsing using string slices
pub struct Path<'a> {
    /// Original path string
    original: &'a str,

    /// Whether this is an absolute path (starts with `/`)
    is_absolute: bool,

    /// Path segments (directories/commands)
    /// Does not include `.` or `..` (those are processed during resolution)
    segments: heapless::Vec<&'a str, MAX_PATH_DEPTH>,
}

impl<'a> Path<'a> {
    /// Parse path string into Path structure
    /// Supports:
    ///   - Absolute paths: `/system/reboot`
    ///   - Relative paths: `../network/status`, `./cmd`, `cmd`
    ///   - Parent navigation: `..` (go up one level)
    ///   - Current directory: `.` (stay at current level)
    /// Returns:
    ///   - Ok(Path) - Successfully parsed
    ///   - Err(CliError::InvalidPath) - Invalid path syntax
    ///   - Err(CliError::PathTooDeep) - Exceeds MAX_PATH_DEPTH
    pub fn parse(input: &'a str) -> Result<Self, CliError>;

    /// Check if this is an absolute path (starts with `/`)
    pub fn is_absolute(&self) -> bool;

    /// Get iterator over path segments
    /// Segments do NOT include `.` or `..`
    /// Those are represented in the segments slice as-is and
    /// processed during tree traversal
    pub fn segments(&self) -> impl Iterator<Item = &str>;

    /// Get original path string (for error messages)
    pub fn as_str(&self) -> &str;
}
```

**Parsing behavior**:
```rust
// Absolute paths
Path::parse("/system/reboot")
  => Path { is_absolute: true, segments: ["system", "reboot"] }

// Relative paths
Path::parse("network/status")
  => Path { is_absolute: false, segments: ["network", "status"] }

// Parent navigation
Path::parse("../hw/led")
  => Path { is_absolute: false, segments: ["..", "hw", "led"] }

// Current directory
Path::parse("./cmd")
  => Path { is_absolute: false, segments: [".", "cmd"] }

// Complex navigation
Path::parse("../../system/debug")
  => Path { is_absolute: false, segments: ["..", "..", "system", "debug"] }
```

**Resolution behavior** (see INTERNALS.md Level 4):
- Start from root (absolute) or current directory (relative)
- For each segment:
  - `..` → Pop from path stack (go up one level)
  - `.` → No-op (stay at current level)
  - Other → Find child in current directory, push to path stack
- Access control checked at EVERY segment during traversal
- Returns `InvalidPath` for both nonexistent and inaccessible nodes (security)

**Example tree construction**:
```rust
const REBOOT_CMD: CommandMeta<MyAccessLevel> = CommandMeta {
    name: "reboot",
    description: "Reboot the device",
    access_level: MyAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const SYSTEM_DIR: Directory<MyAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&REBOOT_CMD),
        // ... other nodes
    ],
    access_level: MyAccessLevel::User,
};

const ROOT: Directory<MyAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Directory(&SYSTEM_DIR),
        // ... other nodes
    ],
    access_level: MyAccessLevel::Guest,
};
```

---

## Request/Response Types

### HistoryDirection Enum

```rust
/// History navigation direction (up/down arrows)
/// Used by Request::History variant
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum HistoryDirection {
    /// Up arrow - navigate to previous command in history
    Previous = 0,

    /// Down arrow - navigate to next command (or back to original buffer)
    Next = 1,
}
```

**Usage**:
- `HistoryDirection::Previous` - Up arrow key pressed
- `HistoryDirection::Next` - Down arrow key pressed
- Size: 1 byte (same as bool, but self-documenting)

### Request Enum

```rust
/// Parsed user input request
/// Returned by InputParser when input is complete
/// Generic over ShellConfig to use configured buffer sizes
pub enum Request<C: ShellConfig> {
    /// Authentication attempt (feature: authentication)
    #[cfg(feature = "authentication")]
    Login {
        username: heapless::String<32>,
        password: heapless::String<64>,
    },

    /// Failed login attempt (feature: authentication)
    #[cfg(feature = "authentication")]
    InvalidLogin,

    /// Execute command
    Command {
        /// Parsed command path (e.g., "system/reboot" or "/hw/led")
        path: heapless::String<C::MAX_INPUT>,

        /// Parsed arguments
        args: heapless::Vec<heapless::String<C::MAX_INPUT>, C::MAX_ARGS>,

        /// Original input string (for history)
        /// Only present when history feature enabled (saves ~128 bytes RAM when disabled)
        #[cfg(feature = "history")]
        original: heapless::String<C::MAX_INPUT>,
    },

    /// Request tab completion suggestions
    #[cfg(feature = "completion")]
    TabComplete {
        /// Current input to complete
        path: heapless::String<C::MAX_INPUT>,
    },

    /// Navigate command history
    #[cfg(feature = "history")]
    History {
        /// Direction of navigation (Previous = up arrow, Next = down arrow)
        direction: HistoryDirection,

        /// Current buffer contents (to restore if needed)
        buffer: heapless::String<C::MAX_INPUT>,
    },
}
```

**Usage with conditional field**:
```rust
// Creating Command request (conditional field)
let request = Request::Command {
    path,
    args,
    #[cfg(feature = "history")]
    original: input_buffer.clone(),
};

// Pattern matching (conditional field)
match request {
    Request::Command {
        path,
        args,
        #[cfg(feature = "history")] original
    } => {
        // Execute command
        let response = execute_command(path, args)?;

        // Add to history if enabled
        #[cfg(feature = "history")]
        if !response.exclude_from_history {
            history.add(&original);
        }
    }
}
```

### Response Struct

```rust
/// Command execution response
/// Controls output formatting and history behavior
/// Generic over ShellConfig to use configured buffer size
pub struct Response<C: ShellConfig> {
    /// Response message content
    pub message: heapless::String<C::MAX_RESPONSE>,

    /// Formatting flags (set by command or Shell)
    pub inline_message: bool,   // Message is inline (don't echo \r\n after command input)
    pub prefix_newline: bool,   // Add \r\n before message
    pub indent_message: bool,   // Indent with 2 spaces
    pub postfix_newline: bool,  // Add \r\n after message
    pub show_prompt: bool,      // Show prompt after response

    /// Prevent input from being saved to history
    /// Only available when `history` feature enabled
    /// Use for sensitive commands (passwords, credentials)
    #[cfg(feature = "history")]
    pub exclude_from_history: bool,
}

impl<C: ShellConfig> Response<C> {
    /// Create success response (default formatting, included in history)
    pub fn success(message: &str) -> Self {
        Self {
            message: heapless::String::from(message).unwrap_or_default(),
            inline_message: false,  // Default: echo newline after command
            prefix_newline: true,
            indent_message: false,
            postfix_newline: true,
            show_prompt: true,
            #[cfg(feature = "history")]
            exclude_from_history: false,
        }
    }

    /// Create success response excluded from history (feature: history)
    #[cfg(feature = "history")]
    pub fn success_no_history(message: &str) -> Self {
        let mut resp = Self::success(message);
        resp.exclude_from_history = true;
        resp
    }

    /// Builder method: Exclude this command from history (feature: history)
    #[cfg(feature = "history")]
    pub fn without_history(mut self) -> Self {
        self.exclude_from_history = true;
        self
    }
}
```

**Default formatting guidelines**:
- Global commands (`?`, `ls`, `logout`, `clear`): Return `Ok(Response::success(...))`
- Custom commands: Return `Ok(Response::success(...))` on success
- Command failures: Return `Err(CliError::CommandFailed(msg))` or other appropriate `CliError` variant
- Sensitive commands: Use `.without_history()` or `success_no_history()` to exclude from command history

---

## Shell Types

### CliState Enum

```rust
/// CLI session state machine
pub enum CliState {
    /// CLI not active (before activate() called)
    Inactive,

    /// Awaiting authentication
    /// Only exists when `authentication` feature enabled
    /// Becomes a "missing variant" when feature disabled
    #[cfg(feature = "authentication")]
    LoggedOut,

    /// Authenticated or auth-disabled mode
    /// Semantics depend on mode:
    ///   - Auth enabled: User successfully logged in
    ///   - Auth disabled: No authentication required
    LoggedIn,
}
```

**Pattern matching with feature gates**:
```rust
// When authentication feature enabled:
match self.state {
    CliState::Inactive => { /* ... */ }
    CliState::LoggedOut => { /* ... */ }
    CliState::LoggedIn => { /* ... */ }
}

// When authentication feature disabled:
// LoggedOut variant doesn't exist, compiler enforces this
match self.state {
    CliState::Inactive => { /* ... */ }
    CliState::LoggedIn => { /* ... */ }
}
```

### User Struct

```rust
/// Authenticated user information
/// Always available (not feature-gated), but only used when authentication enabled
#[derive(Clone)]
pub struct User<L: AccessLevel> {
    /// Username
    pub username: heapless::String<32>,

    /// User's access level
    pub access_level: L,
}

impl<L: AccessLevel> User<L> {
    /// Create new user
    pub fn new(username: &str, access_level: L) -> Self {
        Self {
            username: heapless::String::from(username).unwrap_or_default(),
            access_level,
        }
    }

    /// Get username as string
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Get user's access level
    pub fn access_level(&self) -> &L {
        &self.access_level
    }
}
```

**Note**: User struct is always available (used in method signatures), but `current_user: Option<User<L>>` is only `Some` when authentication is enabled AND user is logged in.

### Shell Struct

```rust
/// Main CLI shell orchestrator
/// Generic over configuration to allow customizable buffer sizes
pub struct Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    /// Reference to command tree (lives in ROM)
    tree: &'tree Directory<L>,

    /// Current authenticated user
    /// None in two cases:
    ///   1. Auth disabled (always None, no user needed)
    ///   2. Auth enabled but not logged in yet
    current_user: Option<User<L>>,

    /// CLI state machine
    state: CliState,

    /// Input buffer (accumulates characters)
    /// Size determined by ShellConfig
    input_buffer: heapless::String<C::MAX_INPUT>,

    /// Current directory path (index stack from root)
    /// Empty = root directory
    /// Depth determined by ShellConfig
    current_path: heapless::Vec<usize, C::MAX_PATH_DEPTH>,

    /// Input parser (handles escape sequences)
    parser: InputParser,

    /// Command history (stub when feature disabled)
    /// Size determined by ShellConfig
    /// Takes two const generics: history size (N) and input buffer size (INPUT_SIZE)
    history: CommandHistory<C::HISTORY_SIZE, C::MAX_INPUT>,

    /// I/O abstraction
    io: IO,

    /// Command execution handlers
    handlers: H,

    /// Credential provider (only when authentication enabled)
    #[cfg(feature = "authentication")]
    credential_provider: &'tree dyn CredentialProvider<L>,

    /// Config type marker (zero-size)
    _config: core::marker::PhantomData<C>,
}
```

**Generic parameters**:
- `'tree`: Lifetime of tree reference (typically `'static`)
- `L`: User-defined AccessLevel implementation
- `IO`: CharIo implementation (platform-specific)
- `H`: CommandHandlers implementation (user-defined)
- `C`: ShellConfig implementation (buffer sizes and limits)

**Constructors** (see IMPLEMENTATION.md Phase 8 for detailed examples):
```rust
// With authentication feature enabled
#[cfg(feature = "authentication")]
impl<'tree, L, IO, H, C> Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    pub fn new(
        tree: &'tree Directory<L>,
        handlers: H,
        credential_provider: &'tree dyn CredentialProvider<L>,
        io: IO,
    ) -> Self {
        // Starts in LoggedOut state
    }
}

// With authentication feature disabled
#[cfg(not(feature = "authentication"))]
impl<'tree, L, IO, H, C> Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    pub fn new(
        tree: &'tree Directory<L>,
        handlers: H,
        io: IO,
    ) -> Self {
        // Starts in LoggedIn state (no user needed)
    }
}
```

---

## Parser Types

### InputParser Struct

```rust
/// Terminal input parser with escape sequence handling
pub struct InputParser {
    /// Parser state machine
    state: ParserState,
}

/// Parser state for escape sequence detection
enum ParserState {
    /// Normal input mode
    Normal,

    /// Saw ESC, waiting for next character
    /// Next ESC = clear buffer, next '[' = sequence start
    EscapeStart,

    /// Saw ESC [, waiting for sequence terminator
    EscapeSequence,
}

/// Events returned by parser
pub enum ParseEvent {
    /// No special action needed
    None,

    /// Character added to buffer
    CharAdded(char),

    /// Backspace pressed
    Backspace,

    /// Enter pressed (input complete)
    Enter,

    /// Tab pressed
    Tab,

    /// Up arrow (history previous)
    UpArrow,

    /// Down arrow (history next)
    DownArrow,

    /// Double-ESC (clear buffer and redraw)
    ClearAndRedraw,
}

impl InputParser {
    /// Create new parser in Normal state
    pub fn new() -> Self {
        Self {
            state: ParserState::Normal,
        }
    }

    /// Process single character through state machine
    /// Updates buffer based on event type
    /// Generic over buffer size (N) to support any configuration
    pub fn process_char<const N: usize>(
        &mut self,
        c: char,
        buffer: &mut heapless::String<N>,
    ) -> Result<ParseEvent, CliError> {
        // State machine implementation
    }
}
```

### CommandHistory Type

```rust
/// Command history with circular buffer
/// Generic over history size (N) and input buffer size (INPUT_SIZE)

// Feature-enabled: Full implementation
#[cfg(feature = "history")]
pub struct CommandHistory<const N: usize, const INPUT_SIZE: usize> {
    buffer: heapless::Vec<heapless::String<INPUT_SIZE>, N>,
    position: Option<usize>,
}

// Feature-disabled: Zero-size stub
#[cfg(not(feature = "history"))]
pub struct CommandHistory<const N: usize, const INPUT_SIZE: usize> {
    _phantom: core::marker::PhantomData<[(); N]>,
}

// Feature-enabled: Full implementation
#[cfg(feature = "history")]
impl<const N: usize, const INPUT_SIZE: usize> CommandHistory<N, INPUT_SIZE> {
    /// Create new empty history
    pub fn new() -> Self {
        Self {
            buffer: heapless::Vec::new(),
            position: None,
        }
    }

    /// Add command to history
    pub fn add(&mut self, cmd: &str) {
        // Real implementation - push to circular buffer
        // If full, oldest entry is replaced
    }

    /// Get previous command (up arrow)
    /// Returns None at beginning of history
    pub fn previous(&mut self) -> Option<heapless::String<INPUT_SIZE>> {
        // Real implementation - navigate backward
        // Saves current buffer on first navigation
    }

    /// Get next command (down arrow)
    /// Returns None at end (restores original buffer)
    pub fn next(&mut self) -> Option<heapless::String<INPUT_SIZE>> {
        // Real implementation - navigate forward
    }
}

// Feature-disabled: Stub implementation (no-ops)
#[cfg(not(feature = "history"))]
impl<const N: usize, const INPUT_SIZE: usize> CommandHistory<N, INPUT_SIZE> {
    /// Create new empty history (zero-size)
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    /// Add command to history (no-op)
    pub fn add(&mut self, _cmd: &str) {
        // No-op
    }

    /// Get previous command (always None)
    pub fn previous(&mut self) -> Option<heapless::String<INPUT_SIZE>> {
        None
    }

    /// Get next command (always None)
    pub fn next(&mut self) -> Option<heapless::String<INPUT_SIZE>> {
        None
    }
}
```

**Usage in Shell**:
```rust
// Shell instantiates with config values
history: CommandHistory<C::HISTORY_SIZE, C::MAX_INPUT>
```

---

## Error Types

### CliError Enum

```rust
/// CLI error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// Command not found in tree
    CommandNotFound,

    /// Path doesn't exist or user lacks access
    /// Intentionally ambiguous for security (don't reveal existence)
    /// SECURITY: Never create a separate "AccessDenied" error - always use InvalidPath
    InvalidPath,

    /// Wrong number of arguments
    InvalidArgumentCount {
        expected_min: usize,
        expected_max: usize,
        received: usize,
    },

    /// Invalid argument format/type (e.g., expected integer, got string)
    InvalidArgumentFormat {
        arg_index: usize,
        expected: heapless::String<32>,
    },

    /// Buffer capacity exceeded
    BufferFull,

    /// Path exceeds maximum depth (MAX_PATH_DEPTH)
    PathTooDeep,

    /// Authentication failed (wrong username/password)
    #[cfg(feature = "authentication")]
    AuthenticationFailed,

    /// Not logged in (tried to execute command while logged out)
    #[cfg(feature = "authentication")]
    NotAuthenticated,

    /// I/O error occurred
    IoError,

    /// Async command called in sync mode (process_char)
    /// Only relevant when async feature enabled
    #[cfg(feature = "async")]
    AsyncNotSupported,

    /// Operation timed out
    /// Used by command implementations with timeout logic
    Timeout,

    /// Command executed but reported failure
    /// Use this to return error messages from commands
    CommandFailed(heapless::String<MAX_RESPONSE>),

    /// Generic error with message
    Other(heapless::String<MAX_RESPONSE>),
}

impl core::fmt::Display for CliError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CliError::CommandNotFound => write!(f, "Command not found"),
            CliError::InvalidPath => write!(f, "Invalid path"),
            CliError::InvalidArgumentCount { expected_min, expected_max, received } => {
                if expected_min == expected_max {
                    write!(f, "Expected {} arguments, got {}", expected_min, received)
                } else {
                    write!(f, "Expected {}-{} arguments, got {}", expected_min, expected_max, received)
                }
            }
            CliError::InvalidArgumentFormat { arg_index, expected } => {
                write!(f, "Argument {}: expected {}", arg_index + 1, expected)
            }
            CliError::BufferFull => write!(f, "Buffer full"),
            CliError::PathTooDeep => write!(f, "Path too deep"),
            #[cfg(feature = "authentication")]
            CliError::AuthenticationFailed => write!(f, "Authentication failed"),
            #[cfg(feature = "authentication")]
            CliError::NotAuthenticated => write!(f, "Not authenticated"),
            CliError::IoError => write!(f, "I/O error"),
            #[cfg(feature = "async")]
            CliError::AsyncNotSupported => write!(f, "Async command not supported in sync mode"),
            CliError::Timeout => write!(f, "Operation timed out"),
            CliError::CommandFailed(msg) => write!(f, "{}", msg),
            CliError::Other(msg) => write!(f, "{}", msg),
        }
    }
}
```

**Security note**: `InvalidPath` is intentionally ambiguous - returns same error for non-existent paths and inaccessible paths to prevent revealing existence of restricted commands.

---

## Method Signatures

### Shell Core Methods

```rust
impl<'tree, L, IO, H, C> Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    /// Activate CLI and show welcome message
    /// Transitions from Inactive to LoggedOut (auth) or LoggedIn (no auth)
    pub fn activate(&mut self) -> Result<(), IO::Error> {
        // Show welcome, prompt
    }

    /// Process single character (main loop)
    /// Call repeatedly with characters from I/O
    pub fn process_char(&mut self, c: char) -> Result<(), IO::Error> {
        // Parse, execute, respond
    }

    /// Process single character (async version)
    /// Only available when `async` feature enabled
    #[cfg(feature = "async")]
    pub async fn process_char_async(&mut self, c: char) -> Result<(), IO::Error> {
        // Parse, execute (await async commands), respond
    }

    /// Resolve path to node with access control checks
    /// Returns Err(InvalidPath) for both non-existent and inaccessible nodes
    fn resolve_path(
        &self,
        path: &str,
        current_user: Option<&User<L>>,
    ) -> Result<&'tree Node<L>, CliError> {
        // Parse path, walk tree, check access at each segment
    }

    /// Execute command with argument validation
    fn execute_command(
        &mut self,
        cmd: &CommandMeta<L>,
        args: &[&str],
    ) -> Result<Response<C>, CliError> {
        // Validate args, dispatch to handlers
    }

    /// Execute command asynchronously (feature: async)
    #[cfg(feature = "async")]
    async fn execute_command_async(
        &mut self,
        cmd: &CommandMeta<L>,
        args: &[&str],
    ) -> Result<Response<C>, CliError> {
        // Validate args, dispatch to async handler
    }

    /// Generate prompt string
    /// Format: "username@/path/to/dir> " or "/path/to/dir> " (no auth)
    fn generate_prompt(&self) -> heapless::String<C::MAX_PROMPT> {
        // Build prompt from current_user and current_path
    }

    /// Handle "?" global command (list global commands)
    fn handle_help(&mut self) -> Result<Response<C>, CliError> {
        // List all global commands with descriptions
    }

    /// Handle "ls" global command (list current directory)
    fn handle_context_help(&mut self) -> Result<Response<C>, CliError> {
        // List children of current directory (with access filtering)
    }

    /// Handle "logout" global command (feature: authentication)
    #[cfg(feature = "authentication")]
    fn handle_logout(&mut self) -> Result<Response<C>, CliError> {
        // Clear current_user, transition to LoggedOut state
    }

    /// Send response to user via I/O
    fn send_response(&mut self, response: &Response<C>) -> Result<(), IO::Error> {
        // Format and send response based on flags
    }
}
```

### Path Navigation Methods

```rust
impl<L: AccessLevel> Directory<L> {
    /// Find direct child by name (no access control)
    pub fn find_child(&self, name: &str) -> Option<&Node<L>> {
        self.children.iter().find(|n| n.name() == name)
    }
}

// Path resolution is part of Shell (needs access control context)
impl<'tree, L, IO, H> Shell<'tree, L, IO, H> {
    /// Walk path from current directory
    /// Returns final node with access control enforcement
    fn resolve_path(
        &self,
        path: &str,
        current_user: Option<&User<L>>,
    ) -> Result<&'tree Node<L>, CliError> {
        // Implementation in Phase 4 + access control in Phase 8
    }
}
```

### Authentication Methods (feature: authentication)

```rust
#[cfg(feature = "authentication")]
impl<'tree, L, IO, H, C> Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    /// Attempt login with credentials
    fn handle_login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<Response<C>, CliError> {
        // Verify credentials, transition to LoggedIn
    }

    /// Logout current user
    fn handle_logout(&mut self) -> Result<Response<C>, CliError> {
        // Clear current_user, transition to LoggedOut
    }
}
```

---

## Usage Examples

### Creating a Complete Shell

```rust
use nut_shell::*;

// 1. Define access level
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum MyAccessLevel {
    Guest = 0,
    User = 1,
    Admin = 2,
}

impl AccessLevel for MyAccessLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Guest" => Some(Self::Guest),
            "User" => Some(Self::User),
            "Admin" => Some(Self::Admin),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Guest => "Guest",
            Self::User => "User",
            Self::Admin => "Admin",
        }
    }
}

// 2. Choose or define configuration
// Use DefaultConfig for typical systems, or create custom
type MyConfig = DefaultConfig;  // or MinimalConfig, or custom

// 3. Define command functions
fn reboot_fn<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    // Actual reboot logic here
    Ok(Response::success("Rebooting..."))
}

fn status_fn<C: ShellConfig>(_args: &[&str]) -> Result<Response<C>, CliError> {
    Ok(Response::success("System OK"))
}

// 4. Create command metadata
const REBOOT: CommandMeta<MyAccessLevel> = CommandMeta {
    name: "reboot",
    description: "Reboot the device",
    access_level: MyAccessLevel::Admin,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

const STATUS: CommandMeta<MyAccessLevel> = CommandMeta {
    name: "status",
    description: "Show system status",
    access_level: MyAccessLevel::User,
    kind: CommandKind::Sync,
    min_args: 0,
    max_args: 0,
};

// 5. Build tree
const SYSTEM_DIR: Directory<MyAccessLevel> = Directory {
    name: "system",
    children: &[
        Node::Command(&REBOOT),
        Node::Command(&STATUS),
    ],
    access_level: MyAccessLevel::User,
};

const ROOT: Directory<MyAccessLevel> = Directory {
    name: "/",
    children: &[
        Node::Directory(&SYSTEM_DIR),
    ],
    access_level: MyAccessLevel::Guest,
};

// 6. Implement handlers
struct MyHandlers;

impl CommandHandlers<MyConfig> for MyHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<MyConfig>, CliError> {
        match name {
            "reboot" => reboot_fn::<MyConfig>(args),
            "status" => status_fn::<MyConfig>(args),
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// 7. Create Shell
fn main() {
    let handlers = MyHandlers;
    let io = MyIo::new(); // Platform-specific

    // Type annotation helps inference
    let mut shell: Shell<_, _, _, _, MyConfig> = Shell::new(&ROOT, handlers, io);

    shell.activate().unwrap();

    loop {
        if let Some(c) = io.get_char().unwrap() {
            shell.process_char(c).unwrap();
        }
    }
}
```

**Custom configuration example**:
```rust
// Define custom config for your application
struct HighCapacityConfig;

impl ShellConfig for HighCapacityConfig {
    const MAX_INPUT: usize = 512;
    const MAX_PATH_DEPTH: usize = 16;
    const MAX_ARGS: usize = 64;
    const MAX_PROMPT: usize = 128;
    const MAX_RESPONSE: usize = 1024;
    const HISTORY_SIZE: usize = 50;
}

// Use in handlers and shell
type MyConfig = HighCapacityConfig;

impl CommandHandlers<MyConfig> for MyHandlers {
    // ... implementation
}

let mut shell: Shell<_, _, _, _, MyConfig> = Shell::new(&ROOT, handlers, io);
```

---

## Notes

- **All types are `no_std` compatible** - Use `heapless` for dynamic allocation
- **Const initialization** - Tree types must be const-initializable
- **Feature gates** - Some fields/methods only available with features enabled
- **Lifetimes** - `'tree` lifetime ensures tree outlives Shell
- **Zero-cost abstractions** - Generics and enums compiled away via monomorphization

For implementation guidance, see [IMPLEMENTATION.md](IMPLEMENTATION.md).
