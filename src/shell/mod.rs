//! Shell orchestration and command processing.
//!
//! The `Shell` struct brings together all components to provide interactive CLI functionality.
//! See [DESIGN.md](../../docs/DESIGN.md) for unified architecture pattern.

use crate::auth::{AccessLevel, User};
use crate::config::ShellConfig;
use crate::error::CliError;
use crate::io::CharIo;
use crate::response::Response;
use crate::tree::{CommandKind, Directory, Node};
use core::marker::PhantomData;

// Sub-modules
pub mod handlers;
pub mod history;
pub mod decoder;

// Re-export key types
pub use handlers::CommandHandlers;
pub use history::CommandHistory;
pub use decoder::{InputDecoder, InputEvent};

/// History navigation direction.
///
/// Used by `Request::History` variant. Self-documenting alternative to bool.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HistoryDirection {
    /// Up arrow key (navigate to older command)
    Previous = 0,

    /// Down arrow key (navigate to newer command or restore original)
    Next = 1,
}

/// CLI state (authentication state).
///
/// Tracks whether the CLI is active and whether user is authenticated.
/// Used by unified architecture pattern to drive behavior.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CliState {
    /// CLI not active
    Inactive,

    /// Awaiting authentication
    #[cfg(feature = "authentication")]
    LoggedOut,

    /// Authenticated or auth-disabled mode
    LoggedIn,
}

/// Request type representing parsed user input.
///
/// Generic over `C: ShellConfig` to use configured buffer sizes.
/// Variants are feature-gated based on available features.
///
/// See [TYPE_REFERENCE.md](../../docs/TYPE_REFERENCE.md) for complete type definition.
#[derive(Debug, Clone)]
pub enum Request<C: ShellConfig> {
    /// Valid authentication attempt
    #[cfg(feature = "authentication")]
    Login {
        /// Username
        username: heapless::String<32>,
        /// Password
        password: heapless::String<64>,
    },

    /// Invalid authentication attempt
    #[cfg(feature = "authentication")]
    InvalidLogin,

    /// Execute command
    Command {
        /// Command path
        path: heapless::String<128>, // TODO: Use C::MAX_INPUT when const generics stabilize
        /// Command arguments
        args: heapless::Vec<heapless::String<128>, 16>, // TODO: Use C::MAX_INPUT and C::MAX_ARGS
        /// Original command string for history
        #[cfg(feature = "history")]
        original: heapless::String<128>, // TODO: Use C::MAX_INPUT
        /// Phantom data for config type (will be used when const generics stabilize)
        _phantom: PhantomData<C>,
    },

    /// Request completions
    #[cfg(feature = "completion")]
    TabComplete {
        /// Partial path to complete
        path: heapless::String<128>, // TODO: Use C::MAX_INPUT
    },

    /// Navigate history
    #[cfg(feature = "history")]
    History {
        /// Navigation direction
        direction: HistoryDirection,
        /// Current buffer content
        buffer: heapless::String<128>, // TODO: Use C::MAX_INPUT
    },
}

/// Shell orchestration struct.
///
/// Brings together all components following the unified architecture pattern.
/// Uses single code path for both auth-enabled and auth-disabled modes.
///
/// Generic over:
/// - `'tree`: Lifetime of command tree (typically 'static)
/// - `L`: AccessLevel implementation
/// - `IO`: CharIo implementation
/// - `H`: CommandHandlers implementation
/// - `C`: ShellConfig implementation
///
/// See [DESIGN.md](../../docs/DESIGN.md) for unified architecture pattern.
pub struct Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    /// Command tree root
    tree: &'tree Directory<L>,

    /// Current user (None when logged out or auth disabled)
    current_user: Option<User<L>>,

    /// CLI state (auth state)
    state: CliState,

    /// Input buffer (using concrete size for now - TODO: use C::MAX_INPUT when const generics stabilize)
    input_buffer: heapless::String<128>,

    /// Current directory path (stack of child indices, using concrete size - TODO: use C::MAX_PATH_DEPTH when const generics stabilize)
    current_path: heapless::Vec<usize, 8>,

    /// Input decoder (escape sequence state machine)
    decoder: InputDecoder,

    /// Command history (using concrete sizes - TODO: use C::HISTORY_SIZE and C::MAX_INPUT when const generics stabilize)
    #[cfg_attr(not(feature = "history"), allow(dead_code))]
    history: CommandHistory<10, 128>,

    /// I/O interface
    io: IO,

    /// Command handlers
    handlers: H,

    /// Credential provider
    #[cfg(feature = "authentication")]
    credential_provider: &'tree (dyn crate::auth::CredentialProvider<L, Error = ()> + 'tree),

    /// Config type marker (zero-size)
    _config: PhantomData<C>,
}

// ============================================================================
// Debug implementation
// ============================================================================

impl<'tree, L, IO, H, C> core::fmt::Debug for Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug_struct = f.debug_struct("Shell");
        debug_struct
            .field("state", &self.state)
            .field("input_buffer", &self.input_buffer.as_str())
            .field("current_path", &self.current_path);

        if let Some(user) = &self.current_user {
            debug_struct.field("current_user", &user.username.as_str());
        } else {
            debug_struct.field("current_user", &"None");
        }

        #[cfg(feature = "authentication")]
        debug_struct.field("credential_provider", &"<dyn CredentialProvider>");

        debug_struct.finish_non_exhaustive()
    }
}

// ============================================================================
// Constructors (feature-conditional)
// ============================================================================

#[cfg(feature = "authentication")]
impl<'tree, L, IO, H, C> Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    /// Create new Shell with credential provider for when authentication enabled.
    ///
    /// Starts in `Inactive` state. Call `activate()` to show welcome message and prompt.
    pub fn new(
        tree: &'tree Directory<L>,
        handlers: H,
        credential_provider: &'tree (dyn crate::auth::CredentialProvider<L, Error = ()> + 'tree),
        io: IO,
    ) -> Self {
        Self {
            tree,
            handlers,
            current_user: None,
            state: CliState::Inactive,
            input_buffer: heapless::String::new(),
            current_path: heapless::Vec::new(),
            decoder: InputDecoder::new(),
            history: CommandHistory::new(),
            io,
            credential_provider,
            _config: PhantomData,
        }
    }
}

#[cfg(not(feature = "authentication"))]
impl<'tree, L, IO, H, C> Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    /// Create new Shell
    ///
    /// Starts in `Inactive` state. Call `activate()` to show welcome message and prompt.
    pub fn new(tree: &'tree Directory<L>, handlers: H, io: IO) -> Self {
        Self {
            tree,
            handlers,
            current_user: None,
            state: CliState::Inactive,
            input_buffer: heapless::String::new(),
            current_path: heapless::Vec::new(),
            decoder: InputDecoder::new(),
            history: CommandHistory::new(),
            io,
            _config: PhantomData,
        }
    }
}

// ============================================================================
// Core methods (unified implementation for both modes)
// ============================================================================

impl<'tree, L, IO, H, C> Shell<'tree, L, IO, H, C>
where
    L: AccessLevel,
    IO: CharIo,
    H: CommandHandlers<C>,
    C: ShellConfig,
{
    /// Activate the shell (show welcome message and initial prompt).
    ///
    /// Transitions from `Inactive` to appropriate state (LoggedOut or LoggedIn).
    pub fn activate(&mut self) -> Result<(), IO::Error> {
        #[cfg(feature = "authentication")]
        {
            self.state = CliState::LoggedOut;
            self.io.write_str(C::MSG_WELCOME_AUTH)?;
            self.io.write_str(C::MSG_LOGIN_PROMPT)?;
        }

        #[cfg(not(feature = "authentication"))]
        {
            self.state = CliState::LoggedIn;
            self.io.write_str(C::MSG_WELCOME_NO_AUTH)?;
            self.generate_and_write_prompt()?;
        }

        Ok(())
    }

    /// Deactivate the shell (transition to Inactive state).
    ///
    /// Clears user session, input buffer, and returns to root directory.
    /// The shell will ignore all input until `activate()` is called again.
    ///
    /// This is useful for:
    /// - Clean shutdown sequences
    /// - Temporarily suspending the shell
    /// - Resetting to initial state
    pub fn deactivate(&mut self) {
        self.state = CliState::Inactive;
        self.current_user = None;
        self.input_buffer.clear();
        self.current_path.clear();
    }

    /// Process a single character of input.
    ///
    /// Main entry point for character-by-character processing.
    /// Returns Ok(()) on success, Err on I/O error.
    pub fn process_char(&mut self, c: char) -> Result<(), IO::Error> {
        // Decode character into logical event
        let event = self.decoder.decode_char(c);

        match event {
            InputEvent::None => Ok(()), // Still accumulating sequence

            InputEvent::Char(ch) => {
                // Try to add to buffer
                match self.input_buffer.push(ch) {
                    Ok(_) => {
                        // Successfully added - echo (with password masking if applicable)
                        let echo_char = self.get_echo_char(ch);
                        self.io.put_char(echo_char)?;
                        Ok(())
                    }
                    Err(_) => {
                        // Buffer full - beep and ignore
                        self.io.put_char('\x07')?; // Bell character
                        Ok(())
                    }
                }
            }

            InputEvent::Backspace => {
                // Remove from buffer if not empty
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                    // Echo backspace sequence
                    self.io.write_str("\x08 \x08")?;
                }
                Ok(())
            }

            InputEvent::DoubleEsc => {
                // Clear buffer and redraw (Shell's interpretation of double-ESC)
                self.input_buffer.clear();
                self.clear_line_and_redraw()
            }

            InputEvent::Enter => self.handle_enter(),

            InputEvent::Tab => self.handle_tab(),

            InputEvent::UpArrow => self.handle_history(HistoryDirection::Previous),

            InputEvent::DownArrow => self.handle_history(HistoryDirection::Next),
        }
    }

    /// Poll for incoming characters and process them.
    ///
    /// This is a **convenience method** for simple polling loops where the Shell actively
    /// reads from its I/O. For more control or better embedded patterns, use
    /// [`process_char()`](Self::process_char) directly.
    ///
    /// # When to Use
    ///
    /// Use `poll()` for:
    /// - Simple blocking UART in bare-metal systems
    /// - Quick prototypes and examples
    /// - Native applications with blocking stdio
    ///
    /// # When NOT to Use
    ///
    /// **Do not use `poll()` if you need:**
    /// - **Interrupt-driven UART**: Read characters in an interrupt handler and buffer them,
    ///   then call `process_char()` from your main loop
    /// - **DMA-based I/O**: Use DMA circular buffers and call `process_char()` for each
    ///   character from the buffer
    /// - **Async/await patterns**: Use `process_char()` from within your async context
    /// - **RTOS integration**: Read from RTOS queues and call `process_char()`
    /// - **Low power modes**: Waking from sleep to read requires interrupt-based approach
    ///
    /// In these cases, your application should control **when** and **how** characters
    /// are read, then feed them to the Shell via `process_char()`.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if no character available or character processed successfully
    /// - `Err` on I/O error
    pub fn poll(&mut self) -> Result<(), IO::Error> {
        if let Some(c) = self.io.get_char()? {
            self.process_char(c)?;
        }
        Ok(())
    }

    /// Determine what character to echo based on password masking rules.
    ///
    /// When in LoggedOut state (login prompt), characters after the `:` delimiter
    /// are masked with `*` for password privacy.
    ///
    /// # Masking Rules
    ///
    /// - Characters before first `:` are echoed normally (username)
    /// - The first `:` character is echoed normally (delimiter)
    /// - All characters after `:` are echoed as `*` (password)
    ///
    /// # Arguments
    ///
    /// * `ch` - The character that was just added to the input buffer
    ///
    /// # Returns
    ///
    /// The character to echo to the terminal (`*` for masked, or original char)
    fn get_echo_char(&self, ch: char) -> char {
        #[cfg(feature = "authentication")]
        {
            // Password masking only applies during login (LoggedOut state)
            if self.state == CliState::LoggedOut {
                // Count colons in buffer (parser has already added current char)
                let colon_count = self.input_buffer.matches(':').count();

                // Logic: Mask if buffer had at least one colon before this character
                // - colon_count == 0: No delimiter yet, echo normally
                // - colon_count == 1 && ch == ':': First colon (just added), echo normally
                // - Otherwise: We're in password territory, mask it
                if colon_count == 0 || (colon_count == 1 && ch == ':') {
                    return ch; // Username or delimiter
                } else {
                    return '*'; // Password
                }
            }
        }

        // Default: echo character as-is
        ch
    }

    /// Generate prompt string.
    ///
    /// Format: `username@path> ` (or `@path> ` when no user/auth disabled)
    // TODO: Use C::MAX_PROMPT when const generics stabilize
    fn generate_prompt(&self) -> heapless::String<128> {
        let mut prompt = heapless::String::new();

        // Username part
        if let Some(user) = &self.current_user {
            prompt.push_str(user.username.as_str()).ok();
        }
        prompt.push('@').ok();

        // Path part
        prompt.push('/').ok();
        if !self.current_path.is_empty() {
            if let Ok(path_str) = self.get_current_path_string() {
                prompt.push_str(&path_str).ok();
            }
        }

        prompt.push_str("> ").ok();
        prompt
    }

    /// Write prompt to I/O.
    fn generate_and_write_prompt(&mut self) -> Result<(), IO::Error> {
        let prompt = self.generate_prompt();
        self.io.write_str(prompt.as_str())
    }

    /// Write formatted response to I/O, applying all Response formatting flags.
    ///
    /// This is the key method that makes Response flags functional.
    /// It interprets and applies:
    /// - `prefix_newline`: Adds blank line before message
    /// - `indent_message`: Indents all lines with 2 spaces
    /// - `postfix_newline`: Adds newline after message
    ///
    /// Note: `inline_message` is handled by the caller (handle_input_line at line 655-658)
    /// and `show_prompt` is handled by the caller (handle_input_line at line 670-672).
    fn write_formatted_response(&mut self, response: &Response<C>) -> Result<(), IO::Error> {
        // Prefix newline (blank line before output)
        if response.prefix_newline {
            self.io.write_str("\r\n")?;
        }

        // Write message (with optional indentation)
        if response.indent_message {
            // Split by lines and indent each
            for (i, line) in response.message.split("\r\n").enumerate() {
                if i > 0 {
                    self.io.write_str("\r\n")?;
                }
                self.io.write_str("  ")?; // 2-space indent
                self.io.write_str(line)?;
            }
        } else {
            // Write message as-is
            self.io.write_str(&response.message)?;
        }

        // Postfix newline
        if response.postfix_newline {
            self.io.write_str("\r\n")?;
        }

        Ok(())
    }

    /// Get current directory node.
    fn get_current_dir(&self) -> Result<&'tree Directory<L>, CliError> {
        let mut current: &Directory<L> = self.tree;

        for &index in self.current_path.iter() {
            match current.children.get(index) {
                Some(Node::Directory(dir)) => current = dir,
                Some(Node::Command(_)) | None => return Err(CliError::InvalidPath),
            }
        }

        Ok(current)
    }

    /// Get current path as string (for prompt).
    // TODO: Use C::MAX_INPUT when const generics stabilize
    fn get_current_path_string(&self) -> Result<heapless::String<128>, CliError> {
        let mut path_str = heapless::String::new();
        let mut current: &Directory<L> = self.tree;

        for (i, &index) in self.current_path.iter().enumerate() {
            match current.children.get(index) {
                Some(Node::Directory(dir)) => {
                    if i > 0 {
                        path_str.push('/').map_err(|_| CliError::BufferFull)?;
                    }
                    path_str
                        .push_str(dir.name)
                        .map_err(|_| CliError::BufferFull)?;
                    current = dir;
                }
                _ => return Err(CliError::InvalidPath),
            }
        }

        Ok(path_str)
    }

    /// Handle Enter key (submit command or login).
    fn handle_enter(&mut self) -> Result<(), IO::Error> {
        // Note: Newline after input is written by the handlers
        // (conditionally based on Response.inline_message flag for commands)

        let input = self.input_buffer.clone();
        self.input_buffer.clear();

        match self.state {
            CliState::Inactive => Ok(()),

            #[cfg(feature = "authentication")]
            CliState::LoggedOut => self.handle_login_input(&input),

            CliState::LoggedIn => self.handle_input_line(&input),
        }
    }

    /// Handle a valid login attempt.
    #[cfg(feature = "authentication")]
    fn handle_login_input(&mut self, input: &str) -> Result<(), IO::Error> {
        // Login doesn't support inline mode - always add newline
        self.io.write_str("\r\n")?;

        if input.contains(':') {
            // Format: username:password
            let parts: heapless::Vec<&str, 2> = input.splitn(2, ':').collect();
            if parts.len() == 2 {
                let username = parts[0];
                let password = parts[1];

                // Attempt authentication
                match self.credential_provider.find_user(username) {
                    Ok(Some(user)) if self.credential_provider.verify_password(&user, password) => {
                        // Login successful
                        self.current_user = Some(user);
                        self.state = CliState::LoggedIn;
                        self.io.write_str(C::MSG_LOGIN_SUCCESS)?;
                        self.generate_and_write_prompt()?;
                    }
                    _ => {
                        // Login failed (user not found or wrong password)
                        self.io.write_str(C::MSG_LOGIN_FAILED)?;
                        self.io.write_str(C::MSG_LOGIN_PROMPT)?;
                    }
                }
            } else {
                self.io.write_str(C::MSG_INVALID_LOGIN_FORMAT)?;
                self.io.write_str(C::MSG_LOGIN_PROMPT)?;
            }
        } else {
            // No colon - invalid format, show error
            self.io.write_str(C::MSG_INVALID_LOGIN_FORMAT)?;
            self.io.write_str(C::MSG_LOGIN_PROMPT)?;
        }

        Ok(())
    }

    /// Handle user input line when in LoggedIn state.
    ///
    /// Processes three types of input:
    /// 1. Global commands (?, ls, clear, logout)
    /// 2. Tree navigation (paths resolving to directories)
    /// 3. Tree commands (paths resolving to Node::Command)
    fn handle_input_line(&mut self, input: &str) -> Result<(), IO::Error> {
        // Skip empty input
        if input.trim().is_empty() {
            self.io.write_str("\r\n")?;
            self.generate_and_write_prompt()?;
            return Ok(());
        }

        // Check for global commands first (non-tree operations)
        // Global commands don't support inline mode
        match input.trim() {
            "?" => {
                self.io.write_str("\r\n")?;
                self.show_help()?;
                self.generate_and_write_prompt()?;
                return Ok(());
            }
            "ls" => {
                self.io.write_str("\r\n")?;
                self.show_ls()?;
                self.generate_and_write_prompt()?;
                return Ok(());
            }
            "clear" => {
                // Clear screen - no newline needed before ANSI clear sequence
                self.io.write_str("\x1b[2J\x1b[H")?; // ANSI clear screen
                self.generate_and_write_prompt()?;
                return Ok(());
            }
            #[cfg(feature = "authentication")]
            "logout" => {
                self.io.write_str("\r\n")?;
                self.current_user = None;
                self.state = CliState::LoggedOut;
                self.current_path.clear();
                self.io.write_str(C::MSG_LOGOUT)?;
                self.io.write_str(C::MSG_LOGIN_PROMPT)?;
                return Ok(());
            }
            _ => {}
        }

        // Handle tree operations (navigation or command execution)
        match self.execute_tree_path(input) {
            Ok(response) => {
                // Add newline after input UNLESS response wants inline mode
                if !response.inline_message {
                    self.io.write_str("\r\n")?;
                }

                // Write formatted response (implements all Response flags!)
                self.write_formatted_response(&response)?;

                // Add to history if not excluded
                #[cfg(feature = "history")]
                if !response.exclude_from_history {
                    self.history.add(input);
                }

                // Show prompt if requested by response
                if response.show_prompt {
                    self.generate_and_write_prompt()?;
                }
            }
            Err(e) => {
                // Errors don't support inline mode - add newline
                self.io.write_str("\r\n")?;

                // Write error message
                self.io.write_str("Error: ")?;
                match e {
                    CliError::CommandNotFound => self.io.write_str("Command not found")?,
                    CliError::InvalidPath => self.io.write_str("Invalid path")?,
                    CliError::InvalidArgumentCount { .. } => {
                        // TODO: Format detailed error message when write! macro available in no_std
                        self.io.write_str("Invalid argument count")?;
                    }
                    CliError::InvalidArgumentFormat { .. } => {
                        self.io.write_str("Invalid argument format")?
                    }
                    CliError::BufferFull => self.io.write_str("Buffer full")?,
                    CliError::PathTooDeep => self.io.write_str("Path too deep")?,
                    #[cfg(feature = "async")]
                    CliError::AsyncNotSupported => {
                        self.io
                            .write_str("Async command requires process_char_async()")?
                    }
                    _ => self.io.write_str("Unknown error")?,
                }
                self.io.write_str("\r\n")?;
                self.generate_and_write_prompt()?;
            }
        }

        Ok(())
    }

    /// Execute a tree path (navigation or command execution).
    ///
    /// Resolves the path and either:
    /// - Navigates to a directory (if path resolves to Node::Directory)
    /// - Executes a tree command (if path resolves to Node::Command)
    ///
    /// Note: "command" here refers specifically to Node::Command,
    /// not generic user input.
    fn execute_tree_path(&mut self, input: &str) -> Result<Response<C>, CliError> {
        // Parse path and arguments
        // TODO: Use C::MAX_ARGS + 1 when const generics stabilize (command + args)
        let parts: heapless::Vec<&str, 17> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err(CliError::CommandNotFound);
        }

        let path_str = parts[0];
        let args = &parts[1..];

        // Resolve path to node
        let (target_node, new_path) = self.resolve_path(path_str)?;

        // Case 1: Directory navigation
        if let Node::Directory(_) = target_node {
            self.current_path = new_path;
            #[cfg(feature = "history")]
            return Ok(Response::success("").without_history());
            #[cfg(not(feature = "history"))]
            return Ok(Response::success(""));
        }

        // Case 2: Tree command execution (Node::Command)
        if let Node::Command(cmd_meta) = target_node {
            // Check access control - use InvalidPath for security (don't reveal access denied)
            if let Some(user) = &self.current_user {
                if user.access_level < cmd_meta.access_level {
                    return Err(CliError::InvalidPath);
                }
            }

            // Validate argument count
            if args.len() < cmd_meta.min_args || args.len() > cmd_meta.max_args {
                return Err(CliError::InvalidArgumentCount {
                    expected_min: cmd_meta.min_args,
                    expected_max: cmd_meta.max_args,
                    received: args.len(),
                });
            }

            // Dispatch to command handlers
            match cmd_meta.kind {
                CommandKind::Sync => {
                    // Execute synchronous tree command
                    self.handlers.execute_sync(cmd_meta.name, args)
                }
                #[cfg(feature = "async")]
                CommandKind::Async => {
                    // Async tree command called from sync context
                    Err(CliError::AsyncNotSupported)
                }
            }
        } else {
            Err(CliError::CommandNotFound)
        }
    }

    /// Resolve a path string to a node.
    ///
    /// Returns (node, path_stack) where path_stack is the navigation path.
    // TODO: Use C::MAX_PATH_DEPTH when const generics stabilize
    fn resolve_path(
        &self,
        path_str: &str,
    ) -> Result<(&'tree Node<L>, heapless::Vec<usize, 8>), CliError> {
        // Start from current directory or root
        // TODO: Use C::MAX_PATH_DEPTH when const generics stabilize
        let mut working_path: heapless::Vec<usize, 8> = if path_str.starts_with('/') {
            heapless::Vec::new() // Absolute path starts from root
        } else {
            self.current_path.clone() // Relative path starts from current
        };

        // Parse path
        // TODO: Use C::MAX_PATH_DEPTH when const generics stabilize
        let segments: heapless::Vec<&str, 8> = path_str
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .collect();

        // Navigate through segments
        for segment in segments.iter() {
            if *segment == ".." {
                // Parent directory
                working_path.pop();
                continue;
            }

            // Find child with this name
            let current_dir = self.get_dir_at_path(&working_path)?;
            let mut found = false;

            for (index, child) in current_dir.children.iter().enumerate() {
                // Check access control
                let node_level = match child {
                    Node::Command(cmd) => cmd.access_level,
                    Node::Directory(dir) => dir.access_level,
                };

                if let Some(user) = &self.current_user {
                    if user.access_level < node_level {
                        continue; // User lacks access, skip this node
                    }
                }

                if child.name() == *segment {
                    // Found it!
                    if child.is_directory() {
                        // Navigate into directory
                        working_path
                            .push(index)
                            .map_err(|_| CliError::PathTooDeep)?;
                    } else {
                        // It's a command - return it
                        return Ok((child, working_path));
                    }
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(CliError::CommandNotFound);
            }
        }

        // Path resolved to a directory
        let dir_node = self.get_node_at_path(&working_path)?;
        Ok((dir_node, working_path))
    }

    /// Get directory at specific path.
    // TODO: Use C::MAX_PATH_DEPTH when const generics stabilize
    fn get_dir_at_path(&self, path: &heapless::Vec<usize, 8>) -> Result<&'tree Directory<L>, CliError> {
        let mut current: &Directory<L> = self.tree;

        for &index in path.iter() {
            match current.children.get(index) {
                Some(Node::Directory(dir)) => current = dir,
                Some(Node::Command(_)) | None => return Err(CliError::InvalidPath),
            }
        }

        Ok(current)
    }

    /// Get node at specific path.
    // TODO: Use C::MAX_PATH_DEPTH when const generics stabilize
    fn get_node_at_path(&self, path: &heapless::Vec<usize, 8>) -> Result<&'tree Node<L>, CliError> {
        if path.is_empty() {
            // Root directory - need to find a way to return it as a Node
            // For now, return error since we can't construct Node::Directory here
            return Err(CliError::InvalidPath);
        }

        // TODO: Use C::MAX_PATH_DEPTH when const generics stabilize
        let parent_path: heapless::Vec<usize, 8> = path.iter().take(path.len() - 1).copied().collect();
        let parent_dir = self.get_dir_at_path(&parent_path)?;

        let last_index = *path.last().ok_or(CliError::InvalidPath)?;
        parent_dir
            .children
            .get(last_index)
            .ok_or(CliError::InvalidPath)
    }

    /// Handle Tab completion.
    fn handle_tab(&mut self) -> Result<(), IO::Error> {
        #[cfg(feature = "completion")]
        {
            // Get current directory
            let current_dir = match self.get_current_dir() {
                Ok(dir) => dir,
                Err(_) => return self.generate_and_write_prompt(), // Error, just redraw prompt
            };

            // Suggest completions
            let result = crate::tree::completion::suggest_completions::<L, 16>(
                current_dir,
                self.input_buffer.as_str(),
                self.current_user.as_ref(),
            );

            match result {
                Ok(completion) if completion.is_complete => {
                    // Single match - replace buffer and update display
                    self.input_buffer.clear();
                    match self.input_buffer.push_str(&completion.completion) {
                        Ok(()) => {
                            // Redraw line
                            self.io.write_str("\r")?; // Carriage return
                            let prompt = self.generate_prompt();
                            self.io.write_str(prompt.as_str())?;
                            self.io.write_str(self.input_buffer.as_str())?;
                        }
                        Err(_) => {
                            // Completion too long for buffer - beep
                            self.io.put_char('\x07')?;
                        }
                    }
                }
                Ok(completion) if !completion.all_matches.is_empty() => {
                    // Multiple matches - show them
                    self.io.write_str("\r\n")?;
                    for m in completion.all_matches.iter() {
                        self.io.write_str(m.as_str())?;
                        self.io.write_str("  ")?;
                    }
                    self.io.write_str("\r\n")?;
                    self.generate_and_write_prompt()?;
                    self.io.write_str(self.input_buffer.as_str())?;
                }
                _ => {
                    // No matches or error - just beep
                    self.io.put_char('\x07')?; // Bell character
                }
            }
        }

        #[cfg(not(feature = "completion"))]
        {
            // Completion disabled - just beep
            self.io.put_char('\x07')?; // Bell character
        }

        Ok(())
    }

    /// Handle history navigation.
    fn handle_history(&mut self, direction: HistoryDirection) -> Result<(), IO::Error> {
        #[cfg(feature = "history")]
        {
            let history_entry = match direction {
                HistoryDirection::Previous => self.history.previous(),
                HistoryDirection::Next => self.history.next(),
            };

            if let Some(entry) = history_entry {
                // Replace buffer with history entry
                self.input_buffer = entry;
                // Redraw line
                self.clear_line_and_redraw()?;
            }
        }

        #[cfg(not(feature = "history"))]
        {
            // History disabled - ignore
            let _ = direction; // Silence unused warning
        }

        Ok(())
    }

    /// Show help (? command).
    fn show_help(&mut self) -> Result<(), IO::Error> {
        self.io.write_str("Global commands:\r\n")?;
        self.io.write_str("  ?         - Show this help\r\n")?;
        self.io.write_str("  ls        - List directory contents\r\n")?;

        #[cfg(feature = "authentication")]
        self.io.write_str("  logout    - End session\r\n")?;

        self.io.write_str("  clear     - Clear screen\r\n")?;
        self.io.write_str("  ESC ESC   - Clear input buffer\r\n")?;

        Ok(())
    }

    /// Show directory listing (ls command).
    fn show_ls(&mut self) -> Result<(), IO::Error> {
        let current_dir = match self.get_current_dir() {
            Ok(dir) => dir,
            Err(_) => {
                self.io.write_str("Error accessing directory\r\n")?;
                return Ok(());
            }
        };

        for child in current_dir.children.iter() {
            // Check access control
            let node_level = match child {
                Node::Command(cmd) => cmd.access_level,
                Node::Directory(dir) => dir.access_level,
            };

            if let Some(user) = &self.current_user {
                if user.access_level < node_level {
                    continue; // User lacks access, skip this node
                }
            }

            // Format output
            match child {
                Node::Command(cmd) => {
                    self.io.write_str("  ")?;
                    self.io.write_str(cmd.name)?;
                    self.io.write_str("  - ")?;
                    self.io.write_str(cmd.description)?;
                    self.io.write_str("\r\n")?;
                }
                Node::Directory(dir) => {
                    self.io.write_str("  ")?;
                    self.io.write_str(dir.name)?;
                    self.io.write_str("/  - Directory\r\n")?;
                }
            }
        }

        Ok(())
    }

    /// Clear current line and redraw with prompt and buffer.
    fn clear_line_and_redraw(&mut self) -> Result<(), IO::Error> {
        self.io.write_str("\r\x1b[K")?; // CR + clear to end of line
        self.generate_and_write_prompt()?;
        self.io.write_str(self.input_buffer.as_str())?;
        Ok(())
    }

    // ========================================
    // Test-only accessors
    // ========================================

    /// Get reference to I/O interface (test-only).
    ///
    /// Available in both unit tests and integration tests.
    #[doc(hidden)]
    pub fn __test_io(&self) -> &IO {
        &self.io
    }

    /// Get mutable reference to I/O interface (test-only).
    ///
    /// Available in both unit tests and integration tests.
    #[doc(hidden)]
    pub fn __test_io_mut(&mut self) -> &mut IO {
        &mut self.io
    }

    /// Get reference to input buffer (test-only).
    ///
    /// Available in both unit tests and integration tests.
    #[doc(hidden)]
    pub fn __test_get_input_buffer(&self) -> &str {
        self.input_buffer.as_str()
    }

    /// Set authenticated user (test-only, requires authentication feature).
    ///
    /// Allows tests to manually set authentication state.
    #[doc(hidden)]
    #[cfg(feature = "authentication")]
    pub fn __test_set_authenticated_user(&mut self, user: Option<User<L>>) -> Result<(), CliError> {
        let is_some = user.is_some();
        self.current_user = user;
        if is_some {
            self.state = CliState::LoggedIn;
        } else {
            self.state = CliState::LoggedOut;
        }
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AccessLevel;
    use crate::config::DefaultConfig;
    use crate::io::CharIo;
    use crate::tree::Directory;

    // Mock access level
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum MockLevel {
        User = 0,
    }
    impl AccessLevel for MockLevel {
        fn from_str(s: &str) -> Option<Self> {
            match s {
                "User" => Some(Self::User),
                _ => None,
            }
        }
        fn as_str(&self) -> &'static str {
            "User"
        }
    }

    // Mock I/O that captures output
    struct MockIo {
        output: heapless::String<512>,
    }
    impl MockIo {
        fn new() -> Self {
            Self {
                output: heapless::String::new(),
            }
        }
        fn get_output(&self) -> &str {
            &self.output
        }
    }
    impl CharIo for MockIo {
        type Error = ();
        fn get_char(&mut self) -> Result<Option<char>, ()> {
            Ok(None)
        }
        fn put_char(&mut self, c: char) -> Result<(), ()> {
            self.output.push(c).map_err(|_| ())
        }
        fn write_str(&mut self, s: &str) -> Result<(), ()> {
            self.output.push_str(s).map_err(|_| ())
        }
    }

    // Mock handlers
    struct MockHandlers;
    impl CommandHandlers<DefaultConfig> for MockHandlers {
        fn execute_sync(
            &self,
            _name: &str,
            _args: &[&str],
        ) -> Result<crate::response::Response<DefaultConfig>, crate::error::CliError> {
            Err(crate::error::CliError::CommandNotFound)
        }

        #[cfg(feature = "async")]
        async fn execute_async(
            &self,
            _name: &str,
            _args: &[&str],
        ) -> Result<crate::response::Response<DefaultConfig>, crate::error::CliError> {
            Err(crate::error::CliError::CommandNotFound)
        }
    }

    // Test tree
    const TEST_TREE: Directory<MockLevel> = Directory {
        name: "/",
        children: &[],
        access_level: MockLevel::User,
    };

    #[test]
    fn test_history_direction() {
        assert_eq!(HistoryDirection::Previous as u8, 0);
        assert_eq!(HistoryDirection::Next as u8, 1);
    }

    #[test]
    fn test_cli_state() {
        assert_eq!(CliState::Inactive, CliState::Inactive);
        assert_eq!(CliState::LoggedIn, CliState::LoggedIn);

        #[cfg(feature = "authentication")]
        assert_ne!(CliState::LoggedOut, CliState::LoggedIn);
    }

    #[test]
    fn test_activate_deactivate_lifecycle() {
        let io = MockIo::new();
        let handlers = MockHandlers;

        // Create shell - should start in Inactive state
        #[cfg(feature = "authentication")]
        {
            use crate::auth::CredentialProvider;
            struct MockProvider;
            impl CredentialProvider<MockLevel> for MockProvider {
                type Error = ();
                fn find_user(&self, _username: &str) -> Result<Option<crate::auth::User<MockLevel>>, ()> {
                    Ok(None)
                }
                fn verify_password(&self, _user: &crate::auth::User<MockLevel>, _password: &str) -> bool {
                    false
                }
                fn list_users(&self) -> Result<heapless::Vec<&str, 32>, ()> {
                    Ok(heapless::Vec::new())
                }
            }
            let provider = MockProvider;
            let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
                Shell::new(&TEST_TREE, handlers, &provider, io);

            // Should start in Inactive state
            assert_eq!(shell.state, CliState::Inactive);
            assert!(shell.current_user.is_none());

            // Activate should transition to LoggedOut (auth enabled)
            shell.activate().unwrap();
            assert_eq!(shell.state, CliState::LoggedOut);

            // Deactivate should return to Inactive
            shell.deactivate();
            assert_eq!(shell.state, CliState::Inactive);
            assert!(shell.current_user.is_none());
            assert!(shell.input_buffer.is_empty());
            assert!(shell.current_path.is_empty());
        }

        #[cfg(not(feature = "authentication"))]
        {
            let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
                Shell::new(&TEST_TREE, handlers, io);

            // Should start in Inactive state
            assert_eq!(shell.state, CliState::Inactive);

            // Activate should transition to LoggedIn (auth disabled)
            shell.activate().unwrap();
            assert_eq!(shell.state, CliState::LoggedIn);

            // Deactivate should return to Inactive
            shell.deactivate();
            assert_eq!(shell.state, CliState::Inactive);
            assert!(shell.current_user.is_none());
            assert!(shell.input_buffer.is_empty());
            assert!(shell.current_path.is_empty());
        }
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_write_formatted_response_default() {
        // Test default formatting (no flags set)
        let io = MockIo::new();
        let handlers = MockHandlers;
        let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
            Shell::new(&TEST_TREE, handlers, io);

        let response = crate::response::Response::<DefaultConfig>::success("Test message");
        shell.write_formatted_response(&response).unwrap();

        // Default: message + postfix newline
        assert_eq!(shell.io.get_output(), "Test message\r\n");
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_write_formatted_response_with_prefix_newline() {
        let io = MockIo::new();
        let handlers = MockHandlers;
        let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
            Shell::new(&TEST_TREE, handlers, io);

        let response = crate::response::Response::<DefaultConfig>::success("Test")
            .with_prefix_newline();
        shell.write_formatted_response(&response).unwrap();

        // prefix newline + message + postfix newline
        assert_eq!(shell.io.get_output(), "\r\nTest\r\n");
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_write_formatted_response_indented() {
        let io = MockIo::new();
        let handlers = MockHandlers;
        let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
            Shell::new(&TEST_TREE, handlers, io);

        let response = crate::response::Response::<DefaultConfig>::success("Line 1\r\nLine 2")
            .indented();
        shell.write_formatted_response(&response).unwrap();

        // Each line indented with 2 spaces + postfix newline
        assert_eq!(shell.io.get_output(), "  Line 1\r\n  Line 2\r\n");
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_write_formatted_response_indented_single_line() {
        let io = MockIo::new();
        let handlers = MockHandlers;
        let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
            Shell::new(&TEST_TREE, handlers, io);

        let response = crate::response::Response::<DefaultConfig>::success("Single line")
            .indented();
        shell.write_formatted_response(&response).unwrap();

        // Single line indented
        assert_eq!(shell.io.get_output(), "  Single line\r\n");
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_write_formatted_response_without_postfix_newline() {
        let io = MockIo::new();
        let handlers = MockHandlers;
        let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
            Shell::new(&TEST_TREE, handlers, io);

        let response = crate::response::Response::<DefaultConfig>::success("No newline")
            .without_postfix_newline();
        shell.write_formatted_response(&response).unwrap();

        // Message without trailing newline
        assert_eq!(shell.io.get_output(), "No newline");
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_write_formatted_response_combined_flags() {
        let io = MockIo::new();
        let handlers = MockHandlers;
        let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
            Shell::new(&TEST_TREE, handlers, io);

        let response = crate::response::Response::<DefaultConfig>::success("Multi\r\nLine")
            .with_prefix_newline()
            .indented();
        shell.write_formatted_response(&response).unwrap();

        // Prefix newline + indented lines + postfix newline
        assert_eq!(shell.io.get_output(), "\r\n  Multi\r\n  Line\r\n");
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_write_formatted_response_all_flags_off() {
        let io = MockIo::new();
        let handlers = MockHandlers;
        let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
            Shell::new(&TEST_TREE, handlers, io);

        let response = crate::response::Response::<DefaultConfig>::success("Raw")
            .without_postfix_newline();
        shell.write_formatted_response(&response).unwrap();

        // No formatting at all
        assert_eq!(shell.io.get_output(), "Raw");
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_write_formatted_response_empty_message() {
        let io = MockIo::new();
        let handlers = MockHandlers;
        let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
            Shell::new(&TEST_TREE, handlers, io);

        let response = crate::response::Response::<DefaultConfig>::success("");
        shell.write_formatted_response(&response).unwrap();

        // Empty message still gets postfix newline
        assert_eq!(shell.io.get_output(), "\r\n");
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_write_formatted_response_indented_multiline() {
        let io = MockIo::new();
        let handlers = MockHandlers;
        let mut shell: Shell<MockLevel, MockIo, MockHandlers, DefaultConfig> =
            Shell::new(&TEST_TREE, handlers, io);

        let response = crate::response::Response::<DefaultConfig>::success("A\r\nB\r\nC\r\nD")
            .indented()
            .without_postfix_newline();
        shell.write_formatted_response(&response).unwrap();

        // All 4 lines indented, no trailing newline
        assert_eq!(shell.io.get_output(), "  A\r\n  B\r\n  C\r\n  D");
    }

    #[test]
    #[cfg(not(feature = "authentication"))]
    fn test_inline_message_flag() {
        // Test that inline_message flag is properly recognized
        let response = crate::response::Response::<DefaultConfig>::success("... processing")
            .inline();

        assert!(response.inline_message, "inline() should set inline_message flag");

        // Note: The actual inline behavior (no newline after input) is tested
        // via integration tests, as it requires simulating full command execution.
        // This test verifies the flag is set correctly.
    }
}
