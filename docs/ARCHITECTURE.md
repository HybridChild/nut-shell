# CLIService Rust Port - Architecture

This document records the architectural decisions made when porting CLIService from C++ to Rust. It explains the rationale behind structural choices and documents alternatives considered.

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

**Result**: ~25 C++ files → ~9 Rust files, more maintainable codebase

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

### 4. Completion Implementation
**Decision**: Free functions or trait methods in `completion` module

**Rationale**: No state needed, avoid class wrapper

**Alternative Considered**: Separate Completer type (C++ approach)

### 5. State Management
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
- **Simplified architecture**: Enums replace class hierarchies, reducing ~25 C++ files to ~9 Rust files

## Module Structure

**Simplified structure (9 files vs C++'s 25):**

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
│   └── completion.rs   # Tab completion logic
├── response.rs         # Response type + formatting
├── io.rs               # CharIo trait
└── user.rs             # User + AccessLevel trait
```

**Rationale for consolidation:**
- **Request types**: Single enum replaces class hierarchy (5 classes → 1 enum)
- **State management**: Inline in cli/mod.rs (too small for separate file)
- **Path resolution**: Methods on existing types (no separate PathResolver class)
- **Tree types**: Combined in tree/mod.rs (related const-init concerns)
- **Access control**: Unified with User in user.rs (closely related)

## References

- C++ implementation: `CLIService/` subdirectory
- Implementation tracking: `IMPLEMENTATION.md`
- Working guidance: `CLAUDE.md`
