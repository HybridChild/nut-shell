# cli-service Documentation

Complete documentation for the cli-service embedded CLI library.

## Quick Navigation

| Document | Purpose | When to Read |
|----------|---------|--------------|
| **[SPECIFICATION.md](SPECIFICATION.md)** | Exact behavioral requirements | Implementing features, understanding what the system does |
| **[DESIGN.md](DESIGN.md)** | Design decisions and rationale | Understanding why design choices were made, feature gating |
| **[INTERNALS.md](INTERNALS.md)** | Runtime behavior and data flow | Understanding how the system works internally |
| **[IMPLEMENTATION.md](IMPLEMENTATION.md)** | Implementation roadmap | Finding what to build next, build commands |
| **[SECURITY.md](SECURITY.md)** | Authentication and security | Implementing auth, credential storage, access control |
| **[PHILOSOPHY.md](PHILOSOPHY.md)** | Design philosophy | Evaluating feature requests, understanding project scope |

## Documentation Overview

### [SPECIFICATION.md](SPECIFICATION.md) - WHAT the System Does
**35KB • Behavioral Specification**

Authoritative reference for exact system behavior:
- Terminal I/O (character echo, control sequences, escape sequences)
- Authentication flow (login, password masking, session management)
- Tab completion behavior (single/multiple matches, directory handling)
- Command history (navigation, storage rules)
- Response formatting (indentation, newlines, prompt control)
- Global commands (`help`, `?`, `logout`, `clear`)
- Path resolution (absolute/relative, `.` and `..`)
- Access control enforcement
- Example command trees

**Read this when:** You need to know exactly how a feature should behave.

---

### [DESIGN.md](DESIGN.md) - WHY It's Designed This Way
**37KB • Design Decisions & Rationale**

Design decisions and architectural patterns:
- Command syntax rationale (path-based, no trailing slash, reserved keywords)
- Key design decisions (6 major decisions with alternatives considered)
- Feature gating patterns (authentication, completion, history)
- Unified architecture approach (single code path for auth-enabled/disabled)
- Stub function pattern (minimizing `#[cfg]` branching)
- Module structure (14 modules, organization rationale)
- Implementation benefits (zero-cost abstractions, ROM placement, O(1) operations)

**Read this when:** You want to understand the reasoning behind design choices or need feature gating examples.

---

### [INTERNALS.md](INTERNALS.md) - HOW the System Works
**35KB • Runtime Internals**

Complete data flow from character input to terminal output:
- High-level system overview (7-layer architecture)
- Level 1-7 detailed pseudocode implementations:
  - Character input processing
  - InputParser state machine
  - Command input processing
  - Path parsing & tree navigation
  - Request processing
  - Interactive features (tab completion, history)
  - Response formatting & output
- Complete flow diagrams (authentication enabled/disabled)
- State transition diagrams
- Access control enforcement points
- Memory layout (Flash/RAM)
- Performance characteristics (time complexity table)
- Thread safety considerations
- Error handling strategy

**Read this when:** You need to understand the runtime behavior or implement a complex feature.

---

### [IMPLEMENTATION.md](IMPLEMENTATION.md) - Implementation Roadmap
**14KB • Task Tracking & Build Commands**

Phased implementation plan and build workflows:
- 10 implementation phases (Foundation → Polish)
- Task breakdown per phase
- Success criteria for each phase
- Test-driven development workflow
- Complete build command reference:
  - Quick iteration commands
  - Feature validation
  - Embedded target verification
  - Pre-commit validation
  - CI simulation
  - Troubleshooting
- Current status tracking

**Read this when:** You need to know what to build next or how to build/test the project.

---

### [SECURITY.md](SECURITY.md) - Security Design
**25KB • Authentication & Access Control**

Security architecture and best practices:
- Security vulnerabilities analysis (plaintext passwords, hardcoded credentials, etc.)
- Rust implementation security design (SHA-256 hashing, salting, constant-time comparison)
- Password hashing rationale (why SHA-256 vs bcrypt/Argon2)
- Credential storage options:
  - Build-time environment variables (default)
  - Flash storage (production recommended)
  - Const provider (testing only)
  - Custom trait-based providers
- Access control system (generic AccessLevel trait, path-based validation)
- Authentication feature gating (unified architecture approach)
- Implementation patterns (login flow, password masking, credential hashing)
- Testing & validation (unit tests, integration tests, security tests)
- Threat model and security assumptions

**Read this when:** Implementing authentication, choosing credential storage, or evaluating security requirements.

---

### [PHILOSOPHY.md](PHILOSOPHY.md) - Design Philosophy
**17KB • Feature Decision Framework**

Project philosophy and feature criteria:
- Core principle: Essential CLI primitives only
- What we include:
  - Core functionality (always present)
  - Interactive UX features (default enabled, can disable)
  - Global commands
- What we exclude (with rationale):
  - Shell scripting features
  - Command aliases
  - Output paging
  - Command logging
  - History persistence
  - Advanced line editing
  - Session management
  - Visual feedback
  - Multi-line input
- Decision framework (5 key questions):
  - Cost analysis
  - Embedded relevance
  - Alternative solutions
  - User demand
  - Consistency with philosophy
- Design principles (8 core principles)
- Feature status reference (implemented/future/excluded)
- Evolution guidelines (when to add/reject/defer)
- Success metrics

**Read this when:** Evaluating a feature request or understanding project scope.

---

## Common Questions

### "What does feature X do?"
→ See **[SPECIFICATION.md](SPECIFICATION.md)**

### "Why was it implemented this way?"
→ See **[DESIGN.md](DESIGN.md)**

### "How does the login flow work internally?"
→ See **[INTERNALS.md](INTERNALS.md)** (Level 1 & 3)

### "Should we add feature Y?"
→ See **[PHILOSOPHY.md](PHILOSOPHY.md)** (Decision Framework)

### "How do I implement authentication?"
→ See **[SECURITY.md](SECURITY.md)** (Implementation Patterns) and **[IMPLEMENTATION.md](IMPLEMENTATION.md)** (Phase 2)

### "What's the build command for X?"
→ See **[IMPLEMENTATION.md](IMPLEMENTATION.md)** (Build & Validation Commands)

### "How do I feature-gate a module?"
→ See **[DESIGN.md](DESIGN.md)** (Feature Gating & Optional Features)

---

## Document Relationships

```
┌─────────────────┐
│  PHILOSOPHY.md  │  ← Why we build what we build
└────────┬────────┘
         │ guides
         ▼
┌─────────────────┐
│    DESIGN.md    │  ← Why it's designed this way
└────────┬────────┘
         │ defines
         ▼
┌─────────────────┐
│SPECIFICATION.md │  ← What it should do
└────────┬────────┘
         │ specifies
         ▼
┌─────────────────┐
│  INTERNALS.md   │  ← How it works at runtime
└────────┬────────┘
         │ implements
         ▼
┌─────────────────┐
│IMPLEMENTATION.md│  ← How to build it
└────────┬────────┘
         │ builds
         ▼
┌─────────────────┐
│   SECURITY.md   │  ← How authentication/access control works
└─────────────────┘
```

---

## Document Size Reference

| Document | Size | Lines | Primary Focus |
|----------|------|-------|---------------|
| DESIGN.md | 37KB | ~1065 | Design rationale, feature gating |
| INTERNALS.md | 35KB | ~979 | Runtime behavior, data flow |
| SECURITY.md | 25KB | ~891 | Authentication, security |
| SPECIFICATION.md | 22KB | ~807 | Behavioral requirements |
| PHILOSOPHY.md | 17KB | ~550 | Design philosophy |
| IMPLEMENTATION.md | 14KB | ~453 | Implementation roadmap |

---

## For AI Assistants

See **[../CLAUDE.md](../CLAUDE.md)** for:
- Quick reference for common tasks
- Core architecture patterns
- Critical constraints (no_std, static allocation)
- Common pitfalls & solutions
- Testing patterns
- Build command quick reference

---

**Last Updated:** 2025-11-16
