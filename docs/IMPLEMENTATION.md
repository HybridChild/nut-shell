# nut-shell library - Implementation Plan

**Status**: Implementation Phase 2 in progress  
**Estimated Timeline**: 2-3 weeks

## Overview

This document tracks the implementation phases for nut-shell. The implementation prioritizes **idiomatic Rust patterns** while maintaining behavioral correctness.

**When to use this document:**
- Finding out what phase of implementation we're in
- Understanding what needs to be built next
- Getting the complete build and validation workflow
- Checking task completion status

**Related Documentation:**
- **[DESIGN.md](DESIGN.md)**: Design decisions, command architecture, and rationale
- **[INTERNALS.md](INTERNALS.md)**: Complete runtime internals from input to output
- **[SPECIFICATION.md](SPECIFICATION.md)**: Exact behavioral requirements for each feature
- **[SECURITY.md](SECURITY.md)**: Security design for authentication features
- **[PHILOSOPHY.md](PHILOSOPHY.md)**: Design philosophy and feature decision framework
- **[../CLAUDE.md](../CLAUDE.md)**: Working patterns and practical guidance for implementing features

---

## Prerequisites: Essential Patterns

**IMPORTANT**: Before starting implementation, review these architectural patterns in DESIGN.md. Discovering these mid-implementation will require significant refactoring.

**Required reading:**
1. **Metadata/Execution Separation Pattern** ([DESIGN.md](DESIGN.md) Section 1) - Commands split into const metadata + generic trait for sync/async support
2. **Unified Architecture Pattern** ([DESIGN.md](DESIGN.md) Section 5.2) - Single code path for auth-enabled and auth-disabled modes
3. **Stub Function Pattern** ([DESIGN.md](DESIGN.md) Feature Gating sections) - Feature-gated modules with identical signatures
4. **Access Control Integration** ([INTERNALS.md](INTERNALS.md) Level 4) - Access checks during tree traversal

---

## Module Map

This shows which phase creates which file:

```
Phase 1:
  - tests/fixtures/mod.rs (MockIo, MockAccessLevel, TEST_TREE fixture)
  - src/lib.rs (initial setup with feature gates)
  - Cargo.toml (dependencies and features)

Phase 2:
  - src/io.rs (CharIo trait)
  - src/auth/mod.rs (AccessLevel trait, User struct, CredentialProvider trait, PasswordHasher trait)
  - src/auth/password.rs (Sha256Hasher implementation)
  - src/auth/providers/buildtime.rs (build-time credentials)
  - src/auth/providers/const_provider.rs (hardcoded credentials for testing)
  - src/config.rs (ShellConfig trait, DefaultConfig, MinimalConfig)
  - src/lib.rs or src/error.rs (CliError enum)

Phase 3:
  - src/tree/mod.rs (Node enum, CommandMeta struct, Directory struct, CommandKind enum)
  - src/shell/handlers.rs (CommandHandlers trait definition)
  - tests/fixtures/mod.rs (MockHandlers implementation - validates metadata/execution separation)

Phase 4:
  - src/tree/path.rs (Path type)

Phase 5:
  - src/response.rs (Response type)
  - src/shell/mod.rs (Request enum, HistoryDirection enum, CliState enum - partial)

Phase 6:
  - src/shell/parser.rs (ParseEvent enum, InputParser)
  - src/shell/history.rs (CommandHistory with stub pattern)

Phase 7:
  - src/tree/completion.rs (with stub pattern)

Phase 8:
  - src/shell/mod.rs (Shell implementation, complete)

Phase 9:
  - examples/basic.rs
  - examples/rp2040_uart.rs (optional)
```

## Implementation Phases

### Phase 1: Project Foundation ✅
**Goal**: Runnable Rust project with basic structure and testing infrastructure

**Tasks**:
- [x] Create Cargo.toml with no_std support, heapless dependency
- [x] Create src/lib.rs with feature gates and module declarations
- [x] Create directory structure (shell/, tree/ modules with placeholder files)
- [x] Create testing infrastructure:
  - `tests/fixtures/mod.rs` with `MockIo` implementation for CharIo
  - `tests/fixtures/mod.rs` with simple test tree (used across all phases)
  - `tests/fixtures/mod.rs` with `MockAccessLevel` enum (Guest/User/Admin)
- [x] Document feature flag testing approach:
  - Test with all features: `cargo test --all-features`
  - Test with no features: `cargo test --no-default-features`
  - Test selective features: `cargo test --features authentication`
- [x] Verify `cargo build` on native target
- [x] Verify `cargo build --target thumbv6m-none-eabi` on embedded target

**Success Criteria**: ✅
- Project compiles on both native and embedded targets
- MockIo available for testing CharIo implementations
- Test fixtures can be reused in subsequent phases

---

### Phase 2: I/O & Access Control Foundation
**Goal**: Core traits everything depends on

**Tasks**:
1. Implement `CharIo` trait in `io.rs` (see IO_DESIGN.md for complete design)
   - Define trait with associated error type
   - Character read/write methods:
     * `get_char(&mut self) -> Result<Option<char>, Self::Error>` - Non-blocking read
     * `put_char(&mut self, c: char) -> Result<(), Self::Error>` - Write to buffer
     * `write_str(&mut self, s: &str) -> Result<(), Self::Error>` - Default impl using put_char
   - Document buffering requirements (CRITICAL - see IO_DESIGN.md):
     * All implementations MUST buffer output internally
     * Bare-metal: May flush immediately in put_char() (blocking acceptable)
     * Async: MUST buffer to memory only, flush externally after process_char()
     * Recommended buffer sizes: 256 bytes for async platforms, 0 (immediate) for bare-metal
     * put_char() and write_str() MUST NOT await or block indefinitely
   - Create `StdioStream` implementation for testing (bare-metal pattern with immediate flush)
   - Add basic tests

2. Implement `CliError` enum in `lib.rs` or `error.rs` (foundational error type):
   - `CommandNotFound` - Command not found in tree
   - `InvalidPath` - Path doesn't exist OR user lacks access (intentionally ambiguous for security)
   - `InvalidArguments { expected_min, expected_max, received }` - Wrong argument count
   - `BufferFull` - Buffer capacity exceeded
   - `PathTooDeep` - Path exceeds MAX_PATH_DEPTH
   - `AuthenticationFailed` (feature-gated: `authentication`) - Wrong credentials
   - `NotAuthenticated` (feature-gated: `authentication`) - Tried to execute command while logged out
   - `IoError` - I/O error occurred
   - `AsyncNotSupported` (feature-gated: `async`) - Async command called in sync mode
   - `Timeout` - Operation timed out (used by command implementations)
   - `Other(heapless::String<MAX_RESPONSE>)` - Generic error with message
   - Implement `core::fmt::Display` for user-friendly error messages
   - **SECURITY NOTE**: Never create separate "AccessDenied" error - always use `InvalidPath` to hide node existence

3. Implement access control in `auth/mod.rs` (see DESIGN.md "Module Structure")
   - `AccessLevel` trait with comparison operators
   - Example implementations (e.g., enum with Admin/User/Guest)
   - `User` struct with complete field definition:
     * `username: heapless::String<32>` (always present)
     * `access_level: L` (always present)
     * `password_hash: [u8; 32]` (feature-gated: `authentication`)
     * `salt: [u8; 16]` (feature-gated: `authentication`)
   - Module is feature-gated, but `User` and `AccessLevel` are re-exported at root level (always available)
   - Unit tests

4. Implement authentication infrastructure in `auth/mod.rs` (see SECURITY.md):
   - `CredentialProvider` trait (requires authentication feature):
     * `type Error` - Associated error type
     * `find_user(&self, username: &str) -> Result<Option<User<L>>, Self::Error>`
     * `verify_password(&self, user: &User<L>, password: &str) -> bool`
     * `list_users(&self) -> Result<Vec<&str>, Self::Error>`
   - `PasswordHasher` trait:
     * `hash(&self, password: &str, salt: &[u8]) -> [u8; 32]`
     * `verify(&self, password: &str, salt: &[u8], hash: &[u8; 32]) -> bool`

5. Implement password hashing in `auth/password.rs`:
   - `Sha256Hasher` struct implementing `PasswordHasher` trait
   - SHA-256 hashing using `sha2` crate
   - Constant-time password verification using `subtle::ConstantTimeEq`
   - Salt handling (16-byte salts prepended to password before hashing)
   - Unit tests verifying constant-time comparison

6. Create credential provider implementations in `auth/providers/`:
   - `buildtime.rs` - Build-time environment variables (production use)
   - `const_provider.rs` - Hardcoded credentials (examples/testing ONLY)
   - Note: Flash storage provider can be added later as needed

7. Implement configuration in `config.rs` (see TYPE_REFERENCE.md "Configuration")
   - `ShellConfig` trait with associated constants (MAX_INPUT, MAX_PATH_DEPTH, MAX_ARGS, MAX_PROMPT, MAX_RESPONSE, HISTORY_SIZE)
   - `DefaultConfig` struct (balanced for typical embedded systems: 128/8/16/64/256/10)
   - `MinimalConfig` struct (resource-constrained systems: 64/4/8/32/128/5)
   - All constants are compile-time evaluated (zero runtime cost)
   - Unit tests

**Success Criteria**: Can abstract I/O, access control, configuration, and errors with zero runtime cost

---

### Phase 3a: Tree Core Types
**Goal**: Define foundational tree types with access control

**IMPORTANT**: This phase implements the **metadata/execution separation pattern** (see DESIGN.md Section 1). Commands are split into `CommandMeta` (const metadata in ROM) and execution logic (provided via `CommandHandlers` trait in Phase 8).

**Tasks**:
1. Implement in `tree/mod.rs`:
   - `Node` enum with Command and Directory variants
   - `CommandMeta` struct (metadata only, no execute field):
     - `name: &'static str`
     - `description: &'static str`
     - `access_level: L` (generic over AccessLevel)
     - `kind: CommandKind` (enum: Sync or Async marker)
     - `min_args: usize`, `max_args: usize`
   - `CommandKind` enum:
     - `Sync` - Synchronous command
     - `Async` - Asynchronous command (requires `async` feature)
   - `Directory` struct:
     - `name: &'static str`
     - `children: &'static [Node<L>]` (array reference)
     - `access_level: L`
   - Type checking methods: `is_command()`, `is_directory()`, `name()`, `access_level()`

2. Implement `CommandHandlers` trait in `shell/handlers.rs`:
   - Generic over `C: ShellConfig` (for Response buffer sizing)
   - `execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>`
   - `execute_async(&self, name: &str, args: &[&str]) -> Result<Response<C>, CliError>` (feature-gated: `async`)
   - See TYPE_REFERENCE.md for complete trait definition

3. **CRITICAL: Validate metadata/execution separation pattern early**:
   - Create `MockHandlers` test fixture in `tests/fixtures/mod.rs` implementing `CommandHandlers<DefaultConfig>`
   - Implement 2-3 test commands:
     ```rust
     struct MockHandlers;

     impl CommandHandlers<DefaultConfig> for MockHandlers {
         fn execute_sync(&self, name: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
             match name {
                 "echo" => {
                     let msg = args.join(" ");
                     Ok(Response::success(&msg))
                 }
                 "fail" => Ok(Response::error("Test error")),
                 "reboot" => Ok(Response::success("Rebooting...")),
                 _ => Err(CliError::CommandNotFound),
             }
         }

         #[cfg(feature = "async")]
         async fn execute_async(&self, name: &str, args: &[&str]) -> Result<Response<DefaultConfig>, CliError> {
             match name {
                 "async-wait" => {
                     // Simulate async operation
                     Ok(Response::success("Async complete"))
                 }
                 _ => Err(CliError::CommandNotFound),
             }
         }
     }
     ```
   - Create corresponding const `CommandMeta` instances in TEST_TREE
   - Write integration test that:
     * Validates const metadata compiles
     * Verifies handlers can be instantiated
     * Confirms metadata and execution are properly separated
     * Tests that async trait method compiles (even without awaiting yet)
   - **Async validation** (when `async` feature enabled):
     * Write test that calls `execute_async()` and verifies it compiles
     * Use a simple async runtime or `futures::executor::block_on()` for testing
     * Verify async methods return correct Response types
     * Test that `CommandKind::Async` marker works as expected
     * **Why now?** Async trait method issues won't surface until Phase 8 otherwise - discovering async compilation problems early saves significant refactoring
   - **Why validate now?** This pattern is foundational. If there are issues with const initialization + generic traits, we need to discover them BEFORE building Shell in Phase 8.

4. Unit tests for type construction and pattern matching

**Success Criteria**:
- Can define individual CommandMeta and Directory instances
- Node enum enables zero-cost dispatch via pattern matching
- Access level integration works with generic parameter
- CommandMeta is const-initializable (no function pointers, metadata only)
- CommandHandlers trait compiles with both sync and async methods
- MockHandlers proves the metadata/execution separation pattern works

---

### Phase 3b: Tree Const Initialization
**Goal**: Build const-initializable tree structures in ROM

**Tasks**:
1. Implement const tree construction patterns:
   - Create const `CommandMeta` definitions (metadata only, no execute functions)
   - Create const `Directory` definitions with child arrays
   - Nest directories to create hierarchical structure
   - Example: `/system/reboot`, `/hw/led/set`, etc.
   - Note: Actual command execution functions are implemented later via `CommandHandlers` trait (Phase 8)

2. Create example tree as test fixture in `tests/fixtures/mod.rs`:
   ```rust
   pub const TEST_TREE: Directory<MockAccessLevel> = Directory {
       name: "/",
       children: &[/* ... */],
       access_level: MockAccessLevel::Guest,
   };
   ```

3. Verify const initialization with integration test
4. Verify tree can be placed in ROM (check with `nm` or `objdump`)

**Success Criteria**:
- Tree lives in ROM with zero runtime initialization
- Can construct complex nested tree structures at compile time
- Test fixture available for use in subsequent phases

---

### Phase 4: Path Navigation
**Goal**: Unix-style path resolution with access control integration

**Tasks**:
1. Implement `Path` type in `tree/path.rs`:
   - Parse absolute paths (`/foo/bar`)
   - Parse relative paths (`../foo`, `./bar`, `bar`)
   - Handle ".." (parent) and "." (current) components
   - Path normalization
   - Component iteration
   - Implement path parsing (~190 lines)

2. Add path resolution to `Directory` in `tree/mod.rs`:
   - `find_child(&self, name: &str) -> Option<&Node<L>>`
   - Basic tree walking without access control
   - Use index stack pattern: push child indices, pop for parent
   - Walk tree using stored indices

3. **IMPORTANT**: Prepare for access control integration (implemented in Phase 8):
   - Path resolution will need `current_user` context
   - Access checks happen at EVERY segment during traversal
   - Security principle: Return `Err(CliError::InvalidPath)` for both non-existent and inaccessible nodes
   - This prevents revealing existence of restricted commands
   - Note: Full integration happens in Phase 8 when `Shell::resolve_path()` is implemented

4. Comprehensive tests:
   - Path parsing edge cases
   - Parent navigation (`..`)
   - Absolute vs relative paths
   - Invalid paths return None
   - Deep tree navigation
   - Document placeholder for access control tests (added in Phase 8)

**Success Criteria**:
- Can navigate tree with complex paths like `../system/debug`
- Path resolution methods ready for access control integration
- Understand that full security integration requires `current_user` context (Phase 8)

---

### Phase 5: Request/Response Types
**Goal**: Type-safe command processing types (MUST complete before Phase 6)

**Why this phase comes first**: Phase 6 (Input Processing) needs to convert input buffers into `Request` types, so these types must exist first.

**Tasks**:
1. Implement `HistoryDirection` enum in `shell/mod.rs`:
   - `Previous = 0` - Up arrow key (navigate to older command)
   - `Next = 1` - Down arrow key (navigate to newer command or restore original)
   - Used by `Request::History` variant
   - Size: 1 byte (repr(u8) for efficiency)
   - Self-documenting alternative to bool

2. Implement `Request<C: ShellConfig>` enum in `shell/mod.rs`:

   **IMPORTANT**: Request is generic over `C: ShellConfig` to use configured buffer sizes.
   This enables per-deployment buffer customization without recompilation.
   - `path` fields use `C::MAX_INPUT`
   - `args` uses `C::MAX_ARGS`
   - `buffer` fields use `C::MAX_INPUT`

   **Variants**:
   - `Login { username, password }` - Authentication attempt (feature-gated: `authentication`)
     * `username: heapless::String<32>`
     * `password: heapless::String<64>`
   - `InvalidLogin` - Failed login (feature-gated: `authentication`)
   - `Command { path, args, #[cfg] original }` - Execute command
     * `path: heapless::String<C::MAX_INPUT>`
     * `args: heapless::Vec<heapless::String<C::MAX_INPUT>, C::MAX_ARGS>`
     * `original: heapless::String<C::MAX_INPUT>` (feature-gated: `history`)
     * `original` field saves ~128 bytes RAM when history disabled
   - `TabComplete { path }` - Request completions (feature-gated: `completion`)
     * `path: heapless::String<C::MAX_INPUT>`
   - `History { direction, buffer }` - Navigate history (feature-gated: `history`)
     * `direction: HistoryDirection`
     * `buffer: heapless::String<C::MAX_INPUT>`
   - See TYPE_REFERENCE.md for complete type definition and usage patterns

3. Implement `CliState` enum in `shell/mod.rs`:
   - `Inactive` - CLI not active
   - `LoggedOut` - Awaiting authentication (feature-gated variant)
   - `LoggedIn` - Authenticated or auth-disabled mode

4. Implement `Response<C: ShellConfig>` in `response.rs`:

   **IMPORTANT**: Response is generic over `C: ShellConfig` for buffer sizing.
   Message uses `C::MAX_RESPONSE` buffer size.

   **Fields**:
   - Success/error variants
   - Formatting flags:
     - `inline_message` - Message is inline (don't echo newline after command input)
     - `prefix_newline` - Add newline before message (in response formatter)
     - `indent_message` - Indent output (2 spaces)
     - `postfix_newline` - Add newline after message
     - `show_prompt` - Display prompt after response
     - `exclude_from_history` - Prevent input from being saved to history (feature-gated: `history`)
   - Helper constructors: `Response::success()`, `Response::error()`, `Response::success_no_history()` (feature-gated)
   - Builder method: `without_history()` - Chain to exclude from history (feature-gated)
   - Message content and status code
   - See INTERNALS.md Level 7 for complete response formatting
   - Message uses `C::MAX_RESPONSE` buffer size
   - Implementation example:
     ```rust
     pub struct Response<C: ShellConfig> {
         pub message: heapless::String<C::MAX_RESPONSE>,
         pub is_success: bool,
         pub inline_message: bool,
         pub prefix_newline: bool,
         pub indent_message: bool,
         pub postfix_newline: bool,
         pub show_prompt: bool,
         #[cfg(feature = "history")]
         pub exclude_from_history: bool,
     }

     impl<C: ShellConfig> Response<C> {
         pub fn success(message: &str) -> Self { /* default: include in history */ }
         pub fn error(message: &str) -> Self { /* default: include in history */ }

         #[cfg(feature = "history")]
         pub fn success_no_history(message: &str) -> Self { /* exclude from history */ }

         #[cfg(feature = "history")]
         pub fn without_history(mut self) -> Self {
             self.exclude_from_history = true;
             self
         }
     }
     ```
   - Shell integration: Check `exclude_from_history` before calling `history.add()` (see Phase 6)

5. Tests for request/response handling

**Success Criteria**:
- Can represent all CLI operations type-safely
- Response type supports all formatting modes needed by global commands and custom commands
- Input Parser (Phase 6) can convert buffers to Request types

---

### Phase 6: Input Processing
**Goal**: Terminal I/O with escape sequences

**Tasks**:
1. Implement `ParseEvent` enum in `shell/parser.rs`:
   - `None` - No special action needed
   - `CharAdded(char)` - Character added to buffer
   - `Backspace` - Backspace pressed (remove last char)
   - `Enter` - Enter pressed (input complete)
   - `Tab` - Tab pressed (trigger completion)
   - `UpArrow` - Up arrow (history previous)
   - `DownArrow` - Down arrow (history next)
   - `ClearAndRedraw` - Double-ESC (clear buffer and exit history)
   - Returned by `InputParser::process_char()` to indicate what happened

2. Implement `InputParser` in `shell/parser.rs`:
   - Character-by-character processing
   - Escape sequence state machine (up/down arrows, double-ESC)
   - Double-ESC clear buffer (always enabled, ~50-100 bytes, see PHILOSOPHY.md)
   - Backspace and delete handling
   - Tab key detection
   - Password masking mode for login
   - Buffer management with `heapless::String`
   - Convert buffer to Request when complete
   - Implement input parser (~397 lines)
   - Note: Left/right arrows, Home/End keys are future additions (see PHILOSOPHY.md "Recommended Additions")

3. Implement `CommandHistory<const N: usize, const INPUT_SIZE: usize>` in `shell/history.rs` using stub type pattern (see DESIGN.md "Feature Gating & Optional Features"):
   - Circular buffer with two const generics: N (history size), INPUT_SIZE (buffer size per entry)
   - O(1) add, previous, next operations
   - Position tracking for navigation
   - Implement command history (~85 lines)
   - Feature-gated: Type always exists, methods no-op when `history` feature disabled
   - Zero-size stub type when disabled
   - Used in Shell as: `CommandHistory<C::HISTORY_SIZE, C::MAX_INPUT>` where C: ShellConfig
   - **Shell integration**: After command execution, check `Response.exclude_from_history` flag before calling `history.add()`:
     ```rust
     #[cfg(feature = "history")]
     if !response.exclude_from_history {
         self.history.add(&self.input_buffer);
     }
     ```
   - This allows commands handling sensitive data (passwords, credentials) to prevent their input from being recorded

4. Comprehensive tests:
   - Escape sequence parsing (up/down arrows, double-ESC)
   - Double-ESC clears buffer and exits history navigation
   - ESC + [ starts escape sequence (not cleared)
   - Backspace in middle of line
   - History navigation
   - Password masking
   - Buffer overflow handling

**Success Criteria**:
- Correctly parse all terminal input
- Handle arrows, backspace, tab, double-ESC
- Double-ESC clears input buffer without clearing screen
- O(1) history operations

---

### ⚡ Checkpoint: Type-Level Integration Validation

**At this point, all core types exist.** Before proceeding to Phase 7 (Tab Completion) and Phase 8 (Shell), validate that the type system integrates correctly.

**Why checkpoint here?**
- All foundational types are implemented: CharIo, AccessLevel, User, Node, Path, Request, Response, InputParser, CommandHistory
- Phase 7 adds an optional feature (tab completion)
- Phase 8 brings everything together in Shell
- Better to discover type integration issues NOW than during Shell implementation

**Validation Tasks:**
1. **Create integration test** in `tests/integration/type_validation.rs`:
   - Instantiate all core types together in a single test
   - Create a mock command tree with various access levels
   - Parse a path and resolve it through the tree
   - Create Request instances and convert to Response
   - Verify all generic parameters (L, IO, H, C) work together
   - Test both `DefaultConfig` and `MinimalConfig`

2. **Verify compilation** across feature combinations:
   ```bash
   cargo test --all-features
   cargo test --no-default-features
   cargo test --features authentication
   cargo test --features history
   ```

3. **Check type-level constraints**:
   - Verify `CommandMeta` is const-initializable
   - Confirm `CommandHandlers` trait object safety (if needed)
   - Test lifetime relationships between tree and Shell components
   - Validate generic parameter inference works naturally

4. **Success Criteria**:
   - All types instantiate without compilation errors
   - Generic parameters infer correctly in typical usage
   - No lifetime conflicts between tree and runtime state
   - Feature combinations compile cleanly
   - **If issues found**: Refactor types NOW before Shell implementation

**Time Investment**: 1-2 hours. **Value**: Prevents 4-8 hours of refactoring during Phase 8.

---

### Phase 7: Tab Completion
**Goal**: Smart command/path completion (optional feature)

**Note**: Tab completion grouped here with other input/interaction features (parser in Phase 6, Shell in Phase 8) for logical cohesion.

**Tasks**:
1. Implement in `tree/completion.rs`:
   - Prefix matching for commands and directories
   - Return multiple matches when ambiguous
   - Auto-append "/" for directories
   - Handle partial path completion (`sys/de<TAB>` → `system/debug`)
   - Implement completion logic (~229 lines)

2. Implement feature gating using stub function pattern (see DESIGN.md "Feature Gating & Optional Features"):
   - Add `completion` feature flag to Cargo.toml
   - Add `#[cfg(feature = "completion")]` conditional compilation within module contents
   - Implement stub function pattern: `suggest_completions()` returns empty `Vec` when disabled
   - Module always exists, contents are feature-gated
   - Single `handle_tab()` implementation calls stub functions (no dual methods needed)
   - Parser handles tab key identically in both modes (stub returns empty results)

3. Tests for completion scenarios:
   - Single match completion
   - Multiple match display
   - No matches
   - Directory vs command completion
   - Test builds with feature enabled/disabled
   - Verify no_std compliance with feature disabled
   - Measure code size impact (should be ~2KB)

**Success Criteria**:
- Tab completion works for partial names with proper directory handling
- Feature can be disabled via `--no-default-features` flag
- Graceful degradation when completion disabled
- Code size savings measurable (~2KB)

---

### Phase 8: Shell Orchestration
**Goal**: Bring it all together with unified architecture

**Tasks**:
1. Implement `Shell` struct in `shell/mod.rs` using unified architecture pattern:

   a. **Field definitions**:
   ```rust
   pub struct Shell<'tree, L, IO, H, C>
   where
       L: AccessLevel,
       IO: CharIo,
       H: CommandHandlers<C>,
       C: ShellConfig,
   {
       // ALWAYS present (not feature-gated)
       tree: &'tree Directory<L>,
       current_user: Option<User<L>>,
       state: CliState,
       input_buffer: heapless::String<C::MAX_INPUT>,
       current_path: heapless::Vec<usize, C::MAX_PATH_DEPTH>,
       parser: InputParser,
       history: CommandHistory<C::HISTORY_SIZE, C::MAX_INPUT>,
       io: IO,
       handlers: H,

       // ONLY this field is feature-gated
       #[cfg(feature = "authentication")]
       credential_provider: &'tree dyn CredentialProvider<L>,

       // Config type marker (zero-size)
       _config: core::marker::PhantomData<C>,
   }
   ```

   b. **Feature-conditional constructors**:
   ```rust
   // Constructor when authentication enabled
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
           provider: &'tree dyn CredentialProvider<L>,
           io: IO,
       ) -> Self {
           Self {
               tree,
               handlers,
               current_user: None,  // Start logged out
               state: CliState::LoggedIn,
               credential_provider: provider,
               _config: core::marker::PhantomData,
               // ... other fields
           }
       }
   }

   // Constructor when authentication disabled
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
           Self {
               tree,
               handlers,
               current_user: None,  // No user needed
               state: CliState::LoggedIn,  // Start in logged-in state
               _config: core::marker::PhantomData,
               // ... other fields
           }
       }
   }
   ```

   c. **Core methods** (same implementation for both modes):
   - `activate()` - Show welcome message, initial prompt
   - `process_char()` - Main character processing loop
   - `generate_prompt()` - Create `username@path> ` prompt (unified for both modes)
   - `resolve_path()` - Path navigation with access control checks at each segment
   - `execute_command()` - Run command with argument validation

   d. **Global commands**:
   - `?` - List global commands
   - `ls` - Show current directory contents with descriptions
   - `logout` - End session (only available when authentication enabled)
   - `clear` - Clear screen (platform-dependent)

   e. **Integration with optional features** (using stub patterns):
   - Tab completion: calls `completion::suggest_completions()` (returns empty when disabled)
   - History navigation: calls `history.previous()`/`history.next()` (no-op when disabled)

   f. **Implement Shell orchestration** (~589 lines total)

   **Note on omitted commands**: No `cd`, `pwd`, or `tree` commands per path-based syntax design (see DESIGN.md).
   Path-based navigation makes them redundant:
   - Instead of `cd system`: just type `system`
   - Instead of `pwd`: current path shown in prompt (`user@/current/path>`)
   - Instead of `tree`: use `ls` to explore directory structure

2. Integration tests with mock I/O:
   - Login flow (auth enabled)
   - Navigation between directories
   - Command execution
   - Access control enforcement
   - Tab completion (both enabled and disabled via stubs)
   - History navigation (both enabled and disabled via stubs)
   - Test unified architecture: auth-enabled vs auth-disabled modes
   - Test feature combinations: all features, no features, individual features

3. Async feature testing (when `async` feature enabled):
   - **Async command execution**:
     * Create test handlers with both sync and async commands
     * Verify `process_char_async()` correctly awaits async commands
     * Test that async commands complete before returning
     * Verify output is generated correctly after async command completes
   - **Sync commands in async mode**:
     * Verify sync commands still work when using `process_char_async()`
     * Test mixed command trees (some sync, some async)
   - **Error handling**:
     * Verify `AsyncNotSupported` error when calling sync `process_char()` with async command
     * Test error propagation from async commands (via `?` operator)
   - **I/O buffering with async**:
     * Verify CharIo buffer is flushed after async command completes
     * Test buffer overflow handling during long async operations
   - **Command metadata validation**:
     * Verify `CommandKind::Async` properly routes to `execute_async()` handler
     * Verify `CommandKind::Sync` routes to `execute_sync()` in async mode

**Success Criteria**:
- End-to-end CLI functionality works with all feature combinations
- Unified architecture correctly handles both auth modes
- Stub patterns enable graceful degradation when features disabled

---

### Phase 9: Examples
**Goal**: Demonstrate usage

**Tasks**:
1. Create `examples/basic.rs`:
   - Native stdio CLI
   - Example command tree (system commands, config, etc.)
   - Simple commands (echo, reboot, version, etc.)
   - Interactive session
   - Proper error handling

2. Create `examples/rp2040_uart.rs` (optional):
   - RP2040-specific UART I/O implementation
   - Minimal command tree for embedded
   - Hardware initialization
   - Verify on actual Pico hardware

3. Add documentation comments showing example usage

**Success Criteria**: Can run interactive CLI session with examples

---

### Phase 10: Testing & Polish
**Goal**: Match target quality and functionality

**Tasks**:
1. Write comprehensive tests:
   - Tree operations test
   - CLI test
   - Input parser test
   - Tab completion test
   - Command history test

2. Add Rust-specific tests:
   - Const initialization validation
   - Lifetime safety (compile tests)
   - Zero-size-type optimization checks
   - ROM placement verification

3. Documentation pass:
   - Module-level docs
   - Public API docs
   - Examples in docs
   - Architecture decision records

4. Performance validation:
   - Memory usage profiling
   - Stack usage analysis
   - Verify ROM placement
   - Measure baseline performance

5. Create README.md:
   - Project overview
   - Quick start guide
   - API examples
   - Build instructions
   - Performance characteristics

**Success Criteria**: Comprehensive test coverage, quality documentation

---

## Workflow Best Practices

### Test-Driven Development

For each phase:
1. **Write tests first** based on behavioral specification (see SPECIFICATION.md)
2. **Implement minimal functionality** to pass tests
3. **Iterate** until all tests pass
4. **Refine and optimize** with confidence
5. **Document** public APIs
6. **Commit** working increments

### Testing Strategy

**Unit Tests**: Per module, test individual components
- Path parsing: `path.rs`
- Tree navigation: `tree/mod.rs`
- History operations: `history.rs`
- Parser state machine: `parser.rs`

**Integration Tests**: End-to-end CLI functionality
- Login flow
- Command execution
- Navigation
- Tab completion
- History navigation

**Embedded Tests**: Platform-specific validation
- ROM placement verification
- Stack usage analysis
- Actual hardware testing (Pico)

### Build & Validation Commands - Complete Workflows

**Note:** For a quick reference, see CLAUDE.md "Common Build Commands"

#### Quick Iteration (Development)
```bash
cargo check                              # Fast compile check
cargo test                               # Run all tests
cargo test test_name                     # Run specific test
cargo clippy                             # Lint
cargo fmt                                # Format
cargo run --example basic                # Manual testing
```

#### Feature Validation
```bash
# Test all feature combinations
cargo test --all-features
cargo test --no-default-features
cargo test --no-default-features --features authentication
cargo test --no-default-features --features completion

# Verify compilation with specific features
cargo check --features authentication
cargo clippy --features completion
```

#### Embedded Target Verification
```bash
# Verify no_std compliance
cargo check --target thumbv6m-none-eabi

# Build for embedded (various configurations)
cargo build --target thumbv6m-none-eabi --release
cargo build --target thumbv6m-none-eabi --release --no-default-features
cargo build --target thumbv6m-none-eabi --release --features authentication

# Measure and compare binary sizes
cargo size --target thumbv6m-none-eabi --release -- -A
cargo size --target thumbv6m-none-eabi --release --no-default-features -- -A
```

#### Pre-Commit Validation
```bash
# Full check (one-liner)
cargo fmt && \
cargo clippy --all-features -- -D warnings && \
cargo test --all-features && \
cargo check --target thumbv6m-none-eabi --release

# Or step-by-step:
cargo fmt                                          # 1. Format
cargo check --all-features                         # 2. Compile check
cargo clippy --all-features -- -D warnings         # 3. Lint
cargo test --all-features                          # 4. Test
cargo check --target thumbv6m-none-eabi --release  # 5. Embedded check
cargo doc --no-deps --all-features                 # 6. Doc check
```

#### CI Simulation (Full Validation)
```bash
# All feature combinations
cargo test --all-features
cargo test --no-default-features
cargo test --features authentication
cargo test --features completion

cargo build --all-features
cargo build --no-default-features
cargo build --features authentication
cargo build --features completion

# Embedded builds
cargo build --target thumbv6m-none-eabi --release --all-features
cargo build --target thumbv6m-none-eabi --release --no-default-features

# Quality checks
cargo fmt -- --check
cargo clippy --all-features -- -D warnings
cargo clippy --no-default-features -- -D warnings
cargo doc --no-deps --all-features
```

#### Troubleshooting
```bash
cargo build -vv                          # Verbose build output
cargo tree                               # Show dependency tree
cargo tree --target thumbv6m-none-eabi   # Embedded dependencies
cargo tree --format "{p} {f}"            # Show feature resolution
cargo clean && cargo build               # Clean rebuild
cargo expand --lib                       # Expand macros
```

## Current Status

### Completed
- ✅ Architecture analysis and simplification (documented in CLAUDE.md)
- ✅ Implementation plan documentation
- ✅ Documentation structure refactored (CLAUDE.md = permanent, IMPLEMENTATION.md = task tracking)
- ✅ Phase 1: Project Foundation
  - Complete project structure (Cargo.toml, src/, tests/)
  - All core module placeholders created
  - Test fixtures implemented (MockIo, MockAccessLevel, TEST_TREE)
  - Feature flag testing documented
  - Verified builds on native and embedded targets (thumbv6m-none-eabi)
  - All tests passing (24 total)

### In Progress
- 🟡 Phase 2: I/O & Access Control Foundation (ready to start)

### Upcoming
- ⬜ Phase 3: Tree Data Model
- ⬜ Phase 4: Path Navigation
- ⬜ Phase 5: Request/Response Types
- ⬜ Phase 6: Input Processing
- ⬜ ⚡ Checkpoint: Type-Level Integration Validation
- ⬜ Phase 7: Tab Completion
- ⬜ Phase 8: Shell Orchestration
- ⬜ Phase 9: Examples
- ⬜ Phase 10: Testing & Polish

## Notes

- **Update this document** as implementation progresses (task completion status only)
- **Track blockers** and design questions as they arise
- **Archive when complete** (move to docs/ or delete) - this is a temporary tracking document
- **Reference CLAUDE.md** for architecture decisions and design rationale
- **Reference SPECIFICATION.md** for behavioral requirements
