# SECURITY.md

## Authentication & Access Control Security Design

This document describes the security architecture for authentication and access control in the nut-shell implementation, including rationale, implementation patterns, and best practices for embedded systems.

**Related Documentation:**
- **[DESIGN.md](DESIGN.md)** - Command architecture, unified auth pattern, and feature gating details
- **[PHILOSOPHY.md](PHILOSOPHY.md)** - Security-by-design philosophy and feature criteria
- **[EXAMPLES.md](EXAMPLES.md)** - Authentication usage examples and patterns
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Build and testing workflows

---

## Table of Contents

1. [Security Considerations & Limitations](#security-considerations--limitations)
2. [Rust Implementation Security Design](#rust-implementation-security-design)
3. [Password Hashing](#password-hashing)
4. [Credential Storage Options](#credential-storage-options)
5. [Access Control System](#access-control-system)
6. [Implementation Patterns](#implementation-patterns)
7. [Testing & Validation](#testing--validation)
8. [Security Assumptions](#security-assumptions)

---

## Security Considerations & Limitations

### Design Limitations

#### 1. **SHA-256 vs. Password-Specific KDFs**
- **What we use**: SHA-256 with per-user salts
- **Industry standard**: bcrypt, Argon2, scrypt (designed for password hashing)
- **Why SHA-256**: Memory constraints (RP2040 has 264KB RAM), fast verification critical for embedded
- **Trade-off**: Less resistant to offline brute-force if hashes are extracted from flash
- **Mitigation**: Physical security assumption (see "Security Assumptions" section)

#### 2. **No Rate Limiting by Default**
- **Current design**: No built-in login attempt throttling or account lockout
- **Risk**: Brute-force attacks via serial console if device accessible
- **Recommendation**: Implement rate limiting in `CredentialProvider` if threat model requires
- **Why not built-in**: Adds complexity, timer dependencies, persistent state requirements

#### 3. **No Audit Logging**
- **Current design**: No logging of authentication events or command execution
- **Risk**: Cannot detect or investigate unauthorized access attempts
- **Recommendation**: Add logging in application code if compliance requires
- **Why not built-in**: Storage requirements, flash wear concerns, application-specific needs

#### 4. **Flash Extraction Risk**
- **Vulnerability**: Password hashes stored in flash can be extracted with physical access
- **Attack**: SWD/JTAG access, flash chip desoldering, firmware dump
- **Mitigation**: RP2040 flash read protection (boot2 configuration), secure boot
- **Assumption**: Physical security (device in controlled environment)

#### 5. **No Multi-Factor Authentication**
- **Current design**: Username/password only (single-factor)
- **Risk**: Compromised credentials grant full access
- **Recommendation**: Use external auth systems (LDAP, RADIUS) via custom `CredentialProvider`
- **Why not built-in**: Hardware token support, TOTP complexity, memory constraints

### Common Misuse Scenarios

#### ⚠️ **Using Const Provider in Production**
```rust
// WRONG: Example credentials in production deployment
const ADMIN: User = User {
    username: "admin",
    password_hash: hash_of_admin123,  // Shared across all devices!
    access_level: Admin,
};
```
**Consequence**: Same credentials on all devices, extractable from binary  
**Solution**: Use flash storage or build-time environment variables for per-device secrets

#### ⚠️ **Storing Plaintext in Custom Provider**
```rust
// WRONG: Custom provider with plaintext passwords
impl CredentialProvider for MyProvider {
    fn verify_password(&self, user: &User, password: &str) -> bool {
        user.plaintext_password == password  // DON'T DO THIS
    }
}
```
**Consequence**: Defeats entire security model  
**Solution**: Always hash passwords, use provided `Sha256Hasher` or better

#### ⚠️ **Disabling Authentication in Production**
```bash
# WRONG: Building production firmware without authentication
cargo build --target thumbv6m-none-eabi --no-default-features
```
**Consequence**: Anyone with serial access has full admin rights
**Solution**: Only disable `authentication` feature for development/trusted environments

#### ⚠️ **Weak Salt Generation**
```rust
// WRONG: Non-random or shared salts
const USER_SALT: [u8; 16] = [0; 16];  // All zeros = no salt benefit
const ADMIN_SALT: [u8; 16] = [0; 16]; // Same salt for all users
```
**Consequence**: Vulnerable to rainbow table attacks, hash reuse across users  
**Solution**: Generate unique random salts per user (hardware RNG or build-time random)

### Deployment Context & Risk Assessment

**Single-Device Embedded Systems (RP2040/Pico in Secured Location):**
- ✅ **Well-suited**: SHA-256 with salts provides adequate protection
- ✅ **Low risk**: Physical access to serial console implies some authorization level
- ✅ **Mitigated threats**: Casual/accidental access, credential extraction from binary
- ⚠️ **Assumed**: Device in physically secured enclosure or controlled environment
- ⚠️ **Out of scope**: Sophisticated physical attacks (JTAG debugging, flash extraction)

**Multi-Device Deployments (Production Fleet):**
- ✅ **Supports per-device secrets**: Flash storage or build-time configuration
- ✅ **Credential rotation capable**: Update via flash write, no recompilation required
- ✅ **No credentials in source**: Trait-based providers separate secrets from code
- ⚠️ **Requires provisioning process**: Must generate unique credentials per device during manufacturing/deployment
- ⚠️ **Implementation responsibility**: Application must implement secure flash storage provider

**Network-Exposed Systems:**
- ❌ **Not designed for this**: Serial console authentication assumes local/physical access
- ❌ **Missing protections**: No TLS, no certificate-based auth, no network-specific hardening
- ⚠️ **If exposed via serial-over-network**: Add network security layer (VPN, SSH tunneling, etc.)
- ⚠️ **Consider alternatives**: For network access, use proper network authentication (mTLS, OAuth, RADIUS)

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
// Build time: From environment variable or secure random source
// Example: Use openssl rand -hex 16 to generate, pass via build script
const ADMIN_SALT: [u8; 16] = [
    0x7a, 0x3f, 0x9e, 0x12, 0x8b, 0x4c, 0xd1, 0x56,
    0xe2, 0x91, 0x0a, 0x7f, 0xc3, 0x68, 0xb5, 0x2d,
];  // Generated from ADMIN_SALT_HEX env var via build.rs

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

Access control is enforced during path resolution. Each segment is checked during tree traversal.

**Security properties:**
- Access checked at **every path segment** during traversal
- Inaccessible nodes return `InvalidPath` (same as non-existent nodes)
- No information leakage about node existence
- Parent access doesn't imply child access

**Implementation details:** See [DESIGN.md](DESIGN.md) for path resolution and node access level patterns.

---

## Authentication Feature Gating & Unified Architecture

Authentication can be disabled via Cargo features for unsecured development environments.

**Architecture:** The implementation uses a unified architecture approach with a single code path for both auth-enabled and auth-disabled modes. See [DESIGN.md](DESIGN.md) Section 5.2 "Unified Architecture Pattern" for complete details on the state machine, field organization, and implementation benefits.

**Security-Specific Considerations:**

1. **InvalidPath Error Hiding**: When access is denied, the system returns `CliError::InvalidPath` (same as non-existent paths) to prevent revealing the existence of restricted commands to unauthorized users.

2. **Handler Dispatch Security**: Access control checks occur BEFORE dispatching to `CommandHandler`. Handlers receive only pre-validated, accessible commands, centralizing security in Shell rather than distributing it across handler implementations.

3. **Build Configuration:**
   ```bash
   # With authentication (default)
   cargo build

   # Without authentication (no login required)
   cargo build --no-default-features
   ```

See [DESIGN.md](DESIGN.md) for feature gating patterns and architectural details for both modes.

---

## Implementation Requirements

### Login Flow
1. Parse login request (username:password format)
2. Find user via `CredentialProvider::find_user()`
3. Verify password using constant-time comparison
4. Update session state on success
5. (Optional) Implement rate limiting in `CredentialProvider` if required by threat model

### Password Masking
- Echo characters normally until colon detected
- Mask all characters after colon with asterisks
- Backspace must properly handle masked characters

### Access Control Enforcement
- Check access level at every path segment during tree traversal
- Return `CliError::InvalidPath` for both nonexistent and inaccessible nodes
- Verify access before dispatching to CommandHandler

---

## Testing & Validation

**Test coverage:**
- **Password hashing:** SHA-256 correctness, salt uniqueness, constant-time comparison
- **Authentication flow:** Login/logout state transitions, invalid credentials handling
- **Access control:** Permission enforcement during path traversal, error uniformity (InvalidPath for both nonexistent and inaccessible)
- **Security assertions:** No plaintext password fields (compile-time check), timing attack resistance, per-user salt uniqueness

**See `tests/test_auth_*.rs` for complete test implementations.**

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
- Mask password input after colon character
- Use unified architecture pattern (single code path)
- Feature-gate authentication for optional use
- (Optional) Implement rate limiting in `CredentialProvider` if threat model requires

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
- **[PHILOSOPHY.md](PHILOSOPHY.md)** - Security-by-design philosophy and feature decision framework
- **[EXAMPLES.md](EXAMPLES.md)** - Authentication usage examples and patterns
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Build commands and testing workflows
- **[../CLAUDE.md](../CLAUDE.md)** - AI-assisted development guidance
