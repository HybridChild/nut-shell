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
pub mod parser;

// Re-export key types
pub use handlers::CommandHandlers;
pub use history::CommandHistory;
pub use parser::{InputParser, ParseEvent};

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

    /// Awaiting authentication (feature-gated, but always defined)
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
    /// Authentication attempt (feature-gated: authentication)
    #[cfg(feature = "authentication")]
    Login {
        /// Username
        username: heapless::String<32>,
        /// Password
        password: heapless::String<64>,
    },

    /// Failed login (feature-gated: authentication)
    #[cfg(feature = "authentication")]
    InvalidLogin,

    /// Execute command
    Command {
        /// Command path
        path: heapless::String<128>, // TODO: Use C::MAX_INPUT when const generics stabilize
        /// Command arguments
        args: heapless::Vec<heapless::String<128>, 16>, // TODO: Use C::MAX_INPUT and C::MAX_ARGS
        /// Original command string (for history, feature-gated)
        #[cfg(feature = "history")]
        original: heapless::String<128>, // TODO: Use C::MAX_INPUT
        /// Phantom data for config type (will be used when const generics stabilize)
        _phantom: PhantomData<C>,
    },

    /// Request completions (feature-gated: completion)
    #[cfg(feature = "completion")]
    TabComplete {
        /// Partial path to complete
        path: heapless::String<128>, // TODO: Use C::MAX_INPUT
    },

    /// Navigate history (feature-gated: history)
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
    // ALWAYS present (not feature-gated)
    /// Command tree root
    tree: &'tree Directory<L>,

    /// Current user (None when logged out or auth disabled)
    current_user: Option<User<L>>,

    /// CLI state (auth state)
    state: CliState,

    /// Input buffer (using concrete size for now - TODO: use C::MAX_INPUT when const generics stabilize)
    input_buffer: heapless::String<128>,

    /// Current directory path (stack of child indices, using concrete size)
    current_path: heapless::Vec<usize, 8>,

    /// Input parser (escape sequences)
    parser: InputParser,

    /// Command history (using concrete sizes)
    #[cfg_attr(not(feature = "history"), allow(dead_code))]
    history: CommandHistory<10, 128>,

    /// I/O interface
    io: IO,

    /// Command handlers
    handlers: H,

    // ONLY this field is feature-gated
    /// Credential provider (feature-gated: authentication)
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
    /// Create new Shell with authentication enabled.
    ///
    /// Starts in `LoggedOut` state, requiring login before commands can be executed.
    pub fn new(
        tree: &'tree Directory<L>,
        handlers: H,
        credential_provider: &'tree (dyn crate::auth::CredentialProvider<L, Error = ()> + 'tree),
        io: IO,
    ) -> Self {
        Self {
            tree,
            handlers,
            current_user: None, // Start logged out
            state: CliState::LoggedOut,
            input_buffer: heapless::String::new(),
            current_path: heapless::Vec::new(),
            parser: InputParser::new(),
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
    /// Create new Shell with authentication disabled.
    ///
    /// Starts in `LoggedIn` state with no user, ready to accept commands.
    pub fn new(tree: &'tree Directory<L>, handlers: H, io: IO) -> Self {
        Self {
            tree,
            handlers,
            current_user: None, // No user needed (auth disabled)
            state: CliState::LoggedIn,
            input_buffer: heapless::String::new(),
            current_path: heapless::Vec::new(),
            parser: InputParser::new(),
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
            self.io.write_str("Welcome! Please log in.\r\n")?;
            self.io.write_str("Username: ")?;
        }

        #[cfg(not(feature = "authentication"))]
        {
            self.state = CliState::LoggedIn;
            self.io.write_str("Welcome to nut-shell!\r\n")?;
            self.generate_and_write_prompt()?;
        }

        Ok(())
    }

    /// Process a single character of input.
    ///
    /// Main entry point for character-by-character processing.
    /// Returns Ok(()) on success, Err on I/O error.
    pub fn process_char(&mut self, c: char) -> Result<(), IO::Error> {
        // Parse character
        let event = self
            .parser
            .process_char(c, &mut self.input_buffer)
            .map_err(|_| self.create_io_error())?;

        match event {
            ParseEvent::None => Ok(()), // Still accumulating sequence

            ParseEvent::Character(ch) => {
                // Echo character
                self.io.put_char(ch)?;
                Ok(())
            }

            ParseEvent::Backspace => {
                // Echo backspace sequence
                self.io.write_str("\x08 \x08")?;
                Ok(())
            }

            ParseEvent::Enter => self.handle_enter(),

            ParseEvent::Tab => self.handle_tab(),

            ParseEvent::UpArrow => self.handle_history(HistoryDirection::Previous),

            ParseEvent::DownArrow => self.handle_history(HistoryDirection::Next),

            ParseEvent::ClearAndRedraw => {
                // Buffer already cleared by parser
                self.clear_line_and_redraw()
            }
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

    /// Generate prompt string.
    ///
    /// Format: `username@path> ` (or `@path> ` when no user/auth disabled)
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
        self.io.write_str("\r\n")?;

        let input = self.input_buffer.clone();
        self.input_buffer.clear();

        match self.state {
            CliState::Inactive => Ok(()),

            #[cfg(feature = "authentication")]
            CliState::LoggedOut => self.handle_login_input(&input),

            CliState::LoggedIn => self.handle_command_input(&input),
        }
    }

    /// Handle login input (username or password).
    #[cfg(feature = "authentication")]
    fn handle_login_input(&mut self, input: &str) -> Result<(), IO::Error> {
        // Login logic will be implemented here
        // For now, just prompt for password or attempt login
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
                        self.io.write_str("Login successful!\r\n")?;
                        self.generate_and_write_prompt()?;
                    }
                    _ => {
                        // Login failed (user not found or wrong password)
                        self.io.write_str("Login failed. Try again.\r\n")?;
                        self.io.write_str("Username: ")?;
                    }
                }
            } else {
                self.io.write_str("Invalid format. Use: username:password\r\n")?;
                self.io.write_str("Username: ")?;
            }
        } else {
            // Just username entered, this is fine for now
            self.io.write_str("Password: ")?;
        }

        Ok(())
    }

    /// Handle command input.
    fn handle_command_input(&mut self, input: &str) -> Result<(), IO::Error> {
        // Skip empty commands
        if input.trim().is_empty() {
            self.generate_and_write_prompt()?;
            return Ok(());
        }

        // Check for global commands first
        match input.trim() {
            "?" => {
                self.show_help()?;
                self.generate_and_write_prompt()?;
                return Ok(());
            }
            "ls" => {
                self.show_ls()?;
                self.generate_and_write_prompt()?;
                return Ok(());
            }
            "clear" => {
                self.io.write_str("\x1b[2J\x1b[H")?; // ANSI clear screen
                self.generate_and_write_prompt()?;
                return Ok(());
            }
            #[cfg(feature = "authentication")]
            "logout" => {
                self.current_user = None;
                self.state = CliState::LoggedOut;
                self.current_path.clear();
                self.io.write_str("Logged out.\r\n")?;
                self.io.write_str("Username: ")?;
                return Ok(());
            }
            _ => {}
        }

        // Execute command
        match self.execute_command(input) {
            Ok(response) => {
                // Write response
                self.io.write_str(response.message.as_str())?;
                self.io.write_str("\r\n")?;

                // Add to history if not excluded
                #[cfg(feature = "history")]
                if !response.exclude_from_history {
                    self.history.add(input);
                }

                self.generate_and_write_prompt()?;
            }
            Err(e) => {
                self.io.write_str("Error: ")?;
                match e {
                    CliError::CommandNotFound => self.io.write_str("Command not found")?,
                    CliError::InvalidPath => self.io.write_str("Invalid path")?,
                    CliError::InvalidArgumentCount { .. } => {
                        self.io.write_str("Invalid argument count")?
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

    /// Execute a command string.
    fn execute_command(&mut self, input: &str) -> Result<Response<C>, CliError> {
        // Parse command path and arguments
        let parts: heapless::Vec<&str, 17> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err(CliError::CommandNotFound);
        }

        let path_str = parts[0];
        let args = &parts[1..];

        // Resolve path
        let (target_node, new_path) = self.resolve_path(path_str)?;

        // If it's a directory, navigate to it
        if let Node::Directory(_) = target_node {
            self.current_path = new_path;
            #[cfg(feature = "history")]
            return Ok(Response::success("").without_history());
            #[cfg(not(feature = "history"))]
            return Ok(Response::success(""));
        }

        // It's a command - extract metadata and execute
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

            // Check command kind and dispatch
            match cmd_meta.kind {
                CommandKind::Sync => {
                    // Execute synchronous command
                    self.handlers.execute_sync(cmd_meta.name, args)
                }
                #[cfg(feature = "async")]
                CommandKind::Async => {
                    // Async command called from sync context
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
    fn resolve_path(
        &self,
        path_str: &str,
    ) -> Result<(&'tree Node<L>, heapless::Vec<usize, 8>), CliError> {
        // Start from current directory or root
        let mut working_path: heapless::Vec<usize, 8> = if path_str.starts_with('/') {
            heapless::Vec::new() // Absolute path starts from root
        } else {
            self.current_path.clone() // Relative path starts from current
        };

        // Parse path
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
    fn get_node_at_path(&self, path: &heapless::Vec<usize, 8>) -> Result<&'tree Node<L>, CliError> {
        if path.is_empty() {
            // Root directory - need to find a way to return it as a Node
            // For now, return error since we can't construct Node::Directory here
            return Err(CliError::InvalidPath);
        }

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
                    self.input_buffer
                        .push_str(&completion.completion)
                        .map_err(|_| self.create_io_error())?;

                    // Redraw line
                    self.io.write_str("\r")?; // Carriage return
                    let prompt = self.generate_prompt();
                    self.io.write_str(prompt.as_str())?;
                    self.io.write_str(self.input_buffer.as_str())?;
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

    /// Create generic I/O error (for conversions).
    fn create_io_error(&self) -> IO::Error {
        // This is a workaround since we can't directly convert CliError to IO::Error
        // In practice, the I/O error type would need to support this conversion
        // For now, we'll use a default error value
        unsafe { core::mem::zeroed() }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
}
