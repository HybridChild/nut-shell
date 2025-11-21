# nut-shell Design Philosophy

## Core Principle

> **nut-shell provides the essential interactive CLI primitives for embedded systems - nothing more, nothing less.**

Every feature must justify its existence through the lens of embedded constraints: flash size, RAM usage, and runtime overhead. We favor simplicity and predictability over convenience and flexibility.

---

## What We Include

### **Core Functionality (Always Present)**
These features define what a CLI is and are non-negotiable:

- **Path-based navigation** - Hierarchical command structure with Unix-style paths
- **Command execution** - Execute commands with positional arguments (sync and async)
- **Access control** - User-defined permission hierarchies
- **Input parsing** - Terminal I/O with basic line editing
- **Error handling** - Type-safe response system
- **Const initialization** - Zero runtime overhead, ROM placement
- **Metadata/execution separation pattern** - Commands split into const metadata and runtime handlers

**Why:** Without these, it's not a functional CLI. These are the minimal primitives.

**Note:** The metadata/execution separation pattern enables natural async command support (via `process_char_async()` and `CommandHandlers` trait) without compromising const-initialization or adding heap dependencies.

---

### **Interactive UX Features (Default Enabled, Can Disable)**
Features that significantly improve usability for interactive human operators:

#### **Authentication & Security** (`authentication` feature)
- User login with password masking
- SHA-256 password hashing with salts
- Pluggable credential providers
- Access control enforcement

**Rationale:**
- Essential for production embedded systems
- Prevents unauthorized access to dangerous commands
- Minimal overhead when implemented correctly (~2KB flash)
- Can disable for development/unsecured environments

**Cost:** ~2KB flash, 0 bytes RAM, requires sha2 + subtle crates

---

#### **Tab Completion** (`completion` feature)
- Command and path prefix matching
- Directory vs. command differentiation
- Access-level filtering
- Multi-match display

**Rationale:**
- Dramatically reduces typing on slow serial connections
- Aids discoverability without introspection commands
- Stateless implementation (zero RAM cost)
- Engineers appreciate not memorizing exact names

**Cost:** ~2KB flash, 0 bytes persistent RAM (temporary stack only)

---

#### **Command History** (`history` feature)
- Arrow key navigation (up/down)
- Circular buffer with configurable size
- Original buffer restoration
- Clear on logout

**Rationale:**
- Common expectation for interactive CLIs
- Reduces repetitive typing during debugging
- Configurable capacity (N=4 for constrained, N=10 default)
- RAM cost is explicit and user-controlled

**Cost:** ~500-800 bytes flash, ~1.3KB RAM (N=10), ~0.5KB RAM (N=4)

---

#### **Line Editing** (Always Enabled)
- **Current support:**
  - Backspace/delete (remove characters)
  - Up/down arrows (history navigation)
  - Double-ESC (clear buffer and exit history) - Quick cancel/reset without repeated backspace
  - Tab (completion)
  - Enter (submit)

- **Recommended additions:**
  - Left/right arrows (cursor positioning within line)
  - Home/End keys (jump to start/end of line)

**Rationale:**
- Standard terminal behavior users expect
- Minimal implementation cost (~150-200 bytes total)
- High value for interactive editing
- Escape sequence parser already exists
- Double-ESC provides quick escape from any input state (~50-100 bytes, no RAM)

**Cost:** ~150-200 bytes flash for cursor movement, ~50-100 bytes for double-ESC, 0 bytes RAM

**Why not feature-gated?** Too small to justify the complexity, too essential for usability.

---

### **Global Commands** (Always Enabled)
Reserved keywords that work from any location:

- `?` - List available global commands
- `ls` - Show current directory contents with descriptions
- `logout` - End session (only when authentication enabled)
- `clear` - Clear screen (platform-dependent, may be no-op)

**Rationale:**
- Essential for discoverability and navigation
- Minimal code overhead (~200-300 bytes total)

**Why no `cd`, `ls`, `pwd`?** Path-based syntax makes them redundant:
- `cd system` → just type `system`
- `ls` → use `?` command
- `pwd` → shown in prompt (`user@/current/path>`)

---

## What We Exclude

### **Explicitly Out of Scope**

#### **Shell Scripting Features**
- ❌ Command piping (`cmd1 | cmd2`)
- ❌ Variable expansion (`$VAR`, `${FOO}`)
- ❌ Environment variables (`set VAR=value`)
- ❌ Command substitution (`` `cmd` ``)
- ❌ Conditionals/loops (`if`, `while`, `for`)
- ❌ Batch script execution from storage

**Rationale:**
- Requires dynamic allocation or massive static buffers
- Breaks no_std philosophy
- Serial connections can send commands line-by-line from host
- Host-side automation tools (Python, shell scripts) are better suited
- Massive complexity increase (thousands of lines)
- Not typical for embedded CLIs

**Alternative:** Users can implement scripting at the application layer if needed, or use host-side tools to send commands over serial.

---

#### **Command Aliases**
- ❌ `r` → `reboot`
- ❌ `net` → `network`
- ❌ User-defined shortcuts

**Rationale:**
- Tab completion already solves discoverability
- Alias lookup table costs flash (ROM storage)
- Engineers can memorize common commands
- Adds parser complexity
- Cost doesn't justify benefit (~500 bytes + alias table)

**Alternative:** Tab completion makes typing fast enough (`r<TAB>` → `reboot`).

---

#### **Output Paging/Scrolling**
- ❌ `more`/`less`-style pagination
- ❌ Screen-aware output formatting

**Rationale:**
- Most embedded commands have short outputs
- Terminal emulators already provide scrollback
- Adds significant complexity (~1-2KB)
- Requires terminal size queries (platform-specific)
- Not typical for embedded CLIs

**Alternative:** Terminal emulators handle scrolling. Commands should be designed for concise output.

---

#### **Command Logging / Audit Trail**
- ❌ Persistent command history across reboots
- ❌ Audit logging to flash/external storage
- ❌ Session replay functionality

**Rationale:**
- Platform-specific (depends on storage infrastructure)
- Flash wear concerns (write leveling required)
- Better handled at application layer
- Out of scope for CLI library

**Alternative:** Applications can implement logging in their command execution handlers if needed.

---

#### **History Persistence**
- ❌ Save history to flash across reboots

**Rationale:**
- Flash write wear concerns
- In-memory history sufficient for interactive sessions
- Adds storage layer dependency
- Cost doesn't justify benefit

**Alternative:** Could be added later as optional `flash-history` feature if users explicitly request it.

---

#### **Advanced Line Editing**
- ⚠️ Ctrl+K (kill to end of line) - **Future consideration**
- ⚠️ Ctrl+U (kill entire line) - **Future consideration**
- ⚠️ Ctrl+A / Ctrl+E (emacs-style home/end) - **Future consideration**
- ⚠️ Ctrl+W (delete word backward) - **Future consideration**
- ❌ Vi-mode editing
- ❌ Undo/redo

**Rationale:**
- Each shortcut adds ~50-100 bytes
- Less discoverable than arrow keys
- Power-user features with diminishing returns
- Can add as future enhancement if requested

**Current stance:** Wait for user demand before implementing.

---

#### **Session Management**
- ⚠️ Auto-logout after inactivity timeout - **Future optional feature**
- ❌ Multiple concurrent sessions
- ❌ Session persistence across reboots

**Rationale:**
- Auto-logout requires timer/RTC integration (platform-specific)
- Multiple sessions assume multi-threading (not no_std friendly)
- Session persistence adds storage dependency

**Current stance:** Document auto-logout as future `session-timeout` feature for security-critical deployments.

---

#### **Visual Feedback / Status Indicators**
- ⚠️ Custom prompt callbacks - **Document as extension point**
- ⚠️ Mode indicators in prompt (`[RO]`, `[DEBUG]`) - **User-implementable**
- ❌ Color/ANSI styling support

**Rationale:**
- Highly application-specific
- Can be implemented via custom `generate_prompt()` override
- Color support adds terminal capability detection complexity
- Better as extension point than built-in feature

**Current stance:** Document how users can customize prompts in their implementation.

---

#### **Multi-line Input / Line Continuation**
- ⚠️ `\` for line continuation - **Wait for demand**

**Rationale:**
- **Pro:** Useful for commands with many arguments
- **Pro:** Minimal implementation (~50-100 bytes)
- **Con:** Rare use case (most embedded commands are short)
- **Con:** Adds parser state complexity

**Current stance:** Only add if real use cases emerge with 10+ argument commands.

---

## Decision Framework

When evaluating a new feature, ask:

### **1. Cost Analysis**
- **Flash cost:** How many bytes of code?
- **RAM cost:** Persistent or temporary allocation?
- **Dependency cost:** New crates required?
- **Complexity cost:** How many lines of code? New modules?

**Threshold:** Features >500 bytes should be optional (feature-gated) unless essential.

---

### **2. Embedded Relevance**
- Is this typical for embedded system CLIs?
- Does it solve a problem unique to embedded contexts?
- Or is it desktop shell behavior being imported unnecessarily?

**Example:** Command history (YES - useful for debugging). Piping (NO - desktop shell feature).

---

### **3. Alternative Solutions**
- Can terminal emulators handle this? (scrolling, colors)
- Can host-side tools handle this? (scripting, batch commands)
- Can users implement this at application layer? (custom commands, logging)

**Principle:** Don't reinvent what already exists in better forms elsewhere.

---

### **4. User Demand**
- Is anyone actually asking for this?
- How many users would benefit?
- What's the workaround if we don't add it?

**Principle:** Wait for demonstrated demand before adding "nice to have" features.

---

### **5. Consistency with Philosophy**
- Does it align with "essential CLI primitives only"?
- Does it maintain no_std compatibility?
- Does it preserve static allocation?
- Does it avoid runtime overhead?

**Red flags:** Dynamic allocation, runtime initialization, platform-specific dependencies.

---

## Design Principles

### **1. Simplicity Over Features**
Every feature is a liability:
- More code to maintain
- More surface area for bugs
- More flash/RAM consumption
- More complexity for users

**Default answer is NO.** Features must earn their inclusion.

---

### **2. Const Over Runtime**
Prefer compile-time decisions:
- Const-initialized trees (ROM placement)
- Feature flags (compile-time elimination)
- Generic type parameters (monomorphization, no vtables)
- Static allocation (predictable memory usage)

**Avoid:** Heap allocation, lazy initialization, runtime configuration.

---

### **3. Embedded-First Mindset**
This is not a desktop shell:
- Serial connections are slow (9600-115200 baud typical)
- Flash is precious (32KB-256KB typical)
- RAM is scarce (8KB-64KB typical)
- No filesystem assumed
- Single-threaded execution
- Deterministic behavior required

**Design for RP2040 (32KB flash, 264KB RAM), not Linux.**

---

### **4. Graceful Degradation**
Features should be independently disable-able:
- `--no-default-features` = minimal working CLI
- Each feature adds specific, measurable value
- No cascading dependencies between optional features
- Core functionality works without any optional features

**Example:** Authentication, completion, and history are all independent.

---

### **5. Security by Design**
When security features are enabled:
- Password masking (prevent shoulder surfing)
- Hashed storage (never plaintext passwords)
- Access control enforcement (inaccessible nodes invisible)
- No information leakage (same error for nonexistent vs. forbidden)

When disabled (development mode):
- Zero overhead
- No security checks
- Full access to tree

**No half-measures.** Either secure or explicitly unsecured.

---

### **6. Zero-Cost Abstractions**
Generic programming should compile to optimal code:
- Trait-based I/O (monomorphized, no vtables)
- Generic access levels (compile-time enforcement)
- Feature flags (dead code elimination)
- Inline-friendly designs

**Verify:** Check assembly output, measure binary sizes.

---

### **7. Path-Based Philosophy**
Unix-style paths replace traditional commands:
- `system/reboot` instead of `cd system && reboot`
- `../network/status` instead of `cd ../network && status`
- Tab completion makes paths fast to type
- Prompt shows current location
- `?` command shows local contents

**Why better:**
- Less typing overall
- No state confusion (always know where you are)
- Scriptable (absolute paths work anywhere)
- Natural for hierarchical commands

---

### **8. Interactive Discovery**
Users learn through interaction, not documentation:
- `?` shows global commands
- `ls` shows current directory contents with descriptions
- Tab completion reveals available options
- Prompts show current context
- Error messages are specific and actionable

---

## Feature Status Reference

### **Core (Always Present)**
- ✅ Path-based navigation
- ✅ Command execution
- ✅ Global commands (`ls`, `?`)
- ✅ Basic line editing (backspace, enter)
- ✅ Double-ESC clear
- ✅ Access control framework

### **Default Enabled (Can Disable)**
- ✅ Authentication (`authentication` feature) - ~2KB flash
- ✅ Tab completion (`completion` feature) - ~2KB flash
- ✅ Command history (`history` feature) - ~0.5-0.8KB flash, configurable RAM

### **Recommended Additions**
- ⚠️ Left/right arrow keys (cursor positioning) - ~100 bytes
- ⚠️ Home/End keys - ~50 bytes

### **Future Considerations (Not Implemented)**
- ⚠️ Session timeout (`session-timeout` feature)
- ⚠️ Advanced Ctrl shortcuts (Ctrl+K, Ctrl+U, etc.)
- ⚠️ Multi-line input (line continuation with `\`)
- ⚠️ Custom prompt callbacks
- ⚠️ History persistence (`flash-history` feature)

### **Explicitly Excluded**
- ❌ Command piping
- ❌ Variable expansion
- ❌ Environment variables
- ❌ Shell scripting
- ❌ Command aliases
- ❌ Output paging
- ❌ Audit logging (library level)
- ❌ Multiple sessions
- ❌ Color/ANSI styling (built-in)
- ❌ Vi-mode editing
- ❌ Undo/redo

---

## Evolution Guidelines

### **When to Add Features**
1. Multiple users request the same capability
2. Feature has clear embedded use case (not desktop shell import)
3. Cost is justified by value (measured in flash/RAM bytes)
4. No reasonable alternative exists
5. Can be implemented as optional feature (feature flag)

### **When to Reject Features**
1. Can be handled by terminal emulator
2. Can be handled by host-side tools
3. Can be implemented at application layer
4. Breaks no_std compatibility
5. Requires dynamic allocation
6. Cost exceeds 500 bytes without proportional value
7. Is desktop shell feature without embedded justification

### **When to Defer Features**
1. No demonstrated user demand yet
2. Unclear if implementation approach is optimal
3. Platform-specific (want to see demand across multiple platforms)

---

## Success Metrics

A successful CLI library for embedded systems should:

1. ✅ **Compile on thumbv6m-none-eabi** (RP2040 target)
2. ✅ **Fit in 32KB flash** (with all default features)
3. ✅ **Use <8KB RAM** (with default configuration)
4. ✅ **Zero heap allocation** (pure stack + static)
5. ✅ **Enable feature toggling** (each feature independently disable-able)
6. ✅ **Provide interactive UX** (when features enabled)
7. ✅ **Degrade gracefully** (minimal build still useful)
8. ✅ **Maintain security** (when authentication enabled)
9. ✅ **Remain maintainable** (<5000 lines of code total)
10. ✅ **Serve real use cases** (actual embedded deployments)

**Non-goals:**
- ❌ Feature parity with bash/zsh
- ❌ Maximum flexibility/configurability
- ❌ Support every possible use case
- ❌ Be all things to all users

---

## Conclusion

**nut-shell is intentionally constrained.**

We provide essential interactive CLI primitives for embedded systems - path navigation, command execution, access control, and optional UX features (completion, history, authentication).

We deliberately exclude shell scripting, dynamic features, and desktop conveniences that don't translate to embedded constraints.

**When in doubt, say NO.** Features are forever. Simplicity is a feature.

---

## Questions to Ask

Before proposing a feature addition, ask:

1. **Why can't the user implement this in their command handler?**
2. **Why can't the terminal emulator handle this?**
3. **Why can't a host-side script handle this?**
4. **What are 5 embedded systems that need this?**
5. **What's the workaround if we don't add it?**
6. **How many bytes will this cost?**
7. **Can it be feature-gated?**
8. **Does it work in no_std?**
9. **Does it require dynamic allocation?**
10. **Is this a desktop shell behavior being cargo-culted?**

If you can't answer these convincingly, the feature doesn't belong in nut-shell.

---

## Related Documentation

This philosophy is implemented across:
- **[DESIGN.md](DESIGN.md)** - Design decisions and patterns implementing these principles
- **[SECURITY.md](SECURITY.md)** - Security-by-design principles in authentication system
- **[EXAMPLES.md](EXAMPLES.md)** - Practical usage examples following these constraints
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Build workflows and testing strategies
- **[../CLAUDE.md](../CLAUDE.md)** - Patterns for working within these constraints

---

**Maintained by:** nut-shell project
**Purpose:** Guide feature decisions and maintain project focus
**When to update:** When feature requests arise, when philosophy evolves
