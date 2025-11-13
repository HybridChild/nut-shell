# SECURITY.md

## Authentication & Access Control Security Design

This document describes the security architecture for authentication and access control in the Rust CLIService implementation, including rationale, implementation patterns, and best practices for embedded systems.

---

## Table of Contents

1. [Current C++ Implementation Analysis](#current-c-implementation-analysis)
2. [Security Vulnerabilities & Concerns](#security-vulnerabilities--concerns)
3. [Rust Implementation Security Design](#rust-implementation-security-design)
4. [Password Hashing](#password-hashing)
5. [Credential Storage Options](#credential-storage-options)
6. [Access Control System](#access-control-system)
7. [Feature Gating](#feature-gating)
8. [Implementation Patterns](#implementation-patterns)
9. [Migration Guide](#migration-guide)
10. [Testing & Validation](#testing--validation)
11. [Security Assumptions](#security-assumptions)

---

## Current C++ Implementation Analysis

### Authentication Flow

The C++ implementation uses a simple username/password authentication system:

```cpp
// From CLIService/cliService/example/main.cpp
std::vector<User> users {
    {"admin", "admin123", AccessLevel::Admin},
    {"user", "user123", AccessLevel::User}
};
```

**Login Process:**
1. User enters `username:password` (e.g., `admin:secretpass`)
2. Password characters are masked with `*` after colon is detected
3. System searches user vector for exact string match
4. On success: user logged in, state changes to `LoggedIn`
5. On failure: error message displayed, remains in `LoggedOut` state

**Access Control:**
- Each node (command/directory) has an `AccessLevel`
- User's level must be >= node's level
- Validation checks entire path from target node to root
- Access denied if any node in path requires higher privileges

---

## Security Vulnerabilities & Concerns

### Critical Issues

#### 1. **Plaintext Password Storage**
- Passwords stored as unencrypted strings in memory and binary
- Visible in source code and version control
- Extractable from compiled binary using `strings` command
- No protection against memory dumps or binary inspection

#### 2. **Hardcoded Credentials in Source**
- Credentials committed to version control repository
- Shared across all deployments (no per-device secrets)
- Requires recompilation to change passwords
- Example credentials (`admin123`) may be used in production

#### 3. **No Password Hashing**
- Direct string comparison: `user.getPassword() == password`
- No salt, no key derivation
- Vulnerable to rainbow table attacks if passwords leaked
- Cannot enforce password complexity requirements

#### 4. **Binary String Exposure**
```bash
$ strings cli_service | grep admin
admin
admin123
```

#### 5. **Unlimited Login Attempts**
- No rate limiting or account lockout
- Brute force attacks possible via serial console
- No logging of failed attempts

### Context-Specific Risks

**For Embedded Systems (RP2040/Pico):**
- ⚠️ **Medium Risk**: Physical access usually implies complete control
- ⚠️ **Medium Risk**: UART/USB serial access often indicates physical presence
- ✅ **Mitigated by**: Device typically in physically secured enclosure
- ✅ **Mitigated by**: Limited attack surface (no network stack)

**For Networked/Multi-Device Deployments:**
- ❌ **High Risk**: Same credentials on all devices
- ❌ **High Risk**: Credentials in repository accessible to all developers
- ❌ **Critical Risk**: No credential rotation capability

---

## Rust Implementation Security Design

### Core Principles

1. **No plaintext passwords** - SHA-256 hashed credentials only
2. **No credentials in source** - Build-time or runtime configuration
3. **Extensible architecture** - Trait-based credential providers
4. **Per-device secrets** - Flash storage enables unique credentials
5. **Optional authentication** - Feature-gated for flexibility
6. **Constant-time comparison** - Prevents timing attacks
7. **Salted hashes** - User-specific salts prevent rainbow tables

### Architecture Overview

```rust
// Trait-based credential provider system
pub trait CredentialProvider<L: AccessLevel> {
    type Error;

    fn find_user(&self, username: &str) -> Result<Option<User<L>>, Self::Error>;
    fn verify_password(&self, user: &User<L>, password: &str) -> bool;
    fn list_users(&self) -> Result<Vec<&str>, Self::Error>;
}

// User with generic access level and hashed credentials
pub struct User<L: AccessLevel> {
    pub username: heapless::String<32>,
    pub password_hash: [u8; 32],  // SHA-256 hash
    pub salt: [u8; 16],           // User-specific salt
    pub access_level: L,
}

// Password hasher abstraction
pub trait PasswordHasher {
    fn hash(&self, password: &str, salt: &[u8]) -> [u8; 32];
    fn verify(&self, password: &str, salt: &[u8], hash: &[u8; 32]) -> bool;
}
```

---

## Password Hashing

### SHA-256 Choice Rationale

**Why SHA-256 instead of bcrypt/Argon2?**

| Factor | SHA-256 | bcrypt/Argon2 |
|--------|---------|---------------|
| **Memory Usage** | ~1KB | ~16KB - 1MB |
| **Computation** | Fast (~1μs) | Slow by design (10-100ms) |
| **Embedded Suitability** | ✅ Excellent | ⚠️ Challenging |
| **Security with Salt** | ✅ Strong | ✅ Stronger |
| **Physical Access Assumption** | ✅ Sufficient | ⚠️ Overkill |
| **RP2040 RAM** | ✅ 264KB available | ⚠️ Memory constrained |

**Decision:** SHA-256 with per-user salts provides:
- Sufficient security for physically-secured embedded devices
- Low memory footprint (critical for RP2040's 264KB RAM)
- Fast verification (~microseconds vs milliseconds)
- Protection against rainbow table attacks via salts
- Industry-standard cryptographic primitive

**Security Properties:**
- 256-bit output (2^256 possible hashes)
- Collision resistance
- Preimage resistance
- No known practical attacks
- NIST approved (FIPS 180-4)

### Implementation

```rust
use sha2::{Sha256, Digest};

pub struct Sha256Hasher;

impl PasswordHasher for Sha256Hasher {
    fn hash(&self, password: &str, salt: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(salt);
        hasher.update(password.as_bytes());
        hasher.finalize().into()
    }

    fn verify(&self, password: &str, salt: &[u8], hash: &[u8; 32]) -> bool {
        let computed = self.hash(password, salt);

        // Constant-time comparison to prevent timing attacks
        subtle::ConstantTimeEq::ct_eq(&computed, hash).into()
    }
}
```

### Salt Generation

**Per-User Salts:**
- 128-bit (16 bytes) random salt per user
- Prevents rainbow table attacks across users
- Stored alongside password hash
- Generated once during user creation

**Salt Sources:**
```rust
// Build time: From secure random generator
const ADMIN_SALT: [u8; 16] = *b"unique_salt_0001";

// Runtime: From hardware RNG (RP2040 ROSC)
use rp2040_hal::rosc::RingOscillator;
let salt = rosc.get_random_bytes::<16>();

// Flash storage: Stored with hashed password
struct StoredCredential {
    username: [u8; 32],
    password_hash: [u8; 32],
    salt: [u8; 16],
    access_level: u8,
}
```

---

## Credential Storage Options

### 1. Build-Time Environment Variables (Default)

**Use Case:** Production deployments with build-time configuration

**Security Level:** ⭐⭐⭐⭐ (High)

**Implementation:**
```rust
// build.rs
fn main() {
    // Read from secure build environment
    let users = std::env::var("CLI_USERS")
        .expect("CLI_USERS not set");

    // Format: "username:hash:salt:level;username:hash:salt:level"
    // Example: "admin:deadbeef...:cafebabe...:Admin"

    println!("cargo:rustc-env=CLI_USERS={}", users);
}

// src/auth/providers/buildtime.rs
pub struct BuildTimeProvider<L> {
    users: heapless::Vec<User<L>, MAX_USERS>,
}

impl<L: AccessLevel> BuildTimeProvider<L> {
    pub const fn new() -> Self {
        // Parse CLI_USERS at compile time
        // Store as const data in ROM
    }
}
```

**Advantages:**
- ✅ No credentials in source code
- ✅ Different credentials per build/deployment
- ✅ Credentials in ROM (not modifiable at runtime)
- ✅ Can be set by CI/CD securely
- ✅ Zero runtime overhead

**Disadvantages:**
- ⚠️ Requires rebuild to change credentials
- ⚠️ Same credentials for all devices in build batch
- ⚠️ Hash visible in binary (requires `strings` to extract)

**Best For:**
- Single-device or small-batch deployments
- Environments where credentials rarely change
- When rebuild process is acceptable for rotation

### 2. Flash Storage (Production Recommended)

**Use Case:** Production embedded systems requiring per-device credentials

**Security Level:** ⭐⭐⭐⭐⭐ (Highest)

**Implementation:**
```rust
use rp2040_flash::{flash, FLASH_SIZE};

// Dedicate last 4KB sector for credentials
const CREDENTIAL_SECTOR: u32 = (FLASH_SIZE - 4096) as u32;

pub struct FlashProvider<L> {
    _phantom: PhantomData<L>,
}

impl<L: AccessLevel> FlashProvider<L> {
    pub fn load_users(&self) -> Result<Vec<User<L>>, FlashError> {
        let data = flash::read_sector(CREDENTIAL_SECTOR)?;
        self.parse_credentials(data)
    }

    pub fn update_user(&mut self, user: &User<L>) -> Result<(), FlashError> {
        // Admin-only command to update credentials
        flash::erase_sector(CREDENTIAL_SECTOR)?;
        flash::write_sector(CREDENTIAL_SECTOR, &self.serialize(user))?;
        Ok(())
    }
}
```

**Advantages:**
- ✅ Per-device unique credentials
- ✅ Updateable without recompilation
- ✅ Survives firmware updates (separate flash sector)
- ✅ Can implement credential rotation
- ✅ No credentials in source or binary

**Disadvantages:**
- ⚠️ Requires flash write capability (admin command)
- ⚠️ Wear leveling considerations (flash has limited writes)
- ⚠️ Initial provisioning process needed

**Best For:**
- Production deployments with many devices
- Systems requiring credential rotation
- High-security environments
- Devices with unique per-device identities

**Provisioning Process:**
```rust
// During manufacturing/first boot
impl<L: AccessLevel> FlashProvider<L> {
    pub fn provision(&mut self, admin_password: &str) -> Result<(), FlashError> {
        // Generate unique salt from hardware RNG
        let salt = self.get_hardware_random_salt();

        // Hash provided password
        let hash = Sha256Hasher.hash(admin_password, &salt);

        // Store in flash
        let admin = User {
            username: heapless::String::from("admin"),
            password_hash: hash,
            salt,
            access_level: L::admin(),
        };

        self.update_user(&admin)
    }
}
```

### 3. Const Provider (Examples/Testing Only)

**Use Case:** Examples, prototypes, testing

**Security Level:** ⭐ (Low - NOT for production)

**Implementation:**
```rust
// examples/basic_auth.rs
const EXAMPLE_USERS: &[User<ExampleAccessLevel>] = &[
    User {
        username: heapless::String::from_str("admin").unwrap(),
        password_hash: [0xde, 0xad, 0xbe, 0xef, /* ... */],
        salt: [0xca, 0xfe, 0xba, 0xbe, /* ... */],
        access_level: ExampleAccessLevel::Admin,
    },
];

pub struct ConstProvider {
    users: &'static [User<ExampleAccessLevel>],
}
```

**Advantages:**
- ✅ Simple implementation
- ✅ No build-time dependencies
- ✅ Good for examples/documentation

**Disadvantages:**
- ❌ Hardcoded in binary
- ❌ Same credentials everywhere
- ❌ NOT suitable for production

**Best For:**
- Example code
- Unit/integration tests
- Prototyping
- Documentation

### 4. Custom Trait-Based Provider

**Use Case:** Specialized backends (LDAP, external auth, HSM)

**Security Level:** Depends on implementation

**Implementation:**
```rust
// User implements custom provider
pub struct LdapProvider {
    server: &'static str,
    // LDAP configuration
}

impl<L: AccessLevel> CredentialProvider<L> for LdapProvider {
    type Error = LdapError;

    fn find_user(&self, username: &str) -> Result<Option<User<L>>, Self::Error> {
        // Query LDAP server
        // Return user with access level mapping
    }

    fn verify_password(&self, user: &User<L>, password: &str) -> bool {
        // Delegate to LDAP bind
    }
}
```

**Advantages:**
- ✅ Maximum flexibility
- ✅ Can integrate with existing infrastructure
- ✅ Supports complex scenarios (2FA, federation, etc.)

**Best For:**
- Integration with existing auth systems
- Complex multi-device deployments
- Specialized security requirements

---

## Access Control System

### Generic AccessLevel Trait

```rust
pub trait AccessLevel: Copy + Clone + PartialEq + PartialOrd {
    fn from_str(s: &str) -> Option<Self>;
    fn as_str(&self) -> &'static str;
}

// User-defined enum
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MyAccessLevel {
    Guest = 0,
    User = 1,
    Operator = 2,
    Admin = 3,
}

impl AccessLevel for MyAccessLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Guest" => Some(Self::Guest),
            "User" => Some(Self::User),
            "Operator" => Some(Self::Operator),
            "Admin" => Some(Self::Admin),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Guest => "Guest",
            Self::User => "User",
            Self::Operator => "Operator",
            Self::Admin => "Admin",
        }
    }
}
```

### Path-Based Access Validation

```rust
impl<'tree, L: AccessLevel, IO: CharIo> CliService<'tree, L, IO> {
    fn validate_access(&self, node: &Node<L>) -> Result<(), CliError> {
        let current_user = self.current_user
            .as_ref()
            .ok_or(CliError::NotLoggedIn)?;

        // Check node's required access level
        if current_user.access_level < node.access_level() {
            return Err(CliError::AccessDenied);
        }

        // Check entire path from node to root
        let mut current = Some(node);
        while let Some(n) = current {
            if current_user.access_level < n.access_level() {
                return Err(CliError::AccessDenied);
            }
            current = n.parent();
        }

        Ok(())
    }
}
```

### Node Access Levels

```rust
const ROOT: &[Node<MyAccessLevel>] = &[
    Node::Directory(Directory {
        name: "system",
        access_level: MyAccessLevel::User,  // Requires User level
        children: &[
            Node::Command(Command {
                name: "reboot",
                access_level: MyAccessLevel::Admin,  // Requires Admin
                execute: reboot_fn,
            }),
        ],
    }),
];
```

---

## Feature Gating & Optional Features

### Overview

The Rust implementation provides optional features that can be enabled or disabled at compile time to accommodate different deployment scenarios and resource constraints. This allows fine-grained control over code size, dependencies, and functionality.

**Available Optional Features:**
- **authentication**: User login and access control system (default: enabled)
- **completion**: Tab completion for commands and paths (default: enabled)

**Philosophy:**
- Features are enabled by default for best user experience
- Can be disabled individually or in combination for constrained environments
- No runtime overhead when disabled (eliminated at compile time)
- Graceful degradation when features are unavailable

---

### Authentication Feature

#### Cargo.toml Configuration

```toml
[features]
default = ["authentication"]

# Core authentication system
authentication = []

# Flash storage provider (requires RP2040)
flash-storage = ["authentication", "rp2040-flash"]

# Optional: Additional providers
ldap-auth = ["authentication", "ldap3"]
external-auth = ["authentication"]

[dependencies]
heapless = "0.8"

# Conditional dependencies
sha2 = { version = "0.10", default-features = false, optional = true }
rp2040-flash = { version = "0.3", optional = true }
```

#### Conditional Compilation

```rust
// src/lib.rs
#[cfg(feature = "authentication")]
pub mod auth;

#[cfg(feature = "authentication")]
pub use auth::{User, AccessLevel, CredentialProvider};

// src/cli/mod.rs
pub struct CliService<'tree, L, IO>
where
    L: AccessLevel,
    IO: CharIo,
{
    #[cfg(feature = "authentication")]
    current_user: Option<User<L>>,

    #[cfg(feature = "authentication")]
    credential_provider: &'tree dyn CredentialProvider<L>,

    #[cfg(feature = "authentication")]
    state: CliState,

    // ... other fields
}

#[cfg(feature = "authentication")]
impl<'tree, L, IO> CliService<'tree, L, IO> {
    pub fn login(&mut self, username: &str, password: &str) -> Result<(), CliError> {
        // Authentication logic
    }
}

#[cfg(not(feature = "authentication"))]
impl<'tree, L, IO> CliService<'tree, L, IO> {
    // No-op login (always succeeds)
    pub fn login(&mut self, _username: &str, _password: &str) -> Result<(), CliError> {
        Ok(())
    }

    // Access control always allows
    fn validate_access(&self, _node: &Node<L>) -> Result<(), CliError> {
        Ok(())
    }
}
```

#### Build Examples

```bash
# Default build (authentication enabled)
cargo build

# Disable authentication for debugging
cargo build --no-default-features

# Production build with flash storage
cargo build --release --features flash-storage

# Embedded target
cargo build --target thumbv6m-none-eabi --release --features flash-storage
```

---

### Auto-Completion Feature

Tab completion is an optional feature that provides interactive command and path completion. While it enhances user experience significantly, it can be disabled to reduce code size in severely constrained embedded environments or when only programmatic/scripted CLI access is expected.

#### Cargo.toml Configuration

```toml
[features]
default = ["authentication", "completion"]

# Core authentication system
authentication = []

# Tab completion for commands and paths
completion = []

[dependencies]
heapless = "0.8"

# No additional dependencies required for completion
# (uses only core Rust and heapless for bounded collections)
```

#### Code Size Impact

| Build Configuration | Flash Usage | RAM Impact | Use Case |
|---------------------|-------------|------------|----------|
| **With completion** | +~2KB | Temporary only (stack) | Interactive CLI usage |
| **Without completion** | Baseline | None | Scripted/programmatic access |

**Memory Characteristics:**
- Completion algorithm is stateless (no persistent RAM usage)
- Temporary allocations during tab processing only
- Uses `heapless::Vec` for bounded match results
- All completion code placed in ROM
- Estimated compiled size: 1.5-2.5KB depending on optimization level

#### Conditional Compilation

```rust
// src/tree/mod.rs
#[cfg(feature = "completion")]
pub mod completion;

#[cfg(feature = "completion")]
pub use completion::{CompletionResult, complete_path};

// src/cli/mod.rs
pub struct CliService<'tree, L, IO>
where
    L: AccessLevel,
    IO: CharIo,
{
    #[cfg(feature = "completion")]
    last_completion: Option<CompletionResult>,

    // ... other fields
}

// Tab key handling with dual implementation
#[cfg(feature = "completion")]
impl<'tree, L, IO> CliService<'tree, L, IO> {
    fn handle_tab(&mut self) -> Result<Response, CliError> {
        // Full completion logic
        let result = completion::complete_path(
            &self.input_buffer,
            self.current_directory(),
            self.current_user.as_ref().map(|u| &u.access_level)
        )?;

        // Store for potential re-display
        self.last_completion = Some(result.clone());

        // Return completion suggestions to user
        Ok(Response::completion(result))
    }
}

#[cfg(not(feature = "completion"))]
impl<'tree, L, IO> CliService<'tree, L, IO> {
    fn handle_tab(&mut self) -> Result<Response, CliError> {
        // Option 1: Silent ignore (recommended for embedded)
        Ok(Response::empty())

        // Option 2: Echo literal tab character
        // self.io.put_char('\t')?;
        // Ok(Response::empty())
    }
}
```

#### Implementation Details

**When completion is enabled:**
1. Tab key triggers path resolution and prefix matching
2. Current directory and access level determine visible options
3. Common prefix auto-completed if unambiguous
4. Multiple matches displayed for user selection
5. Directories shown with trailing `/` separator

**When completion is disabled:**
1. Tab key silently ignored (no action)
2. All completion code eliminated from binary
3. Zero runtime overhead
4. `CompletionResult` type and module not compiled

#### Build Examples

```bash
# Default build (completion enabled)
cargo build

# Minimal build without completion
cargo build --no-default-features --features authentication

# Embedded target with completion
cargo build --target thumbv6m-none-eabi --release

# Embedded target without completion (maximum size optimization)
cargo build --target thumbv6m-none-eabi --release \
  --no-default-features --features authentication

# Testing build without optional features
cargo build --no-default-features

# Custom feature combination
cargo build --features "authentication,completion,logging"
```

#### When to Enable/Disable

**Enable completion when:**
- ✅ Interactive CLI usage expected (human operators)
- ✅ Flash size is not critically constrained (<90% capacity)
- ✅ User experience is a priority
- ✅ Training/learning environment for new users
- ✅ Development and debugging workflows

**Disable completion when:**
- ❌ Flash size is critically constrained (>95% capacity)
- ❌ Only programmatic/scripted CLI access expected
- ❌ Minimizing attack surface is required
- ❌ Every byte counts (bootloader, recovery mode, minimal systems)
- ❌ No interactive terminal available (headless operation)

**Security Considerations:**
- Completion reveals available commands/paths to authenticated users
- Does not bypass access control (respects `AccessLevel`)
- No sensitive data exposed through completion
- Minimal attack surface (stateless algorithm)
- Safe to enable in most security contexts

---

### Combined Feature Configuration

Multiple features can be enabled or disabled in combination to suit different deployment scenarios.

#### Common Configuration Patterns

```toml
# Full-featured build (default)
[features]
default = ["authentication", "completion"]

# Minimal embedded (size-optimized)
[features]
default = []

# Interactive but unsecured (development only)
[features]
default = ["completion"]

# Secured but non-interactive (scripted access)
[features]
default = ["authentication"]
```

#### Build Examples by Scenario

```bash
# Development workstation (full features, fast iteration)
cargo build --all-features

# Production embedded device (both features)
cargo build --target thumbv6m-none-eabi --release

# Constrained device (authentication only, ~2KB saved)
cargo build --target thumbv6m-none-eabi --release \
  --no-default-features --features authentication

# Unsecured lab equipment (completion only, for ease of use)
cargo build --no-default-features --features completion

# Minimal bootloader/recovery (no optional features)
cargo build --target thumbv6m-none-eabi --release --no-default-features

# CI/CD testing (test all feature combinations)
cargo test --all-features
cargo test --no-default-features
cargo test --features authentication
cargo test --features completion
```

#### Feature Dependencies

```
authentication (independent)
  ├── No dependencies on other features
  └── Requires: sha2, subtle (optional crates)

completion (independent)
  ├── No dependencies on other features
  └── Requires: No additional crates (uses heapless only)

Note: Features are completely independent and can be
enabled in any combination without conflicts.
```

#### Code Size Comparison

| Configuration | Estimated Flash | Use Case |
|---------------|----------------|----------|
| `--no-default-features` | Baseline | Absolute minimum |
| `--features authentication` | Baseline + ~2KB | Secured, non-interactive |
| `--features completion` | Baseline + ~2KB | Interactive, unsecured |
| `--features authentication,completion` | Baseline + ~4KB | Full-featured (default) |

*Note: Actual sizes depend on target architecture, optimization level, and LLVM version. Use `cargo size` to measure your specific build.*

---

## Implementation Patterns

### Login Flow

```rust
// 1. Parse login request
let login_request = match parser.parse_line(&input) {
    ParseResult::LoginRequest { username, password } => (username, password),
    _ => return Err(CliError::InvalidFormat),
};

// 2. Find user
let user = credential_provider
    .find_user(&login_request.username)?
    .ok_or(CliError::InvalidCredentials)?;

// 3. Verify password (constant-time comparison)
if !credential_provider.verify_password(&user, &login_request.password) {
    // Rate limiting: delay after failed attempt
    delay_ms(1000);
    return Err(CliError::InvalidCredentials);
}

// 4. Update session state
self.current_user = Some(user);
self.state = CliState::LoggedIn;

Ok(Response::success("Logged in"))
```

### Password Masking During Input

```rust
impl InputParser {
    fn echo_character(&mut self, c: char) -> Result<(), IoError> {
        if self.state == CliState::LoggedOut {
            // Check if we've seen a colon (username:password)
            if let Some(colon_pos) = self.buffer.find(':') {
                // Mask password characters with '*'
                if self.buffer.len() > colon_pos + 1 {
                    return self.io.put_char('*');
                }
            }
        }

        // Echo normally
        self.io.put_char(c)
    }
}
```

### Credential Hashing Helper

```rust
// Tool for generating hashed credentials
// Usage: cargo run --bin hash-password -- "mypassword"

use sha2::{Sha256, Digest};
use rand::RngCore;

fn main() {
    let password = std::env::args().nth(1).expect("Usage: hash-password <password>");

    // Generate random salt
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);

    // Hash password
    let mut hasher = Sha256::new();
    hasher.update(&salt);
    hasher.update(password.as_bytes());
    let hash = hasher.finalize();

    // Output in format for CLI_USERS env var
    println!("Salt: {}", hex::encode(&salt));
    println!("Hash: {}", hex::encode(&hash));
    println!();
    println!("Format for CLI_USERS:");
    println!("username:{}:{}:AccessLevel", hex::encode(&hash), hex::encode(&salt));
}
```

---

## Migration Guide

### From C++ Hardcoded Credentials

#### Step 1: Generate Hashed Credentials

```bash
# Use the password hashing tool
cargo run --bin hash-password -- "admin123"
# Output: admin:a1b2c3...:d4e5f6...:Admin

cargo run --bin hash-password -- "user123"
# Output: user:7a8b9c...:1d2e3f...:User
```

#### Step 2: Set Build-Time Environment

```bash
# Set environment variable
export CLI_USERS="admin:a1b2c3...:d4e5f6...:Admin;user:7a8b9c...:1d2e3f...:User"

# Build with hashed credentials
cargo build --release
```

#### Step 3: Update C++ Code

**Before (C++):**
```cpp
std::vector<User> users {
    {"admin", "admin123", AccessLevel::Admin},
    {"user", "user123", AccessLevel::User}
};
```

**After (Rust):**
```rust
// No hardcoded credentials in source!
// Loaded from environment at build time
let provider = BuildTimeProvider::new();
let service = CliService::new(root, provider, io);
```

#### Step 4: Remove Plaintext Passwords

```bash
# Clean version control
git rm passwords.txt
git commit -m "Remove hardcoded credentials"

# Audit for exposed credentials
git log --all --full-history -- "*password*"
```

### For Production: Migrate to Flash Storage

```rust
// 1. Compile with flash-storage feature
// cargo build --release --features flash-storage --target thumbv6m-none-eabi

// 2. Provision device on first boot
let mut provider = FlashProvider::new();

if provider.is_empty()? {
    // First boot: provision with temporary password
    provider.provision("temporary_admin_password")?;

    println!("Device provisioned. Please login and change password.");
}

// 3. User changes password via admin command
// CLI> passwd
// Current password: temporary_admin_password
// New password: <strong password>
// Confirm: <strong password>
// Password updated successfully
```

---

## Testing & Validation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let hasher = Sha256Hasher;
        let salt = [0u8; 16];
        let password = "test_password";

        let hash = hasher.hash(password, &salt);
        assert!(hasher.verify(password, &salt, &hash));
        assert!(!hasher.verify("wrong", &salt, &hash));
    }

    #[test]
    fn test_constant_time_comparison() {
        // Verify timing attack resistance
        let hasher = Sha256Hasher;
        let salt = [0u8; 16];

        let correct = "correct_password";
        let wrong1 = "c";  // Differs at first char
        let wrong2 = "correct_passwor";  // Differs at last char

        let hash = hasher.hash(correct, &salt);

        // All comparisons should take similar time
        let start = now();
        hasher.verify(wrong1, &salt, &hash);
        let time1 = elapsed(start);

        let start = now();
        hasher.verify(wrong2, &salt, &hash);
        let time2 = elapsed(start);

        // Times should be within 1% (constant-time)
        assert!((time1 as f64 - time2 as f64).abs() / time1 as f64 < 0.01);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_login_flow() {
    let provider = TestProvider::new(&[
        ("admin", "hashed_password", AccessLevel::Admin),
    ]);

    let mut service = CliService::new(root, provider, test_io);

    // Should start logged out
    assert_eq!(service.state(), CliState::LoggedOut);

    // Invalid login
    assert!(service.login("admin", "wrong").is_err());
    assert_eq!(service.state(), CliState::LoggedOut);

    // Valid login
    assert!(service.login("admin", "correct").is_ok());
    assert_eq!(service.state(), CliState::LoggedIn);
}

#[test]
fn test_access_control() {
    let provider = TestProvider::new(&[
        ("user", "hash", AccessLevel::User),
        ("admin", "hash", AccessLevel::Admin),
    ]);

    let mut service = CliService::new(root, provider, test_io);

    // Login as user
    service.login("user", "password").unwrap();

    // Can access User-level commands
    assert!(service.execute("system info").is_ok());

    // Cannot access Admin-level commands
    assert_eq!(
        service.execute("system reboot"),
        Err(CliError::AccessDenied)
    );
}
```

### Security Tests

```rust
#[test]
fn test_no_plaintext_in_binary() {
    // Ensure passwords are not stored in plaintext
    let binary = include_bytes!(env!("CARGO_BIN_FILE_CLI_SERVICE"));
    let binary_str = String::from_utf8_lossy(binary);

    // Should not find plaintext passwords
    assert!(!binary_str.contains("admin123"));
    assert!(!binary_str.contains("user123"));
}

#[test]
fn test_salt_uniqueness() {
    // Ensure different users have different salts
    let users = load_users();
    let salts: HashSet<_> = users.iter().map(|u| u.salt).collect();

    assert_eq!(salts.len(), users.len(), "Salts must be unique per user");
}
```

---

## Security Assumptions

### Threat Model

**In Scope:**
- ✅ Password guessing via serial console
- ✅ Binary inspection for credential extraction
- ✅ Memory dumps from running device
- ✅ Timing attacks during password verification
- ✅ Rainbow table attacks on leaked hashes

**Out of Scope:**
- ⚠️ Physical attacks (JTAG, flash extraction)
- ⚠️ Side-channel attacks (power analysis, EM)
- ⚠️ Supply chain attacks (malicious firmware)
- ⚠️ Social engineering

### Physical Security Assumptions

**This authentication system assumes:**

1. **Physical Access Control**
   - Device is in secured enclosure or location
   - UART/USB ports not publicly accessible
   - Attacker cannot easily extract flash chip

2. **Authorized Serial Access**
   - Serial console access implies some level of authorization
   - Not protecting against sophisticated physical attacks
   - Focus on preventing casual/accidental unauthorized access

3. **No Network Exposure**
   - CLI accessible only via local serial connection
   - No remote authentication required
   - No network-based brute force attacks possible

### When This System Is Sufficient

✅ **Good for:**
- Embedded devices in secured locations
- Lab equipment with physical access control
- Industrial control systems in restricted areas
- Development/debug interfaces on physical hardware
- Single-user devices with occasional configuration access

❌ **Insufficient for:**
- Network-exposed services
- Multi-tenant systems
- High-security applications (medical, aerospace, financial)
- Devices in publicly accessible locations
- Systems requiring compliance (HIPAA, PCI-DSS)

### Recommendations for High-Security Environments

If your threat model requires stronger protections:

1. **Use Argon2id** instead of SHA-256 (accept performance cost)
2. **Implement secure boot** with signed firmware
3. **Enable flash read protection** (RP2040 boot2 configuration)
4. **Add hardware security module** (separate crypto chip)
5. **Implement certificate-based auth** (mTLS, client certificates)
6. **Use external authentication** (LDAP, RADIUS, OAuth)
7. **Add audit logging** (tamper-evident log of all access)
8. **Implement 2FA** (TOTP, hardware tokens)

---

## Best Practices Summary

### DO ✅

- **Use SHA-256 with per-user salts** for embedded systems
- **Store credentials in flash** for production deployments
- **Use build-time env vars** to keep secrets out of source
- **Implement constant-time comparison** for password verification
- **Rate limit failed login attempts** (delay after failure)
- **Mask password input** (show asterisks after colon)
- **Validate access on entire path** (root to target node)
- **Use feature gates** for optional authentication
- **Generate unique salts** per user (hardware RNG)
- **Provide password change command** for administrators
- **Document security assumptions** clearly
- **Test for timing attacks** in password verification
- **Audit binaries** for plaintext credential leakage

### DON'T ❌

- **Never commit plaintext passwords** to version control
- **Never use same credentials** across all devices
- **Never skip salt** when hashing passwords
- **Never use variable-time comparison** (enables timing attacks)
- **Never store passwords** in easily extractable locations
- **Never allow unlimited login attempts** without delays
- **Never echo passwords** to console (even in debug builds)
- **Never hardcode production credentials** in examples
- **Never reuse salts** across users
- **Never ignore failed login attempts** (should log/alert)

---

## References

### Standards & Specifications

- **NIST FIPS 180-4**: SHA-256 Secure Hash Standard
- **NIST SP 800-63B**: Digital Identity Guidelines (Authentication)
- **OWASP ASVS**: Application Security Verification Standard
- **CWE-256**: Plaintext Storage of a Password
- **CWE-798**: Use of Hard-coded Credentials

### Rust Crates

- `sha2`: SHA-2 family cryptographic hashing
- `subtle`: Constant-time operations
- `heapless`: Static allocation data structures
- `rp2040-flash`: RP2040 flash memory access
- `rand`: Random number generation

### Additional Reading

- ["Cryptographic Right Answers"](https://www.latacora.com/blog/2018/04/03/cryptographic-right-answers/) by Colin Percival
- "Password Storage Cheat Sheet" (OWASP)
- Raspberry Pi Pico Datasheet (RP2040 security features)

---

## Changelog

- **2025-11-09**: Initial security architecture document
  - Authentication system design
  - Password hashing approach (SHA-256)
  - Credential storage options
  - Migration guide from C++ implementation

---

**Document Status:** Draft
**Last Updated:** 2025-11-09
**Author:** CLIService Rust Port Team
**Review Status:** Pending Security Review
