# cli-service Rust Port - Architecture

This document records the architectural decisions made when porting cli-service from C++ to Rust. It explains the rationale behind structural choices and documents alternatives considered.

**Key Principle**: Port C++ *behavior*, not *structure*.

## C++ Complexity Assessment

The original C++ implementation contains ~25 files with 1345 lines of core logic. Analysis reveals two categories of complexity:

**Actually Complex Components** (irreducible complexity):
- **InputParser** (~250 lines): Escape sequences, password masking, buffer management
- **Path normalization** (~70 lines): ".." handling, absolute/relative conversion
- **Tab completion** (~150 lines): Prefix matching, multi-option display
- **CLIService orchestration** (~300 lines): Request dispatch, command execution

**C++-Specific Artifacts** (can be simplified in Rust):
- Request class hierarchy (5+ classes with virtual dispatch)
- PathResolver as separate class (83 lines, just 3 methods)
- PathCompleter with only static methods
- CLIState as separate file (3-variant enum, 14 lines)
- Separate Node/Directory/Command files

## Architectural Simplifications

The Rust implementation deliberately simplifies the C++ architecture by leveraging Rust idioms:

| C++ Pattern | Rust Simplification | Impact |
|-------------|-------------------|--------|
| 5+ Request classes + virtual dispatch | Single `Request` enum | Eliminates inheritance, dynamic_cast chains |
| Separate PathResolver class | Methods on `Directory` and `Path` | Reduces indirection, fewer files |
| PathCompleter class (static methods) | Free functions in `completion` module | No class wrapper needed |
| Separate CLIState file | Inline in `cli/mod.rs` | Too small for separate file |
| Split node/directory/command files | Combined in `tree/mod.rs` | Related const-init concerns |

**Result**: ~25 C++ files → ~14 Rust files (8 core + optional features), more maintainable codebase

## Command Syntax

The CLI uses a path-based syntax that mirrors filesystem navigation, optimized for embedded systems with minimal parsing overhead.

### Core Syntax Rules

**Navigation** (both absolute and relative):
```
/> system              # Navigate to directory (relative)
/system> network       # Navigate to subdirectory (relative)
/system/network> ..    # Navigate to parent directory
/system> /hw/led       # Navigate using absolute path
/system> /             # Navigate to root
```

**Command Execution** (both absolute and relative):
```
/system> reboot        # Execute command in current directory (relative)
/> system/reboot       # Execute command using absolute path
/hw/led> set 255 0 0   # Execute with positional arguments
/> hw/led/set 255 0 0  # Execute with args using absolute path
```

**Global Commands** (reserved keywords):
```
?         # Show current directory contents with descriptions
help      # List available global commands
logout    # End session (only when authentication feature enabled)
clear     # Clear screen (optional, platform-dependent)
```

### Disambiguation Rules

1. **Path resolution**: Parse input as path + optional arguments
2. **Tree lookup**: Walk tree structure to resolve path
3. **Node type determines behavior**:
   - If path resolves to `Node::Directory` → navigate to that directory
   - If path resolves to `Node::Command` → execute that command
   - If path matches reserved keyword → execute global command
4. **Validation**: No command or directory may use reserved keyword names (enforced at tree construction)

### Design Rationale

**Why path-based navigation?**
- Natural for hierarchical structures
- Enables both quick navigation (`system`) and direct access (`system/network/status`)
- Scriptable over serial connection
- Minimal parser complexity (~50 lines)

**Why no trailing slash convention?**
- Tree structure unambiguously determines if path is directory or command
- Less typing on slow serial connections
- Simpler mental model

**Why reserved keywords instead of prefixed globals?**
- Using `/tree` would conflict with absolute path syntax (`/system/tree`)
- Small reserved word list (3-4 keywords) validated at compile time
- Matches C++ implementation's proven approach

**Why no `tree` global command?**
- Engineers typically know structure (defined in code)
- Tab completion + `?` command sufficient for exploration
- Saves ~50-100 lines of tree rendering code
- Can be added later as optional feature if needed

**Why positional arguments only?**
- No `--flags` or `-options` reduces parser complexity
- Embedded systems typically have simple command signatures
- Fixed argument counts validated per command
- Familiar to engineers (like embedded command protocols)

### Parsing Implementation

```rust
// Pseudocode for input processing
fn parse_input(input: &str) -> Result<Request, ParseError> {
    let (path_str, args) = split_on_whitespace(input);

    // 1. Check reserved keywords first
    match path_str {
        "help" | "?" | "logout" | "clear" => return global_command(path_str),
        _ => {}
    }

    // 2. Parse as path
    let path = Path::parse(path_str)?;

    // 3. Resolve against tree
    match current_dir.resolve_path(&path)? {
        Node::Directory(_) => Request::Navigate(path),
        Node::Command(_) => Request::Execute(path, args),
    }
}
```

**Zero allocation**: All parsing uses fixed-size `heapless::Vec` buffers, no heap required.

### Comparison with C++ Implementation

The Rust syntax **matches the C++ implementation's core design** with minor refinements:

**Shared Design** (preserved from C++):
- Path-based navigation without `cd` command (just type directory name)
- Positional arguments only (no `--flags`)
- Reserved global keywords: `?` (context help), `help`, `logout`, `clear`
- Parent navigation via `..`
- Both absolute and relative path support
- No trailing slash convention

**Rust Refinements**:
- **No `tree` global command**: C++ includes it, but for embedded Rust it's better to explore via `?` + tab completion. Can be added later as optional feature if needed.
- **Validation at compile time**: Reserved keywords enforced during tree construction in Rust vs runtime checking in C++
- **Simpler parser**: Leveraging Rust's pattern matching reduces parsing complexity

See `CLIService/README.md` for C++ syntax examples.

## Key Design Decisions

### 1. Path Resolution Location
**Decision**: Methods on `Directory` (`resolve_path`) and `Path` (parsing)

**Rationale**: Emphasizes tree navigation, avoids separate class for 83 lines

**Alternative Considered**: Separate PathResolver (C++ approach)

### 2. Request Type Structure
**Decision**: Single enum in `cli/mod.rs`

**Rationale**: Pattern matching replaces inheritance, reduces files

**Alternative Considered**: Multiple types (C++ approach)

### 3. Node Polymorphism
**Decision**: Enum with Command/Directory variants

**Rationale**: Zero-cost dispatch, enables const initialization

**Alternative Considered**: Trait objects (runtime overhead, no const init)

### 4. Authentication System
**Decision**: Optional authentication with trait-based credential providers

**Rationale**: Different deployments have different security requirements:
- Development/debugging environments may not need authentication
- Production embedded systems require secure access control
- Flexibility needed for various credential storage backends (build-time, flash, external)

**Key Design Elements**:
- Generic `AccessLevel` trait allows user-defined permission hierarchies
- `CredentialProvider` trait enables pluggable authentication backends
- Password hashing (SHA-256 with salts) instead of plaintext storage
- User credentials never hardcoded in source code

**Alternative Considered**: Mandatory authentication (C++ has hardcoded auth)

**Feature Gating**: Authentication is optional and can be disabled via Cargo features for unsecured development environments or when authentication is handled externally. When disabled, access control checks are eliminated and all commands are accessible. Estimated code savings: ~2KB for core auth logic, plus dependencies (sha2, subtle). See `SECURITY.md` for comprehensive security design, credential storage options, and feature configuration patterns.

### 5. Completion Implementation
**Decision**: Free functions or trait methods in `completion` module

**Rationale**: No state needed, avoid class wrapper

**Alternative Considered**: Separate Completer type (C++ approach)

**Feature Gating**: Tab completion is optional and can be disabled via Cargo features to reduce code size (~2KB) in constrained environments. When disabled, the entire `completion` module is eliminated at compile time with zero runtime overhead. See `SECURITY.md` "Feature Gating & Optional Features" section for detailed configuration patterns and use cases

### 6. State Management
**Decision**: Inline `CliState` enum in `cli/mod.rs`

**Rationale**: Only 3 variants, too small for separate file

**Alternative Considered**: Separate state.rs file

## Implementation Benefits

These architectural choices provide:

- **Zero-cost I/O abstraction**: Compile-time monomorphization vs runtime vtable dispatch
- **ROM-based trees**: Const-initialized directory structures placed in flash memory
- **O(1) history operations**: Circular buffer vs C++ O(n) erase-from-front
- **Zero-copy parsing**: Input parsed as string slices, no per-argument allocation
- **Lifetime safety**: Compiler prevents dangling references, no manual lifetime management
- **No runtime init**: Tree structures ready at compile time, zero initialization overhead
- **Simplified architecture**: Enums replace class hierarchies, reducing ~25 C++ files to ~14 Rust files (8 core, 6 optional features)

## Module Structure

**Simplified structure (~14 files with all features vs C++'s 25):**

```
src/
├── lib.rs              # Public API and feature gates
├── cli/
│   ├── mod.rs          # CliService + Request enum + CliState enum
│   ├── parser.rs       # InputParser (escape sequences, line editing)
│   └── history.rs      # CommandHistory (circular buffer)
├── tree/
│   ├── mod.rs          # Node enum + Directory + Command structs
│   ├── path.rs         # Path type + resolution methods
│   └── completion.rs   # Tab completion logic (optional, feature-gated)
├── auth/               # Authentication module (optional, feature-gated)
│   ├── mod.rs          # User + AccessLevel trait + CredentialProvider trait
│   ├── hasher.rs       # Password hashing (SHA-256)
│   └── providers/      # Credential storage backends
│       ├── buildtime.rs    # Build-time environment variables
│       ├── flash.rs        # Flash storage (RP2040)
│       └── const_provider.rs  # Hardcoded (examples/testing only)
├── response.rs         # Response type + formatting
└── io.rs               # CharIo trait
```

**Rationale for consolidation:**
- **Request types**: Single enum replaces class hierarchy (5 classes → 1 enum)
- **State management**: Inline in cli/mod.rs (too small for separate file)
- **Path resolution**: Methods on existing types (no separate PathResolver class)
- **Tree types**: Combined in tree/mod.rs (related const-init concerns)
- **Authentication**: Trait-based system in auth/ module (optional, pluggable backends)
- **Completion**: Free functions in tree/completion.rs (optional, stateless logic)

## References

- **SPECIFICATION.md**: Complete behavioral specification (extracted from C++ implementation)
- **IMPLEMENTATION.md**: Implementation tracking and phased development plan
- **SECURITY.md**: Authentication, access control, and security design
- **CLAUDE.md**: Working patterns and practical implementation guidance
