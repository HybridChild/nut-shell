# cli-service - Runtime Internals

This document provides a detailed analysis of cli-service runtime behavior, including complete pseudocode implementations, state machines, data flow, and performance characteristics.

## High-Level Overview

```
┌─────────────┐
│   CharIo    │ ← Platform-specific I/O (UART, USB-CDC, etc.)
└──────┬──────┘
       │ get_char()
       ▼
┌─────────────────────────────────────────────────────────┐
│              CliService Main Loop                       │
│  - State: Inactive/LoggedOut/LoggedIn                   │
│  - CurrentUser: Option<User<L>>                         │
│  - Handlers: H (implements CommandHandlers trait)       │
│  - InputBuffer: heapless::String<128>                   │
│  - History: CommandHistory<N>                           │
│  - Parser: InputParser (escape sequence state machine)  │
└──────┬──────────────────────────────────────────────────┘
       │ process_char(c) or process_char_async(c)
       ▼
┌─────────────────┐
│  InputParser    │ ← State machine for escape sequences
│  - Normal       │
│  - EscapeStart  │
│  - EscapeSeq    │
└──────┬──────────┘
       │ ParseEvent
       ▼
┌────────────────────────────────────────────────┐
│         Event Processing (by state)            │
│  - LoggedOut: handle login                     │
│  - LoggedIn: handle commands/navigation        │
└──────┬─────────────────────────────────────────┘
       │ Request
       ▼
┌─────────────────────────────────────────────────┐
│         Request Handler                         │
│  - Global commands (help, ?, logout, clear)     │
│  - Navigate (change directory)                  │
│  - Execute (run command with args)              │
│  - Tab completion                               │
└──────┬──────────────────────────────────────────┘
       │ uses
       ▼
┌─────────────────────────────────────────────────┐
│          Tree Navigation                        │
│  - Path parsing & resolution                    │
│  - Access control checks                        │
│  - Node lookup (Command vs Directory)           │
└──────┬──────────────────────────────────────────┘
       │ Response
       ▼
┌─────────────────────────────────────────────────┐
│       Response Formatter                        │
│  - Prefix/postfix newlines                      │
│  - Indentation                                  │
│  - Status formatting                            │
└──────┬──────────────────────────────────────────┘
       │ write_str()
       ▼
┌─────────────┐
│   CharIo    │ → Terminal output
└─────────────┘
```

## Level 1: Character Input Processing

```rust
// Main entry point
CliService::process_char(c: char) -> Result<(), IO::Error>
{
    match self.state {
        Inactive => { /* Ignore input */ }

        LoggedOut => {
            // Parse event (escape sequences, editing)
            match self.parser.process_char(c, &mut self.input_buffer)? {
                ParseEvent::Character(ch) => {
                    // Password masking logic
                    if contains_colon(&self.input_buffer) {
                        self.io.put_char('*')?;  // Mask after ':'
                    } else {
                        self.io.put_char(ch)?;   // Echo before ':'
                    }
                }

                ParseEvent::Backspace => {
                    self.io.write_str("\x08 \x08")?;  // Erase character
                }

                ParseEvent::Enter => {
                    self.handle_login_attempt()?;
                }

                ParseEvent::Tab | ParseEvent::UpArrow | ParseEvent::DownArrow => {
                    // Disabled when logged out
                }

                ParseEvent::ClearAndRedraw => {
                    self.input_buffer.clear();
                    self.redraw_line()?;
                }

                _ => {}
            }
        }

        LoggedIn => {
            // Full command processing (see Level 2)
            match self.parser.process_char(c, &mut self.input_buffer)? {
                ParseEvent::Character(ch) => {
                    self.io.put_char(ch)?;  // Echo normally
                }

                ParseEvent::Backspace => {
                    self.io.write_str("\x08 \x08")?;
                }

                ParseEvent::Enter => {
                    self.handle_command_input()?;  // See Level 3
                }

                ParseEvent::Tab => {
                    self.handle_tab_completion()?;  // See Level 4
                }

                ParseEvent::UpArrow => {
                    self.handle_history_previous()?;  // See Level 4
                }

                ParseEvent::DownArrow => {
                    self.handle_history_next()?;
                }

                ParseEvent::ClearAndRedraw => {
                    self.input_buffer.clear();
                    self.history.reset();  // Exit history navigation
                    self.redraw_line()?;
                }

                _ => {}
            }
        }
    }
}
```

## Level 2: InputParser State Machine

```rust
// Escape sequence parser
InputParser::process_char(c: char, buffer: &mut String) -> Result<ParseEvent>
{
    match (self.state, c) {
        // ═══════════════════════════════════════════════════
        // NORMAL MODE
        // ═══════════════════════════════════════════════════
        (Normal, '\x1b') => {
            // ESC starts escape sequence
            self.state = EscapeStart;
            Ok(ParseEvent::None)
        }

        (Normal, '\x08' | '\x7F') => {
            // Backspace/Delete
            if !buffer.is_empty() {
                buffer.pop();
                Ok(ParseEvent::Backspace)
            } else {
                Ok(ParseEvent::None)
            }
        }

        (Normal, '\r' | '\n') => {
            // Enter
            Ok(ParseEvent::Enter)
        }

        (Normal, '\t') => {
            // Tab
            Ok(ParseEvent::Tab)
        }

        (Normal, c) if c.is_ascii_graphic() || c == ' ' => {
            // Regular character
            buffer.push(c).map_err(|_| CliError::BufferFull)?;
            Ok(ParseEvent::Character(c))
        }

        (Normal, _) => {
            // Ignore other control characters
            Ok(ParseEvent::None)
        }

        // ═══════════════════════════════════════════════════
        // ESCAPE START (saw first ESC)
        // ═══════════════════════════════════════════════════
        (EscapeStart, '\x1b') => {
            // Double ESC = clear buffer!
            buffer.clear();
            self.state = Normal;
            Ok(ParseEvent::ClearAndRedraw)
        }

        (EscapeStart, '[') => {
            // ESC [ = begin CSI sequence
            self.state = EscapeSequence;
            self.escape_buffer.clear();
            Ok(ParseEvent::None)
        }

        (EscapeStart, other) => {
            // ESC + other = clear buffer, then process char
            buffer.clear();
            self.state = Normal;
            self.process_char(other, buffer)  // Re-process
        }

        // ═══════════════════════════════════════════════════
        // ESCAPE SEQUENCE (ESC [ ... seen)
        // ═══════════════════════════════════════════════════
        (EscapeSequence, 'A') => {
            // Up arrow
            self.state = Normal;
            Ok(ParseEvent::UpArrow)
        }

        (EscapeSequence, 'B') => {
            // Down arrow
            self.state = Normal;
            Ok(ParseEvent::DownArrow)
        }

        (EscapeSequence, c) if c.is_ascii_alphabetic() => {
            // Unknown sequence - discard
            self.state = Normal;
            Ok(ParseEvent::None)
        }

        (EscapeSequence, c) => {
            // Buffer intermediate chars
            self.escape_buffer.push(c).ok();

            // Overflow protection
            if self.escape_buffer.len() >= 16 {
                self.state = Normal;
                Ok(ParseEvent::None)
            } else {
                Ok(ParseEvent::None)
            }
        }
    }
}
```

## Level 3: Command Input Processing

```rust
CliService::handle_command_input() -> Result<(), IO::Error>
{
    // 1. Skip empty input (show prompt on same line)
    if self.input_buffer.is_empty() {
        self.io.write_str("\r\n")?;
        self.show_prompt()?;
        return Ok(());
    }

    // 2. Parse input into path + args
    let input = self.input_buffer.as_str();
    let (path_str, args) = split_whitespace(input);

    // 3. Check for global commands FIRST
    let response = match path_str {
        "help" => self.handle_help(),
        "?" => self.handle_context_help(),

        #[cfg(feature = "authentication")]
        "logout" => self.handle_logout(),

        "clear" => {
            self.io.write_str("\x1b[2J\x1b[H")?;  // ANSI clear
            Response::success("")
        }

        _ => {
            // 4. Not a global command - parse as path
            let request = self.parse_path_and_args(path_str, args)?;

            // 5. Process request (navigation or execution)
            self.process_request(request)?
        }
    };

    // 6. Add successful commands to history (stub no-ops if disabled)
    if response.is_success() && !path_str.is_empty() {
        self.history.add(&self.input_buffer);
    }

    // 7. Clear buffer and reset history navigation
    self.input_buffer.clear();
    self.history.reset();

    // 8. Echo newline ONLY if message is not inline
    if !response.inline_message {
        self.io.write_str("\r\n")?;
    }

    // 9. Display response
    self.display_response(&response)?;

    // 10. Show prompt (unless response disabled it)
    if response.show_prompt {
        self.show_prompt()?;
    }

    Ok(())
}
```

## Level 4: Path Parsing & Tree Navigation

```rust
CliService::parse_path_and_args(path_str: &str, args: &[&str])
    -> Result<Request, CliError>
{
    // 1. Parse path string into path segments
    let path = Path::parse(path_str)?;  // Handles /, .., ., absolute/relative

    // 2. Resolve path against current directory
    let (node, resolved_path_stack) = self.resolve_path(&path)?;
    //                                     └─ WITH ACCESS CONTROL

    // 3. Determine request type based on node
    match node {
        Node::Directory(dir) => {
            // Navigation request
            Ok(Request::Navigate {
                target_path: resolved_path_stack,
                directory: dir,
            })
        }

        Node::Command(cmd) => {
            // Execution request
            Ok(Request::Execute {
                command: cmd,
                args: args.to_vec(),  // heapless::Vec
            })
        }
    }
}

CliService::resolve_path(&self, path: &Path)
    -> Result<(&Node<L>, PathStack), CliError>
{
    // Start from root or current directory
    let mut current = if path.is_absolute() {
        self.root_directory
    } else {
        self.get_current_directory()
    };

    let mut path_stack = if path.is_absolute() {
        PathStack::new()  // heapless::Vec<usize, MAX_DEPTH>
    } else {
        self.current_path_stack.clone()
    };

    // Walk path segments
    for segment in path.segments() {
        match segment {
            ".." => {
                // Go up one level
                path_stack.pop();
                current = self.walk_to_directory(&path_stack)?;
            }

            "." => {
                // Stay in current directory
            }

            name => {
                // Find child by name
                let child_index = current.children
                    .iter()
                    .position(|node| node.name() == name)
                    .ok_or(CliError::InvalidPath)?;

                let child = &current.children[child_index];

                // ═══════════════════════════════════════════
                // ACCESS CONTROL CHECK
                // ═══════════════════════════════════════════
                self.check_access(child)?;
                //   └─ Returns InvalidPath if denied (hides existence)

                // Update path stack
                path_stack.push(child_index)
                    .map_err(|_| CliError::PathTooDeep)?;

                // Update current if it's a directory
                match child {
                    Node::Directory(dir) => current = dir,
                    Node::Command(_) => {
                        // Command found - return it
                        return Ok((child, path_stack));
                    }
                }
            }
        }
    }

    // Return final directory
    Ok((Node::Directory(current), path_stack))
}

CliService::check_access(&self, node: &Node<L>) -> Result<(), CliError>
{
    #[cfg(feature = "authentication")]
    {
        // SAFETY: current_user is guaranteed to be Some() when in LoggedIn state
        // Commands are only processed in LoggedIn state (see process_char logic)
        // If auth is enabled: LoggedOut state doesn't process commands
        // If auth is disabled: starts in LoggedIn state with current_user = None, but this block is not compiled
        let user = self.current_user
            .as_ref()
            .expect("BUG: check_access called while not logged in");

        if user.access_level < node.access_level() {
            // SECURITY: Return same error as non-existent path to hide existence
            return Err(CliError::InvalidPath);
        }
    }

    #[cfg(not(feature = "authentication"))]
    {
        let _ = node;  // Always allow when auth disabled
    }

    Ok(())
}
```

## Level 5: Request Processing

```rust
CliService::process_request(&mut self, request: Request)
    -> Result<Response, CliError>
{
    match request {
        Request::Navigate { target_path, directory } => {
            // Update current directory
            self.current_path_stack = target_path;

            // No output, just update state
            Ok(Response::success(""))
        }

        Request::Execute { command, args } => {
            // Validate argument count
            let arg_count = args.len();
            if arg_count < command.min_args || arg_count > command.max_args {
                return Ok(Response::error(&format!(
                    "Invalid argument count. Expected {}-{}, got {}",
                    command.min_args, command.max_args, arg_count
                )));
            }

            // Dispatch to handler based on command kind
            let response = match command.kind {
                CommandKind::Sync => {
                    // Synchronous execution via handlers
                    self.handlers.execute_sync(command.name, &args)?
                }
                CommandKind::Async => {
                    // In sync mode (process_char), async commands not supported
                    return Err(CliError::AsyncNotSupported);
                }
            };

            Ok(response)
        }
    }
}
```

**Note on Architecture:** Commands use the metadata/execution separation pattern:

- **CommandMeta**: Const-initializable metadata (name, description, access_level, kind, arg counts) stored in ROM
- **CommandHandlers trait**: User-implemented trait with `execute_sync()` and `execute_async()` methods
- **CommandKind enum**: Marker indicating Sync or Async execution type
- **Dispatch flow**: CliService validates access/args, then dispatches to appropriate handler method based on kind
- **Benefits**: Enables async commands without heap, maintains const-initialization, zero-cost for sync-only builds

This pattern allows both sync and async commands in a single codebase while preserving the no_std, const-initialization constraints. See [DESIGN.md](DESIGN.md) for complete architecture details and rationale.

### Async Processing Flow (Embassy/RTIC)

When using `process_char_async()`, async commands can be awaited inline:

```rust
#[cfg(feature = "async")]
CliService::process_char_async(&mut self, c: char) -> Result<(), IO::Error>
{
    // Same parsing and event handling as process_char()...

    match event {
        ParseEvent::Enter => {
            self.handle_command_input_async().await?;  // Async version
        }
        // ... other events identical
    }
}

#[cfg(feature = "async")]
CliService::process_request_async(&mut self, request: Request)
    -> Result<Response, CliError>
{
    match request {
        Request::Execute { command, args } => {
            // Validate argument count (same as sync)
            let arg_count = args.len();
            if arg_count < command.min_args || arg_count > command.max_args {
                return Ok(Response::error(&format!(
                    "Invalid argument count. Expected {}-{}, got {}",
                    command.min_args, command.max_args, arg_count
                )));
            }

            // Dispatch to handler based on command kind
            let response = match command.kind {
                CommandKind::Sync => {
                    // Sync commands still work in async mode
                    self.handlers.execute_sync(command.name, &args)?
                }
                CommandKind::Async => {
                    // Async commands are awaited inline
                    self.handlers.execute_async(command.name, &args).await?
                }
            };

            Ok(response)
        }

        Request::Navigate { .. } => {
            // Navigation is synchronous (no await needed)
            // ... same as sync version
        }
    }
}
```

**Behavior differences:**
- **Sync `process_char()`**: Async commands return `AsyncNotSupported` error
- **Async `process_char_async()`**: Async commands awaited until complete
- CLI blocks during async execution, but other Embassy tasks continue running
- Natural error propagation via `?` operator

**Usage example:**
```rust
#[embassy_executor::task]
async fn cli_task(usb: UsbDevice) {
    let mut cli = CliService::new(&ROOT, handlers, io);

    loop {
        let c = usb.read_char().await;  // Await input
        cli.process_char_async(c).await.ok();  // May await on async commands
        io.flush().await.ok();  // Flush output
    }
}
```

**Async Command Execution Behavior:**

When `process_char_async()` is used (Embassy/RTIC environments):

**During async command execution:**
- CLI task blocks on the async command (awaits completion)
- User input is NOT processed while command runs
- CharIo may buffer incoming characters (implementation-dependent)
- Other Embassy tasks continue running normally (CLI task is suspended)
- No cancellation mechanism provided (command runs to completion)

**User interaction:**
- User CANNOT send new commands while async command executing
- Characters typed during execution may be:
  - Buffered by CharIo implementation (processed after command completes)
  - Dropped (if CharIo buffer overflows)
  - Implementation-dependent behavior
- No visual feedback that command is running (command should provide its own)

**Timeout handling:**
- Library does NOT implement command timeouts
- Command functions should implement their own timeouts using executor primitives:
  ```rust
  async fn http_get_async(args: &[&str]) -> Result<Response, CliError> {
      embassy_time::with_timeout(Duration::from_secs(30), async {
          HTTP_CLIENT.get(args[0]).await
      })
      .await
      .map_err(|_| CliError::Timeout)?
  }
  ```

**Task spawning from async commands:**
- Async commands CAN spawn background tasks if needed
- Spawned tasks are detached (CLI does not track them)
- Command should return Response immediately after spawning
- Background tasks MUST NOT access CliService (not thread-safe)
- Example use case: Start long-running background operation, return immediately

**Error propagation:**
- Async command errors propagate normally via `?` operator
- Same error handling as sync commands
- Command not added to history on error

## Level 6: Interactive Features (Tab Completion & History)

```rust
CliService::handle_tab_completion() -> Result<(), IO::Error>
{
    // Get current directory node
    let current = self.get_current_directory();

    // Get user's access level for filtering
    let access_level = self.current_user.as_ref().map(|u| u.access_level);

    // Call completion module (stub returns empty Vec if disabled)
    let suggestions = completion::suggest_completions(
        Node::Directory(current),
        self.input_buffer.as_str(),
        access_level,
    )?;

    match suggestions.len() {
        0 => {
            // No matches - do nothing
        }

        1 => {
            // Single match - auto-complete
            let suggestion = suggestions[0];

            // Clear current input
            self.clear_line()?;

            // Write completion
            self.input_buffer.clear();
            self.input_buffer.push_str(suggestion)
                .map_err(|_| CliError::BufferFull)?;

            // Add trailing character
            if is_directory(suggestion, current) {
                self.input_buffer.push('/').ok();
            } else {
                self.input_buffer.push(' ').ok();  // Space after command
            }

            // Redraw line
            self.redraw_line()?;
        }

        _ => {
            // Multiple matches - display options
            self.io.write_str("\r\n")?;
            for suggestion in suggestions {
                self.io.write_str("  ")?;
                self.io.write_str(suggestion)?;
                if is_directory(suggestion, current) {
                    self.io.write_str("/")?;
                }
                self.io.write_str("\r\n")?;
            }
            self.io.write_str("\r\n")?;

            // Redraw original input
            self.show_prompt()?;
            self.io.write_str(self.input_buffer.as_str())?;
        }
    }

    Ok(())
}

CliService::handle_history_previous() -> Result<(), IO::Error>
{
    // Save current buffer if entering history mode
    if !self.history.is_navigating() {
        self.history.save_current(self.input_buffer.clone());
    }

    // Get previous command (stub returns None if disabled)
    if let Some(cmd) = self.history.previous() {
        self.input_buffer = cmd;
        self.redraw_line()?;
    }

    Ok(())
}

CliService::handle_history_next() -> Result<(), IO::Error>
{
    // Get next command or original buffer (stub returns None if disabled)
    if let Some(cmd) = self.history.next() {
        self.input_buffer = cmd;
        self.redraw_line()?;
    }

    Ok(())
}
```

## Level 7: Response Formatting & Output

```rust
CliService::display_response(&self, response: &Response)
    -> Result<(), IO::Error>
{
    // Prefix newline (default: true)
    if response.prefix_newline {
        self.io.write_str("\r\n")?;
    }

    // Indentation (default: true, 2 spaces)
    if response.indent_message && !response.message.is_empty() {
        self.io.write_str("  ")?;
    }

    // Message content
    self.io.write_str(&response.message)?;

    // Postfix newline (default: true)
    if response.postfix_newline {
        self.io.write_str("\r\n")?;
    }

    Ok(())
}

CliService::show_prompt(&self) -> Result<(), IO::Error>
{
    let prompt = self.generate_prompt();
    self.io.write_str(prompt.as_str())?;
    Ok(())
}

CliService::generate_prompt(&self) -> heapless::String<64>
{
    let mut prompt = heapless::String::new();

    // Username (empty if auth disabled or not logged in)
    let username = self.current_user
        .as_ref()
        .map(|u| u.username.as_str())
        .unwrap_or("");

    // Format: username@path>
    prompt.push_str(username).ok();
    prompt.push('@').ok();
    prompt.push_str(&self.current_path_string()).ok();
    prompt.push_str("> ").ok();

    prompt
}

CliService::current_path_string(&self) -> heapless::String<128>
{
    let mut path = heapless::String::new();
    path.push('/').ok();

    // Walk path stack and build string
    let mut current = self.root_directory;
    for &child_index in &self.current_path_stack {
        let child = &current.children[child_index];
        path.push_str(child.name()).ok();
        path.push('/').ok();

        if let Node::Directory(dir) = child {
            current = dir;
        }
    }

    // Remove trailing slash (except for root)
    if path.len() > 1 {
        path.pop();
    }

    path
}
```

## Complete Flow Diagram (Authentication Enabled)

```
┌──────────────────────────────────────────────────────────────────┐
│                     SYSTEM STARTUP                               │
└───────────────────────────┬──────────────────────────────────────┘
                            │
                            ▼
                    activate() called
                            │
                            ▼
                 ┌──────────────────────┐
                 │  State = LoggedOut   │
                 │  current_user = None │
                 └──────────┬───────────┘
                            │
                            ▼
          Display: "Welcome to CLI Service. Please login."
                            │
                            ▼
                    Display prompt: "> "
                            │
┌───────────────────────────┴───────────────────────────────────────┐
│                     CHARACTER INPUT LOOP                          │
└───────────────────────────┬───────────────────────────────────────┘
                            │
                            ▼
                  get_char() from CharIo
                            │
                            ▼
            ┌───────────────────────────────┐
            │   InputParser State Machine   │
            │   - Escape sequence handling  │
            │   - Line editing              │
            │   - Double-ESC clear          │
            └───────────────┬───────────────┘
                            │
                            ▼
                    ┌─── State check
                    │
       ┌────────────┴────────────┐
       │                         │
       ▼                         ▼
┌─────────────┐          ┌──────────────┐
│ LoggedOut   │          │  LoggedIn    │
└──────┬──────┘          └──────┬───────┘
       │                        │
       ▼                        ▼
  Password                  Command
  masking                   processing
       │                        │
       ▼                        ▼
  Enter pressed?           Enter pressed?
       │                        │
       ▼                        ▼
  Parse login           Parse command input
  username:password             │
       │                        ▼
       ▼              ┌─── Global command?
  Authenticate        │
  with provider       ├─ "help" → list globals
       │              ├─ "?" → list current dir
       │              ├─ "logout" → logout
       ▼              ├─ "clear" → clear screen
  Valid?              │
   ├─ Yes ─┐          │
   │       │          └─ No → Parse as path
   │       │                    │
   │       ▼                    ▼
   │  State = LoggedIn     Path::parse()
   │  current_user = User       │
   │       │                    ▼
   │       │               resolve_path()
   ▼       │                    │
 "Invalid login"        ┌───────┴────────┐
   │       │            │                │
   │       │            ▼                ▼
   └───┬───┘       Node::Directory  Node::Command
       │                │                │
       │                ▼                ▼
       │          Navigate to     Execute with args
       │          directory       (validate count)
       │                │                │
       │                └───────┬────────┘
       │                        │
       │                        ▼
       │                   Add to history (if success)
       │                        │
       │                        ▼
       │                  Format response
       │                  (indentation, newlines)
       │                        │
       │                        ▼
       │                  write_str() to CharIo
       │                        │
       │                        ▼
       │                  Display prompt
       │                        │
       │                        │
       │                        │
       └───────────┬────────────┘
                   │
                   ▼
             Loop continues...

```

## State Transition Diagram

```
                    ┌─────────────┐
                    │  Inactive   │
                    └──────┬──────┘
                           │ activate()
                           │
          ┌────────────────┴────────────────┐
          │                                 │
   #[cfg(feature =              #[cfg(not(feature =
   "authentication")]           "authentication"))]
          │                                 │
          ▼                                 ▼
    ┌──────────┐                      ┌──────────┐
    │          │──login_success()────►│          │
    │LoggedOut │                      │ LoggedIn │
    │          │◄────logout()─────────│          │
    └────┬─────┘                      └────┬─────┘
         │                                 │
         │                                 │
         └──► Password masking active      │
              No tab completion            │
              No history navigation        │
                                           │
                                           ├─► History navigation
                                           ├─► Command execution
                                           ├─► Tab completion
                                           ├─► Access control checks
                                           │
                                           └──exit()──► Inactive
```

## Access Control Enforcement Points

```
┌─────────────────────────────────────────────────────────┐
│         EVERY TREE TRAVERSAL CHECKS ACCESS              │
└─────────────────────────────────────────────────────────┘

1. Path Resolution (resolve_path)
   ├─ Check each path segment
   ├─ User level >= Node level
   └─ Fail → return "Invalid path" (hide existence)

2. Tab Completion (suggest_completions)
   ├─ Filter suggestions by access level
   └─ Only show accessible nodes

3. Directory Listing (? command)
   ├─ Filter children by access level
   └─ Only list accessible nodes

4. Command Execution
   ├─ Final check before executing
   └─ Already checked during path resolution

┌─────────────────────────────────────────────────────────┐
│   SECURITY: Inaccessible nodes appear non-existent      │
│   "Invalid path" for both missing AND denied access     │
└─────────────────────────────────────────────────────────┘
```

## Key Architectural Insights

### Unified State Machine
- Single `CliState` enum drives behavior
- `current_user: Option<User<L>>` always present (None = not logged in OR auth disabled)
- Feature gates only affect constructor and initial state

### Zero-Copy Parsing
- Input split into `&str` slices (no allocation)
- `heapless::Vec<&str, MAX_ARGS>` holds argument references
- Path parsing builds stack of indices (not copies of names)

### Path Stack Navigation
- Current location = `Vec<usize, MAX_DEPTH>` (indices from root)
- Walk tree by following indices
- Enables const tree initialization (no pointers)

### Stub Function Pattern
- Completion, history modules always exist
- Feature-disabled = stub returns empty/None
- Single code path in main service (no `#[cfg]` branching)

### Access Control
- Checked at every tree traversal
- Returns `InvalidPath` for both missing and inaccessible
- Filters completion/listing results

## Data Flow Summary

```
Character Input
    │
    ├─► InputParser (state machine)
    │       └─► ParseEvent
    │
    ├─► State-dependent handling
    │       ├─► LoggedOut: password masking
    │       └─► LoggedIn: full processing
    │
    ├─► Command parsing
    │       ├─► Global commands (help, ?, logout, clear)
    │       └─► Path-based commands
    │
    ├─► Path resolution
    │       ├─► Parse path segments (/, .., name)
    │       ├─► Walk tree with access checks
    │       └─► Return Node (Command or Directory)
    │
    ├─► Request processing
    │       ├─► Navigate: update path stack
    │       └─► Execute: call command function
    │
    ├─► Response generation
    │       ├─► Success/Error status
    │       ├─► Message formatting
    │       └─► Prompt control
    │
    └─► Terminal output
            ├─► Indentation
            ├─► Newlines
            └─► Prompt display
```

## Memory Layout (no_std)

```
┌─────────────────────────────────────────────────────┐
│                    FLASH (ROM)                      │
├─────────────────────────────────────────────────────┤
│  - Command tree (const-initialized)                 │
│  - Command function pointers                        │
│  - Static strings (names, descriptions)             │
│  - Code (.text section)                             │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│                     RAM                             │
├─────────────────────────────────────────────────────┤
│  CliService struct:                                 │
│    - state: CliState                    (1 byte)    │
│    - current_user: Option<User<L>>      (~64 bytes) │
│    - input_buffer: String<MAX_INPUT>    (128 bytes) │
│    - history: CommandHistory<HISTORY_SIZE> (~1.3 KB)│
│    - parser: InputParser                (~20 bytes) │
│    - current_path_stack: Vec<usize, MAX_PATH_DEPTH>│
│                                         (32 bytes)  │
│    - io: IO                             (variable)  │
│    - tree: &'static Directory           (pointer)   │
│                                                     │
│  Total (approx):                        ~1.5-2 KB   │
└─────────────────────────────────────────────────────┘
```

### Configurable Const Generics

These buffer sizes are const generics that can be configured at compile time for RAM optimization:

| Constant | Default | Range | RAM Impact | Used In |
|----------|---------|-------|------------|---------|
| `MAX_INPUT` | 128 | 32-256 | N bytes | Input buffer capacity |
| `MAX_PATH_DEPTH` | 8 | 4-16 | N × 4 bytes | Path stack (32 bytes default) |
| `MAX_ARGS` | 16 | 8-32 | 0 bytes (stack only) | Argument parsing slice capacity |
| `HISTORY_SIZE` | 10 | 0-20 | N × 130 bytes | Command history (1.3 KB default) |

**Configuration example:**
```rust
type History = CommandHistory<4>;  // RAM: 4 × 130 = ~520 bytes (vs 1.3 KB)
type PathStack = heapless::Vec<usize, 4>;  // RAM: 16 bytes (vs 32 bytes)
type InputBuffer = heapless::String<64>;  // RAM: 64 bytes (vs 128 bytes)
```

**RAM-constrained configuration** (N=4, MAX_INPUT=64):
- History: ~520 bytes (vs 1.3 KB) → saves ~800 bytes
- Input: 64 bytes (vs 128 bytes) → saves 64 bytes
- Path stack: 16 bytes (vs 32 bytes) → saves 16 bytes
- **Total RAM**: ~880 bytes vs ~1.5 KB (saves ~600 bytes)

**Note:** These are compile-time const generics, not runtime configuration. See [DESIGN.md](DESIGN.md) line 655 for usage in feature configuration examples.

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Character input | O(1) | Direct CharIo trait call |
| Escape sequence parsing | O(1) | State machine with fixed buffer |
| Path parsing | O(n) | n = number of path segments |
| Tree traversal | O(d × c) | d = depth, c = avg children per node |
| Access check | O(1) | Simple comparison |
| Tab completion | O(n) | n = children in current directory |
| History previous/next | O(1) | Circular buffer index arithmetic |
| History add | O(1) | Circular buffer push (amortized) |
| Response formatting | O(n) | n = message length |

## Concurrency and Safety

### Thread Safety: NOT THREAD-SAFE

**CliService is NOT Send or Sync:**
- Cannot be shared between threads or tasks
- Must be owned by a single task/thread
- No internal synchronization mechanisms
- Mutable state not protected by locks

**Design principle:** Embedded systems typically single-threaded or use message-passing, not shared-state concurrency.

### Interrupt Safety: NOT ISR-SAFE

**`process_char()` MUST NOT be called from interrupt handlers:**
- May allocate stack frames (not suitable for ISR stack)
- Performs I/O operations (may block or take significant time)
- Not designed for deterministic execution time

**Correct pattern for interrupt-driven input:**
```rust
static RX_BUFFER: Mutex<RefCell<heapless::Deque<u8, 64>>> = ...;

// ISR: Only buffer the character
fn UART_IRQ() {
    if uart.is_readable() {
        RX_BUFFER.lock(|buf| buf.push_back(uart.read_byte()));
    }
}

// Main loop: Process buffered characters
fn main() {
    loop {
        if let Some(c) = RX_BUFFER.lock(|buf| buf.pop_front()) {
            cli.process_char(c as char).ok();
        }
    }
}
```

### Re-entrancy: NOT RE-ENTRANT

**`process_char()` should not be called recursively:**
- Command handlers should NOT call `process_char()` directly or indirectly
- CharIo implementations should NOT trigger re-entrant calls
- Safe to call from single context only

**Why:** Would corrupt internal state (input buffer, parser state, path stack).

### Embassy Multi-Task Usage

**Correct pattern (single CLI task):**
```rust
#[embassy_executor::task]
async fn cli_task(usb: UsbDevice) {
    let handlers = MyHandlers;
    let mut cli = CliService::new(&ROOT, handlers, usb_io);  // Owned by this task

    loop {
        if let Ok(Some(c)) = usb_io.get_char() {
            cli.process_char_async(c).await.ok();
        }
        usb_io.flush().await.ok();
        Timer::after(Duration::from_millis(10)).await;
    }
}

#[embassy_executor::task]
async fn sensor_task() {
    // Completely independent task - no CLI access
    loop {
        read_sensors().await;
        Timer::after(Duration::from_secs(1)).await;
    }
}
```

**INCORRECT pattern (DO NOT DO THIS):**
```rust
// WRONG: Trying to share CliService between tasks
static CLI: Mutex<CliService> = ...;  // ERROR: CliService not designed for sharing

// WRONG: Accessing CLI from multiple tasks
async fn task_a() { CLI.lock().await.process_char('a'); }  // Race conditions!
async fn task_b() { CLI.lock().await.process_char('b'); }  // Race conditions!
```

### CommandHandlers Thread Safety

**Handler implementations should avoid shared mutable state:**
```rust
// SAFE: Stateless handler
struct MyHandlers;
impl CommandHandlers for MyHandlers {
    fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response, CliError> {
        match name {
            "status" => Ok(Response::success("OK")),
            _ => Err(CliError::CommandNotFound),
        }
    }
}

// SAFE: Read-only shared state
struct MyHandlers<'a> {
    config: &'a Config,  // Immutable reference - safe
}

// UNSAFE: Shared mutable state without synchronization
struct MyHandlers {
    counter: Cell<u32>,  // Mutable - NOT safe if CLI shared (which it shouldn't be)
}
```

**Guideline:** If you need shared mutable state, use message-passing or channels to communicate with other tasks, don't share CliService itself.

### CharIo Thread Safety

**CharIo implementations are responsible for their own thread safety:**
- If CharIo buffers are accessed from ISRs, use appropriate synchronization (Mutex, critical sections)
- If multiple tasks write to same output, CharIo must handle interleaving
- CliService assumes CharIo methods are safe to call from its context

**Example: ISR-safe CharIo buffering:**
```rust
use cortex_m::interrupt::Mutex;

static RX_QUEUE: Mutex<RefCell<heapless::Deque<u8, 64>>> = ...;
static TX_QUEUE: Mutex<RefCell<heapless::Deque<u8, 256>>> = ...;

impl CharIo for UartIo {
    fn get_char(&mut self) -> Result<Option<char>, Self::Error> {
        cortex_m::interrupt::free(|cs| {
            RX_QUEUE.borrow(cs).borrow_mut().pop_front()
        }).map(|b| b as char).ok_or(Error::NoData)
    }

    fn put_char(&mut self, c: char) -> Result<(), Self::Error> {
        cortex_m::interrupt::free(|cs| {
            TX_QUEUE.borrow(cs).borrow_mut().push_back(c as u8)
        }).map_err(|_| Error::BufferFull)
    }
}
```

### State Isolation

**Each CliService instance is completely independent:**
- No global state shared between instances
- Multiple instances can coexist (on different I/O channels)
- Tree structures are const (shared read-only is safe)
- Handlers are passed by value/reference (user controls sharing)

## Error Handling Strategy

```
CliError variants:
├─ BufferFull        → Input exceeds fixed buffer capacity
├─ InvalidPath       → Path doesn't exist OR access denied (security, hides node existence)
├─ InvalidArguments  → Wrong argument count or format
├─ PathTooDeep       → Exceeded MAX_PATH_DEPTH
└─ Custom(...)       → Command-specific errors

Error propagation:
1. Low-level errors (I/O) bubble up as IO::Error
2. CLI logic errors return CliError
3. Commands return Response with status code
4. Parse errors return InvalidPath or InvalidArguments
5. Access denial masqueraded as InvalidPath (security)
```

## Feature Gate Impact

| Feature | Flash Impact | RAM Impact | Affected Modules |
|---------|-------------|------------|------------------|
| `authentication` | +~2 KB | 0 bytes | auth/, cli/mod.rs (minimal) |
| `completion` | +~2 KB | 0 bytes | tree/completion.rs |
| `history` | +~0.5-0.8 KB | +1.3 KB (N=10) | cli/history.rs |
| All disabled | Baseline | Baseline | Minimal CLI only |

**Stub Pattern Benefits:**
- Authentication: Unified state machine (LoggedIn always exists)
- Completion: suggest_completions() always callable (returns empty Vec)
- History: CommandHistory type always exists (methods no-op)
- Result: Minimal `#[cfg]` branching in main CLI service code

---

## See Also

- **[DESIGN.md](DESIGN.md)** - Design decisions, rationale, and feature gating patterns
- **[SPECIFICATION.md](SPECIFICATION.md)** - Complete behavioral specification
- **[IMPLEMENTATION.md](IMPLEMENTATION.md)** - Implementation roadmap and build workflows
- **[SECURITY.md](SECURITY.md)** - Authentication and access control security
- **[PHILOSOPHY.md](PHILOSOPHY.md)** - Design philosophy and feature decision framework

---

*This document describes the runtime internals based on the design decisions documented in DESIGN.md and behavioral specifications in SPECIFICATION.md.*
