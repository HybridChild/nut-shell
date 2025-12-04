# Raspberry Pi Pico - Build-Time Credentials Example

Demonstrates **nut-shell** with build-time credential generation on RP2040 hardware.

This example shows how to use the `nut-shell-credgen` tool to generate hashed credentials at build time from a TOML configuration file. The credentials are compiled into the binary as const data - no runtime hashing, no heap allocation.

## How It Works

### Build Flow

```
credentials.toml   →   nut-shell-credgen   →   credentials.rs   →   final binary
(plaintext)            (build tool)            (hashed)            (compiled)
```

1. **Build script** (`build.rs`) runs during compilation
2. **credgen tool** reads `credentials.toml` with plaintext passwords
3. **Generates** `credentials.rs` with hashed credentials and random salts
4. **Include macro** pulls generated code into binary at compile time
5. **Final binary** contains only hashed credentials (no plaintext)

### Security Model

**Safe for binary:**
- Only hashed credentials compiled into firmware
- Each password gets unique random salt (generated at build time)
- Credentials are const-initializable (stored in flash, not RAM)

**Keep secret (add to .gitignore):**
- `credentials.toml` - contains plaintext passwords
- Should be managed separately from source code

**Auto-generated (in build artifacts):**
- `target/.../credentials.rs` - cleaned by `cargo clean`

## Prerequisites

```bash
rustup target add thumbv6m-none-eabi
cargo install probe-rs --features cli  # or elf2uf2-rs for UF2 bootloader
```

## Setup

### 1. Customize Credentials (Optional)

The example includes default credentials for demo purposes. For production or custom testing:

```bash
# Edit credentials.toml to change usernames/passwords
vim credentials.toml

# IMPORTANT: Add to .gitignore in real projects!
echo "credentials.toml" >> .gitignore
```

Default credentials (see `credentials.toml`):
- `admin:admin123` - Admin access
- `user:user123` - User access

### 2. Build and Flash

```bash
# Build with probe-rs (debug probe)
cargo build --release

# Flash with probe-rs
cargo run --release

# Or build UF2 for bootloader (no debug probe needed)
cargo build --release
elf2uf2-rs target/thumbv6m-none-eabi/release/buildtime buildtime.uf2
# Hold BOOTSEL button, connect USB, copy .uf2 to RPI-RP2 drive
```

### 3. Connect

```bash
# macOS/Linux
screen /dev/tty.usbmodem* 115200

# Or use your preferred serial terminal
```

## Available Commands

Same as standard rp-pico example:

```
/
├── system/
│   ├── info       - Device information
│   ├── uptime     - System uptime
│   └── meminfo    - Memory information
└── hardware/
    ├── get/
    │   ├── temp       - Temperature sensor
    │   └── gpio <pin> - GPIO status
    └── set/
        └── led <on|off> - Control onboard LED

Global:
  ?      - Help
  ls     - List directory
  clear  - Clear screen
  logout - End session
```

## Build Process Details

### What Happens During Build

1. **Cargo runs build.rs:**
   ```bash
   cargo run --bin nut-shell-credgen -- credentials.toml
   ```

2. **credgen generates Rust code:**
   ```rust
   const USERS: [User<PicoAccessLevel>; 2] = [
       User {
           username: heapless::String::from_str("admin").unwrap(),
           password_hash: [0x2c, 0xf2, ...],  // SHA-256 hash
           salt: [0x8a, 0x1b, ...],            // Random salt
           access_level: PicoAccessLevel::Admin,
       },
       // ...
   ];

   pub fn create_provider() -> BuildTimeProvider {
       ConstCredentialProvider::new(USERS, Sha256Hasher)
   }
   ```

3. **Main code includes it:**
   ```rust
   mod credentials {
       include!(concat!(env!("OUT_DIR"), "/credentials.rs"));
   }

   let provider = credentials::create_provider();
   ```

### Rebuilding

Credentials are regenerated when:
- `credentials.toml` changes (automatic via `cargo:rerun-if-changed`)
- `cargo clean` is run (forces full rebuild)
- Build script is modified

```bash
# Force regeneration
cargo clean
cargo build --release
```

## Memory Usage

- **Flash:** ~19KB (with authentication, completion, history)
- **RAM:** <2KB static allocation
- **Credentials:** Const data in flash (not RAM)
- **No heap allocation:** Pure `no_std`

## Feature Configuration

```bash
# Minimal (authentication only)
cargo build --release --no-default-features

# With tab completion
cargo build --release --features completion

# With command history
cargo build --release --features history

# All features (default)
cargo build --release --features completion,history
```

## Credentials File Format

```toml
# Type path to your AccessLevel enum
access_level_type = "rp_pico_buildtime::access_level::PicoAccessLevel"

# Define users
[[users]]
username = "admin"
password = "change-me"
level = "Admin"  # Must match AccessLevel variant

[[users]]
username = "user"
password = "user-pass"
level = "User"
```

**Requirements:**
- `access_level_type` must be fully-qualified type path
- `level` must exactly match your AccessLevel enum variants
- Usernames must be unique
- At least one user required

## Production Deployment

**For production systems:**

1. **Secure credential storage:**
   ```bash
   # Never commit credentials.toml
   echo "credentials.toml" >> .gitignore

   # Store in secure location (password manager, secrets vault)
   # Deploy only to build environment
   ```

2. **Strong passwords:**
   ```toml
   [[users]]
   username = "admin"
   password = "jK8#mN2$pQr9@vXz"  # Use strong random passwords
   level = "Admin"
   ```

3. **Minimal user accounts:**
   ```toml
   # Only create necessary accounts
   # Avoid default/demo credentials
   ```

4. **Binary security:**
   - Hashed credentials are visible in binary with reverse engineering
   - Secure physical access to devices
   - Consider additional flash encryption if available
   - Use firmware signing to prevent unauthorized updates

## Comparison with Runtime Credentials

| Aspect | Build-Time | Runtime |
|--------|-----------|---------|
| Hashing | At build time | At device startup |
| Salt | Random (per build) | Fixed or random |
| RAM usage | Zero (const) | Stores hash+salt |
| Flash usage | Slightly higher | Slightly lower |
| Security | Binary contains hashes | Same |
| Flexibility | Rebuild to change | Could load from storage |

**Build-time advantages:**
- Zero RAM overhead for credential storage
- Const-initializable (no runtime setup)
- Simpler code (no hash-at-startup logic)

## Troubleshooting

### Build errors about credgen

```bash
# Ensure credgen dependencies are available
cargo build --features credgen

# Or install credgen separately
cargo install --path ../.. --bin nut-shell-credgen --features credgen
```

### "access_level_type not found"

Check that the type path in `credentials.toml` matches your module structure:
```toml
access_level_type = "rp_pico_buildtime::access_level::PicoAccessLevel"
#                    ^^^^^^^^^^^^^^^^^ must match crate name
```

### Credentials not updating

```bash
# Force rebuild
cargo clean
cargo build --release
```

## License

Same as parent **nut-shell** project (MIT OR Apache-2.0).
