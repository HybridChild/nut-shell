# SECURITY

## Authentication & Access Control Security Design

This document describes the security architecture for authentication and access control in nut-shell, including design rationale, implementation patterns, and deployment guidance.

**Related Documentation:**
- **[DESIGN.md](DESIGN.md)** - Unified architecture pattern and feature gating details
- **[PHILOSOPHY.md](PHILOSOPHY.md)** - Security-by-design philosophy
- **[EXAMPLES.md](EXAMPLES.md)** - `AccessLevel` implementation and usage patterns
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Build and testing workflows

---

## Table of Contents

1. [Security Model Overview](#security-model-overview)
2. [Password Hashing](#password-hashing)
3. [Access Control System](#access-control-system)
4. [Implementation Patterns](#implementation-patterns)
5. [Security Requirements](#security-requirements)
6. [Threat Model & Assumptions](#threat-model--assumptions)
7. [Deployment Guidance](#deployment-guidance)

---

## Security Model Overview

### Core Design Principles

1. **No plaintext passwords** - SHA-256 hashed credentials only
2. **No credentials in source** - Build-time or runtime configuration
3. **Trait-based providers** - Extensible credential storage
4. **Per-device secrets** - Flash storage enables unique credentials
5. **Optional authentication** - Feature-gated for flexibility
6. **Constant-time comparison** - Prevents timing attacks
7. **Per-user salts** - Prevents rainbow table attacks

### Architecture

```rust
pub trait CredentialProvider<L: AccessLevel> {
    fn find_user(&self, username: &str) -> Result<Option<User<L>>, Self::Error>;
    fn verify_password(&self, user: &User<L>, password: &str) -> bool;
}

pub struct User<L: AccessLevel> {
    pub username: heapless::String<32>,
    pub password_hash: [u8; 32],  // SHA-256
    pub salt: [u8; 16],           // Per-user salt
    pub access_level: L,
}
```

### Limitations

**Not included by design:**
- **Rate limiting** - Implement in `CredentialProvider` if threat model requires
- **Audit logging** - Add in application code if compliance requires
- **Multi-factor authentication** - Use external auth systems (LDAP, RADIUS) via custom provider
- **Bcrypt/Argon2** - Memory constraints favor SHA-256 with salts (see Password Hashing section)

**Physical security assumptions:**
- Device in secured enclosure or controlled environment
- Serial console access implies some authorization level
- Not designed for sophisticated physical attacks (JTAG, flash extraction)

---

## Password Hashing

### SHA-256 Choice Rationale

| Factor | SHA-256 | bcrypt/Argon2 |
|--------|---------|---------------|
| Memory Usage | ~1KB | ~16KB - 1MB |
| Computation | Fast (~1μs) | Slow by design (10-100ms) |
| Embedded Suitability | ✅ Excellent | ⚠️ Challenging |
| Security with Salt | ✅ Strong | ✅ Stronger |
| Typical MCU RAM (~256KB) | ✅ Well-suited | ⚠️ Memory constrained |

**Decision:** SHA-256 with per-user salts provides sufficient security for physically-secured embedded devices with low memory footprint and fast verification.

### Implementation

```rust
use sha2::{Sha256, Digest};
use subtle::ConstantTimeEq;

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
        // Constant-time comparison prevents timing attacks
        computed.ct_eq(hash).into()
    }
}
```

### Salt Generation

**Per-user salts** (128-bit/16 bytes):
- Prevents rainbow table attacks across users
- Generated once during user creation
- Stored alongside password hash

**Sources:**
- **Runtime:** Hardware RNG (e.g., MCU ring oscillator) for flash-stored credentials
- **Build-time:** Environment variables (see Implementation Patterns below)

---

## Access Control System

### `AccessLevel` Trait

User-defined access level hierarchies via the `AccessLevel` trait. The trait requires `PartialOrd` for hierarchical comparison (higher levels have all permissions of lower levels).

**See [EXAMPLES.md](EXAMPLES.md#custom-accesslevel-implementation) for implementation examples using the derive macro or manual implementation.**

### Path-Based Access Validation

Access control enforced during path resolution:

**Security properties:**
- Access checked at **every path segment** during tree traversal
- Inaccessible nodes return `InvalidPath` (same as non-existent nodes)
- No information leakage about node existence
- Parent access doesn't imply child access

**Implementation:** See [DESIGN.md](DESIGN.md) for path resolution and unified architecture pattern.

---

## Implementation Patterns

### Build-Time Credentials

Use environment variables to configure credentials at build time:

```rust
const ADMIN_SALT: [u8; 16] = *b"random_salt_0001";
const ADMIN_HASH: [u8; 32] = /* hash of env!("ADMIN_PASSWORD") with salt */;

struct BuildTimeProvider;

impl CredentialProvider<MyAccessLevel> for BuildTimeProvider {
    fn find_user(&self, username: &str) -> Option<User<MyAccessLevel>> {
        match username {
            "admin" => Some(User {
                username: heapless::String::from("admin"),
                password_hash: ADMIN_HASH,
                salt: ADMIN_SALT,
                access_level: MyAccessLevel::Admin,
            }),
            _ => None,
        }
    }

    fn verify_password(&self, user: &User<MyAccessLevel>, password: &str) -> bool {
        let hasher = Sha256Hasher;
        hasher.verify(password, &user.salt, &user.password_hash)
    }
}

// Build with: ADMIN_PASSWORD=secret123 cargo build
```

**Use cases:**
- Single-device or small-batch deployments
- Development/lab equipment
- No credentials in source control

**Limitations:**
- Same credentials across all builds (unless scripted per-device)
- Requires rebuild to change credentials
- Hash extractable from binary

**For production:** Use flash storage with per-device provisioning.

### Credential Storage Options

| Storage | Use Case | Updateable | Per-Device |
|---------|----------|------------|------------|
| Build-time env vars | Development, small batch | No | Manual scripting |
| Flash storage | Production | Yes | Yes |
| Const provider | Examples/testing | No | No |
| Custom provider | Specialized (LDAP, HSM) | Varies | Varies |

---

## Security Requirements

### Authentication Flow

1. Parse login request (username:password format)
2. Find user via `CredentialProvider::find_user()`
3. Verify password using constant-time comparison
4. Update session state on success
5. **(Optional)** Implement rate limiting in `CredentialProvider` if threat model requires

### Password Input Masking

- Echo characters normally until colon detected
- Mask all characters after colon with asterisks
- Backspace must properly handle masked characters

### Access Control Enforcement

- Check access level at every path segment during tree traversal
- Return `CliError::InvalidPath` for both nonexistent and inaccessible nodes (prevents information leakage)
- Verify access before dispatching to `CommandHandler`

### Feature Gating

Authentication is opt-in via Cargo features. See [DESIGN.md](DESIGN.md) Section 2 for unified architecture pattern supporting both modes.

```bash
# With authentication
cargo build --features authentication

# Without authentication (default)
cargo build
```

---

## Threat Model & Assumptions

### Threats In Scope

- ✅ Password guessing via serial console
- ✅ Binary inspection for credential extraction
- ✅ Memory dumps from running device
- ✅ Timing attacks during password verification
- ✅ Rainbow table attacks on leaked hashes

### Threats Out of Scope

- ⚠️ Physical attacks (JTAG debugging, flash extraction)
- ⚠️ Side-channel attacks (power analysis, EM)
- ⚠️ Supply chain attacks (malicious firmware)
- ⚠️ Social engineering

### Physical Security Assumptions

This authentication system assumes:

1. **Device in secured location** - Enclosure or controlled environment
2. **Serial console not publicly accessible** - UART/USB ports protected
3. **No network exposure** - CLI accessible only via local serial connection

---

## Deployment Guidance

### When This System Is Sufficient

**✅ Appropriate for:**
- Embedded devices in secured locations (lab equipment, industrial control)
- Development/debug interfaces with physical access control
- Single-user devices with occasional configuration access
- Environments where serial access implies some authorization level

**❌ Insufficient for:**
- Network-exposed services or multi-tenant systems
- High-security applications (medical, aerospace, financial)
- Devices in publicly accessible locations
- Systems requiring compliance (HIPAA, PCI-DSS)

### Production Deployment Recommendations

**For multi-device deployments:**
- Use flash storage with per-device unique credentials
- Implement provisioning process during manufacturing
- Consider credential rotation capability

**For higher security requirements:**
- Implement Argon2id instead of SHA-256 (accept performance cost)
- Enable flash read protection (MCU-specific bootloader configuration)
- Add hardware security module (separate crypto chip)
- Implement certificate-based auth (mTLS, client certificates)
- Add audit logging with tamper-evident storage

---

## Testing & Validation

**Security test coverage:**
- Password hashing correctness and constant-time comparison
- Authentication flow state transitions
- Access control enforcement during path traversal
- Error uniformity (InvalidPath for both nonexistent and inaccessible nodes)

**See `tests/test_auth_*.rs` for complete test implementations.**

---

## See Also

- **[DESIGN.md](DESIGN.md)** - Unified architecture pattern and feature gating
- **[PHILOSOPHY.md](PHILOSOPHY.md)** - Security-by-design philosophy
- **[EXAMPLES.md](EXAMPLES.md)** - `AccessLevel` implementation patterns
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Build commands and testing workflows
