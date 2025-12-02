# SECURITY

## Authentication & Access Control Security Design

This document describes the security architecture for authentication and access control in **nut-shell**, including design rationale, implementation patterns, and deployment guidance.

---

## Table of Contents

1. [Security Model Overview](#security-model-overview)
2. [Password Hashing](#password-hashing)
3. [Access Control System](#access-control-system)
4. [Implementation Patterns](#implementation-patterns)
5. [Security Requirements](#security-requirements)
6. [Threat Model & Assumptions](#threat-model--assumptions)
7. [Deployment Guidance](#deployment-guidance)
8. [Testing & Validation](#testing--validation)
9. [See Also](#see-also)

---

## Security Model Overview

### Core Design Principles

1. **No plaintext passwords** - Only SHA-256 hashes stored; passwords never appear in source code or flash
2. **Trait-based providers** - `CredentialProvider` trait for extensible credential storage
3. **Per-device secrets** - Flash storage enables unique credentials per device
4. **Optional authentication** - Feature-gated (`authentication` feature flag)
5. **Constant-time comparison** - Prevents timing-based password guessing
6. **Per-user salts** - Prevents rainbow table attacks

### Architecture

Authentication uses the `CredentialProvider` trait for credential lookup and verification:

```rust
pub trait CredentialProvider<L: AccessLevel> {
    fn find_user(&self, username: &str) -> Result<Option<User<L>>, Self::Error>;
    fn verify_password(&self, user: &User<L>, password: &str) -> bool;
}

pub struct User<L: AccessLevel> {
    pub username: heapless::String<32>,
    pub password_hash: [u8; 32],  // SHA-256
    pub salt: [u8; 16],           // Per-user salt
    pub access_level: L,          // User's permission level
}
```

**Flow:** Shell calls `find_user()` to retrieve user data, then `verify_password()` to check credentials. Implementations can source credentials from const data, flash storage, or external systems.

### Limitations

**Not included by design:**
- **Rate limiting** - Implement in `CredentialProvider` if threat model requires
- **Audit logging** - Add in application code if compliance requires
- **Multi-factor authentication** - Use external auth systems (LDAP, RADIUS) via custom provider
- **Bcrypt/Argon2** - Memory constraints favor SHA-256 with salts (see Password Hashing section)

**Physical security assumptions:**
- Device in controlled environment with monitored physical access
- Serial console accessible locally only (not network-exposed)
- Physical access required limits brute-force attack window
- Not designed for sophisticated physical attacks (JTAG, flash extraction)

---

## Password Hashing

### SHA-256 Choice Rationale

| Factor | SHA-256 | bcrypt/Argon2 |
|--------|---------|---------------|
| Memory Usage | ~1KB | ~16KB - 1MB |
| Computation Speed | Fast (~1μs) | Slow by design (10-100ms) |
| Brute-Force Resistance | ⚠️ Weak (billions of attempts/sec) | ✅ Strong (intentionally slow) |
| Rainbow Table Resistance | ✅ Strong with salt | ✅ Strong with salt |
| Embedded Suitability | ✅ Excellent | ⚠️ Challenging |

**Decision:** SHA-256 with per-user salts is a **security tradeoff** for embedded constraints. It provides rainbow table resistance but limited brute-force protection. Acceptable only for devices in controlled environments where physical access limits attack window.

### Implementation

Each user has a stored salt and password hash. During login, the provided password is hashed with the user's salt and compared against the stored hash.

**Hash computation:** SHA-256 of salt concatenated with password bytes:

```rust
use sha2::{Sha256, Digest};
use subtle::ConstantTimeEq;

fn hash_password(password: &str, salt: &[u8; 16]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(salt);              // Salt first
    hasher.update(password.as_bytes()); // Then password
    hasher.finalize().into()
}
```

**Verification:** Constant-time comparison prevents timing attacks:

```rust
fn verify_password(password: &str, salt: &[u8; 16], expected_hash: &[u8; 32]) -> bool {
    let computed_hash = hash_password(password, salt);
    computed_hash.ct_eq(expected_hash).into()  // Constant-time comparison
}
```

Users implement the `CredentialProvider` trait to supply credentials (see Implementation Patterns section).

### Salt Generation

**Purpose:** Salts prevent attackers from using precomputed rainbow tables. Each user gets a unique 128-bit (16 byte) salt stored alongside their password hash.

| Approach | Salt Source | Use Case | Per-Device Unique |
|----------|-------------|----------|-------------------|
| **Build-time** | Hardcoded constants | Development, testing | ❌ No (same across all devices) |
| **Runtime** | Hardware RNG | Production deployments | ✅ Yes |

**Build-time approach:**

Salts are hardcoded constants in source code. Hash is precomputed offline:

```rust
// 1. Generate random salt offline (e.g., openssl rand -hex 16)
const ADMIN_SALT: [u8; 16] = *b"a3f9d2c8e1b4567a";

// 2. Compute hash offline using hash_password() function
const ADMIN_HASH: [u8; 32] = [
    0x5a, 0x2b, /* ... 32 bytes total ... */
];
```

**Runtime approach:**

Generate salt during user creation or provisioning:

```rust
// During manufacturing/provisioning:
let mut salt = [0u8; 16];
hardware_rng.fill_bytes(&mut salt);  // Fill with random bytes

let password = "initial_password";
let hash = hash_password(password, &salt);

// Store both salt and hash in flash memory
flash.write_user(User { username, salt, password_hash: hash, access_level });
```

**Security note:** Build-time salts protect against rainbow tables but are identical across all devices built from the same source. Production systems should use hardware RNG to generate per-device unique salts.

---

## Access Control System

### `AccessLevel` Trait

Define hierarchical permission levels using the `AccessLevel` trait. The trait requires `PartialOrd` so higher levels automatically inherit permissions from lower levels.

**Example:**
```rust
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
pub enum MyAccessLevel {
    Guest = 0,   // Lowest permissions
    User = 1,    // Guest permissions + user commands
    Admin = 2,   // All permissions
}
```

**See [EXAMPLES.md](EXAMPLES.md#custom-accesslevel-implementation) for complete implementation patterns.**

### Path-Based Access Validation

Every node in the command tree has a minimum `AccessLevel`. When a user navigates or executes commands, the shell checks their access level at each step.

**Security properties:**
- Access checked at **every path segment** during traversal - Parents access doesn't automatically grant child access
- Inaccessible nodes return `InvalidPath` (indistinguishable from non-existent nodes) - Prevents information leakage about protected command existence

**Example:** User with `Guest` access attempting `system/status` fails at the `system/` directory, not at the `status` command.

---

## Implementation Patterns

### Build-Time Credentials

Credentials are hardcoded in source as const values. The password comes from an environment variable at build time to avoid committing secrets.

**Complete example:**

```rust
use sha2::{Sha256, Digest};
use subtle::ConstantTimeEq;

// 1. Hardcoded salt (generated offline, unique per user)
const ADMIN_SALT: [u8; 16] = *b"a3f9d2c8e1b4567a";

// 2. Hash computed at build time from env var
const ADMIN_HASH: [u8; 32] = {
    // In reality, compute this offline and paste here
    // This shows the concept: hash = SHA256(salt || password)
    [0x5a, 0x2b, /* ... precomputed hash bytes ... */]
};

struct BuildTimeProvider;

impl CredentialProvider<MyAccessLevel> for BuildTimeProvider {
    type Error = core::convert::Infallible;

    fn find_user(&self, username: &str) -> Result<Option<User<MyAccessLevel>>, Self::Error> {
        match username {
            "admin" => Ok(Some(User {
                username: heapless::String::from("admin"),
                password_hash: ADMIN_HASH,
                salt: ADMIN_SALT,
                access_level: MyAccessLevel::Admin,
            })),
            _ => Ok(None),
        }
    }

    fn verify_password(&self, user: &User<MyAccessLevel>, password: &str) -> bool {
        let computed_hash = hash_password(password, &user.salt);
        computed_hash.ct_eq(&user.password_hash).into()
    }
}
```

**Workflow:**
1. Generate random salt offline: `openssl rand -hex 16`
2. Compute hash offline using `hash_password()` function from Password Hashing section
3. Hardcode both salt and hash as const values
4. Build and deploy

**Use cases:**
- Development and testing environments
- Single-device deployments
- Small-batch production where per-device credentials aren't required

**Limitations:**
- All devices built from same source have identical credentials
- Changing credentials requires rebuild and reflash
- Hash can be extracted from binary with tools like `strings` or `objdump`

### Flash-Based Credentials

For production deployments, store credentials in flash memory with per-device provisioning:

```rust
struct FlashCredentialProvider {
    flash: &'static FlashStorage,  // Your flash driver
}

impl CredentialProvider<MyAccessLevel> for FlashCredentialProvider {
    type Error = FlashError;

    fn find_user(&self, username: &str) -> Result<Option<User<MyAccessLevel>>, Self::Error> {
        // Read from flash memory
        self.flash.read_user(username)
    }

    fn verify_password(&self, user: &User<MyAccessLevel>, password: &str) -> bool {
        let computed_hash = hash_password(password, &user.salt);
        computed_hash.ct_eq(&user.password_hash).into()
    }
}
```

**Provisioning workflow:**
1. During manufacturing, generate unique salt per device using hardware RNG
2. Set initial password (from secure provisioning station)
3. Write salt and hash to flash using provisioning tool
4. Deploy device with unique credentials

**Benefits:**
- Per-device unique credentials
- Updateable without firmware reflash
- Can implement password change commands

### Credential Storage Comparison

| Approach | Updateable | Per-Device Unique | Use Case |
|----------|------------|-------------------|----------|
| Build-time | ❌ No | ❌ No | Development, testing, single device |
| Flash storage | ✅ Yes | ✅ Yes | Production deployments |
| External (LDAP/RADIUS) | ✅ Yes | ✅ Yes | Enterprise integration |

---

## Security Requirements

### Authentication Flow

When authentication is enabled, users must log in before accessing commands:

1. **Activation** - Call `activate()` to transition from `Inactive` to `LoggedOut` state
2. **Login prompt** - Shell displays `login: ` prompt
3. **Input format** - User enters `username:password` (colon-separated)
4. **User lookup** - Shell calls `CredentialProvider::find_user(username)`
5. **Password verification** - If user found, `verify_password()` checks credentials using constant-time comparison
6. **State transition** - On success, shell transitions from `LoggedOut` to `LoggedIn` state
7. **Rate limiting** (optional) - Implement in `CredentialProvider` if threat model requires protection against brute-force attempts

**Without authentication feature:** `activate()` transitions directly from `Inactive` to `LoggedIn` with no login prompt.

### Access Control Enforcement

Access control is enforced at every step through the command tree:

1. **Path resolution** - When user types `system/status`, each segment (`system/`, then `status`) is checked against `current_user.access_level >= node.min_access_level`
2. **Command execution** - Before dispatching to handler, verify user has sufficient access level
3. **Error uniformity** - Return `CliError::InvalidPath` for both non-existent AND inaccessible nodes to prevent information leakage
4. **No implicit inheritance** - Access to parent directory doesn't grant access to children

**Example:** If `system/` requires `Admin` access, a `Guest` user attempting `system/status` receives `InvalidPath` - neither the directory nor the command's existence is revealed.

### Password Input Security

When entering credentials at the login prompt:

- **Partial masking** - Echo characters normally until `:` detected, then mask all subsequent characters with `*`
- **Backspace handling** - Properly remove masked characters from buffer when user presses backspace
- **No echo of password** - Password portion never appears in plaintext on terminal

**Example:** User typing `admin:secret` sees `admin:******` on screen.

### Feature Gating

Authentication is **opt-in** via the `authentication` Cargo feature:

```bash
# With authentication (login required)
cargo build --features authentication

# Without authentication (no login, full access)
cargo build
```

The unified architecture pattern (see [DESIGN.md](DESIGN.md) Section 2) ensures a single code path for both modes - `CliState` drives behavior instead of `#[cfg]` branching throughout the codebase.

---

## Threat Model & Assumptions

### Threats Addressed

| Threat | Mitigation |
|--------|------------|
| **Timing attacks during password verification** | `subtle::ConstantTimeEq` ensures constant-time comparison regardless of password correctness |
| **Rainbow table attacks** | Per-user salts make precomputed tables ineffective |
| **Credential extraction from binary/memory** | Only hashes stored (not plaintext); attacker still needs to brute-force |
| **Brute-force via serial console** | Optional rate limiting in `CredentialProvider` (not included by default) |

### Threats Not Addressed

These threats are **out of scope** for this authentication system:

| Threat | Why Not Addressed | Mitigation if Required |
|--------|-------------------|------------------------|
| **Physical attacks** (JTAG, flash extraction) | Assumed secured enclosure | Enable flash read protection, use hardware security module |
| **Brute-force attacks** | SHA-256 is fast by design | Implement rate limiting in `CredentialProvider`, or use Argon2id |
| **Side-channel attacks** (power analysis, EM) | Requires specialized equipment and access | Use constant-time implementations throughout, add physical shielding |
| **Supply chain attacks** | Trusted build/deployment environment assumed | Implement secure boot, code signing |
| **Social engineering** | Human factor, not technical control | Security training, operational procedures |

### Physical Security Assumptions

This system is designed for **embedded devices in controlled environments**:

1. **Physical access control** - Device location is monitored or secured (lab, locked cabinet, controlled facility)
2. **Local serial access only** - CLI accessible via UART/USB, not exposed over network
3. **Limited attack window** - Physical access required for password guessing attempts

**Key limitation:** SHA-256's speed allows billions of password attempts per second. This system relies on physical security to limit attacker access time and attempts.

**If these assumptions don't hold** (e.g., network-exposed serial-over-IP), this authentication system is insufficient. See Deployment Guidance below.

---

## Deployment Guidance

### When This System Is Appropriate

**✅ Use this authentication system for:**

| Scenario | Rationale |
|----------|-----------|
| Lab equipment and development tools | Physical security assumed, convenience prioritized |
| Industrial control panels in secured facilities | Locked cabinets provide physical security layer |
| Single-user embedded devices | Occasional configuration access, not multi-tenant |
| Debug/diagnostic interfaces | Physical access required, limited attack window |

**❌ This system is insufficient for:**

| Scenario | Why Insufficient | Alternative |
|----------|------------------|-------------|
| Network-exposed services | No rate limiting, fast hash function | Use TLS with certificate auth, Argon2id hashing |
| High-security applications (medical, aerospace, financial) | Regulatory requirements not met | Implement Argon2id, hardware security module, audit logging |
| Publicly accessible devices | Physical security assumptions violated | Add tamper detection, secure boot, hardware-based auth |
| Compliance requirements (HIPAA, PCI-DSS) | Lacks audit logging, key rotation | Implement comprehensive logging, use certified crypto modules |

### Upgrading Security

If your threat model requires stronger protections than provided:

**Replace SHA-256 with Argon2id:**

```rust
// Replace hash_password() with Argon2id
// Requires more RAM (~16KB+) but provides strong brute-force resistance
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

fn hash_password_argon2(password: &str, salt: &[u8; 16]) -> [u8; 32] {
    // Configure Argon2id with appropriate memory/iteration parameters
    // See argon2 crate documentation for embedded-appropriate settings
}
```

**Add rate limiting:**

```rust
impl CredentialProvider<MyAccessLevel> for RateLimitedProvider {
    fn verify_password(&self, user: &User<MyAccessLevel>, password: &str) -> bool {
        let result = constant_time_verify(password, &user.salt, &user.password_hash);

        if !result {
            // Delay 3 seconds on failure to slow brute-force attempts
            delay_ms(3000);
        }

        result
    }
}
```

**Enable flash read protection:**

Consult your MCU's reference manual for bootloader configuration. Most microcontrollers support read protection levels that prevent JTAG/debug access to flash contents.

---

## Testing & Validation

Security tests verify authentication flow, access control enforcement, and error uniformity. See `tests/test_auth_*.rs` for implementation details.

```bash
cargo test --all-features
```

---

## See Also

- **[DESIGN.md](DESIGN.md)** - Unified architecture pattern and feature gating
- **[PHILOSOPHY.md](PHILOSOPHY.md)** - Security-by-design philosophy
- **[EXAMPLES.md](EXAMPLES.md)** - `AccessLevel` implementation patterns
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Build commands and testing workflows
