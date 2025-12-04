# Raspberry Pi Pico - Build-Time Credentials Example

Demonstrates build-time credential generation using `nut-shell-credgen` on RP2040 hardware.

## Build Flow

```
credentials.toml   →   nut-shell-credgen   →   credentials.rs   →   final binary
(plaintext)            (build tool)            (hashed)            (compiled)
```

1. `build.rs` runs `nut-shell-credgen` during compilation
2. Reads `credentials.toml` with plaintext passwords
3. Generates `credentials.rs` with hashed credentials and random salts
4. `include!` macro pulls generated code into binary
5. Final binary contains only hashed credentials

**Security:** Only hashed credentials in firmware. Plaintext passwords stay in `credentials.toml` (gitignored).

## Quick Start

**Default credentials:**
- `admin:admin123`
- `user:user123`

**Build and run:**
```bash
cargo run --release
```

**Customize credentials:**
```bash
# Edit credentials.toml, then rebuild
vim credentials.toml
cargo clean && cargo build --release
```

## Implementation

**credentials.toml:**
```toml
access_level_type = "rp_pico_buildtime::access_level::PicoAccessLevel"

[[users]]
username = "admin"
password = "secret"
level = "Admin"
```

**build.rs:**
```rust
fn main() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "nut-shell-credgen", "--features", "credgen",
               "--", "credentials.toml"])
        .output().unwrap();

    std::fs::write(
        format!("{}/credentials.rs", std::env::var("OUT_DIR").unwrap()),
        output.stdout
    ).unwrap();
}
```

**main.rs:**
```rust
mod credentials {
    include!(concat!(env!("OUT_DIR"), "/credentials.rs"));
}

let provider = credentials::create_provider();
let shell = Shell::new(&ROOT, handler, &provider, io);
```

## Production Notes

- Add `credentials.toml` to `.gitignore`
- Use strong passwords (not demo credentials)
- Hashed credentials visible in binary (secure physical access)
- Credentials regenerated when `credentials.toml` changes or after `cargo clean`

## License

Same as parent **nut-shell** project (MIT OR Apache-2.0).
