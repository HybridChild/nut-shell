# SECURITY.md

## Authentication & Access Control Security Design

This document describes the security architecture for authentication and access control in the Rust cli-service implementation, including rationale, implementation patterns, and best practices for embedded systems.

**Related Documentation:**
- **[DESIGN.md](DESIGN.md)** - Command architecture, unified auth pattern, and feature gating details
- **[INTERNALS.md](INTERNALS.md)** - Complete authentication flow and state machine implementation
- **[SPECIFICATION.md](SPECIFICATION.md)** - Behavioral specification for authentication (prompts, messages, login flow)
- **[PHILOSOPHY.md](PHILOSOPHY.md)** - Security-by-design philosophy and feature criteria
- **[IMPLEMENTATION.md](IMPLEMENTATION.md)** - Authentication implementation roadmap

---

## Table of Contents

1. [Security Vulnerabilities & Concerns](#security-vulnerabilities--concerns)
2. [Rust Implementation Security Design](#rust-implementation-security-design)
3. [Password Hashing](#password-hashing)
4. [Credential Storage Options](#credential-storage-options)
5. [Access Control System](#access-control-system)
6. [Implementation Patterns](#implementation-patterns)
7. [Testing & Validation](#testing--validation)
8. [Security Assumptions](#security-assumptions)

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

    #[cfg(feature = "authentication")]
    pub password_hash: [u8; 32],  // SHA-256 hash

    #[cfg(feature = "authentication")]
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

The `CredentialProvider` trait enables multiple storage backends:

### 1. Build-Time Environment Variables
- Credentials configured via environment variables during build
- Const data stored in ROM
- Suitable for single-device or small-batch deployments
- No credentials in source control

### 2. Flash Storage (Production Recommended)
- Per-device unique credentials
- Updateable without recompilation
- Survives firmware updates (separate flash sector)
- Requires provisioning process and flash write capability

### 3. Const Provider (Examples/Testing Only)
- Hardcoded credentials in binary
- Simple implementation for examples and tests
- NOT suitable for production

### 4. Custom Trait-Based Provider
- Maximum flexibility for specialized backends
- Can integrate with existing infrastructure (LDAP, HSM, etc.)
- Implementation-specific security properties

---

## Access Control System

### Generic AccessLevel Trait

**Note:** Use `AccessLevel` (CamelCase) when referring to the trait type, "access level" (lowercase) when discussing the concept.

```rust
pub trait AccessLevel: Copy + Clone + PartialEq + PartialOrd {
    fn from_str(s: &str) -> Option<Self>;
    fn as_str(&self) -> &'static str;
}

// User-defined enum (implement your own hierarchy)
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

Access control is enforced during path resolution, checking each segment as the tree is traversed:

```rust
impl<'tree, L: AccessLevel, IO: CharIo> CliService<'tree, L, IO> {
    fn resolve_path(&self, path: &Path) -> Result<&Node<L>, CliError> {
        let mut current = self.get_current_directory();

        for segment in path.segments() {
            // Find child by name
            let child = current.find_child(segment)
                .ok_or(CliError::InvalidPath)?;

            // Check access to this node
            self.check_access(child)?;
            //   └─ Returns InvalidPath if denied (see implementation above)

            // Continue traversal if directory
            if let Node::Directory(dir) = child {
                current = dir;
            } else {
                return Ok(child);  // Command found
            }
        }

        Ok(Node::Directory(current))
    }

    // Access check implementation shown in "Access Control Implementation Pattern" above
}
```

**Key security properties:**
- Access checked **at every path segment** during traversal
- Inaccessible nodes return same error as non-existent nodes
- No information leakage about node existence
- Parent directory access doesn't imply child access (each node checked independently)
- See INTERNALS.md for complete resolve_path() implementation with access control

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

## Authentication Feature Gating & Unified Architecture

Authentication can be disabled via Cargo features for unsecured development environments.

**Architecture:** The implementation uses a unified architecture approach with a single code path for both auth-enabled and auth-disabled modes. See [DESIGN.md](DESIGN.md) Section 5.2 "Unified Architecture Pattern" for complete details on the state machine, field organization, and implementation benefits.

**Access Control Flow:** See [INTERNALS.md](INTERNALS.md) Level 4 "Path Parsing & Tree Navigation" for the complete access control enforcement implementation during tree traversal.

**Security-Specific Considerations:**

1. **InvalidPath Error Hiding**: When access is denied, the system returns `CliError::InvalidPath` (same as non-existent paths) to prevent revealing the existence of restricted commands to unauthorized users.

2. **Handler Dispatch Security**: Access control checks occur BEFORE dispatching to `CommandHandlers`. Handlers receive only pre-validated, accessible commands, centralizing security in CliService rather than distributing it across handler implementations.

3. **Build Configuration:**
   ```bash
   # With authentication (default)
   cargo build

   # Without authentication (no login required)
   cargo build --no-default-features
   ```

See [DESIGN.md](DESIGN.md) for feature gating patterns and [SPECIFICATION.md](SPECIFICATION.md) for behavioral specifications in both modes.

---

## Implementation Requirements

### Login Flow
1. Parse login request (username:password format)
2. Find user via `CredentialProvider::find_user()`
3. Verify password using constant-time comparison
4. Rate limit failed attempts (minimum 1 second delay)
5. Update session state on success

### Password Masking
- Echo characters normally until colon detected
- Mask all characters after colon with asterisks
- Backspace must properly handle masked characters
- See SPECIFICATION.md for complete terminal behavior

### Access Control Enforcement
- Check access level at every path segment during tree traversal
- Return `CliError::InvalidPath` for both nonexistent and inaccessible nodes
- Verify access before dispatching to CommandHandlers
- See INTERNALS.md Level 4 for complete implementation details

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

## Security Requirements Summary

**Password Hashing:**
- Use SHA-256 with per-user salts
- Generate unique 128-bit salts per user
- Implement constant-time comparison to prevent timing attacks
- Store hashed credentials only (never plaintext)

**Credential Storage:**
- Keep credentials out of source control
- Use build-time configuration or flash storage
- Support per-device unique credentials
- Enable credential rotation capability

**Access Control:**
- Validate access at every path segment during traversal
- Return identical errors for nonexistent and inaccessible nodes
- Enforce access before command dispatch
- Use generic AccessLevel trait for user-defined hierarchies

**Authentication Flow:**
- Rate limit failed login attempts (minimum 1 second delay)
- Mask password input after colon character
- Use unified architecture pattern (single code path)
- Feature-gate authentication for optional use

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

## See Also

- **[DESIGN.md](DESIGN.md)** - Unified architecture pattern, feature gating, and authentication design decisions
- **[INTERNALS.md](INTERNALS.md)** - Complete authentication flow, state machine, and access control enforcement
- **[SPECIFICATION.md](SPECIFICATION.md)** - Behavioral specification (login flow, prompts, password masking)
- **[PHILOSOPHY.md](PHILOSOPHY.md)** - Security-by-design philosophy and feature decision framework
- **[IMPLEMENTATION.md](IMPLEMENTATION.md)** - Authentication implementation roadmap and testing strategy
- **[../CLAUDE.md](../CLAUDE.md)** - Working patterns and practical implementation guidance
