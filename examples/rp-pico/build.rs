//! Build script for rp-pico-buildtime example
//!
//! Runs nut-shell-credgen to generate credentials.rs from credentials.toml

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Rebuild if credentials.toml changes
    println!("cargo:rerun-if-changed=credentials.toml");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    // Run nut-shell-credgen to generate credentials code
    // Important: Build for host target, not embedded target
    let host = env::var("HOST").unwrap_or_else(|_| "x86_64-unknown-linux-gnu".into());

    // Get paths
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let credentials_path = manifest_dir.join("credentials.toml");
    let nut_shell_dir = manifest_dir.join("../..");

    // Step 1: Build credgen binary for host
    // We need to clear RUSTFLAGS to avoid embedded linker flags
    let build_status = Command::new("cargo")
        .current_dir(&nut_shell_dir)
        .env_remove("CARGO_ENCODED_RUSTFLAGS")  // Clear any rustflags
        .env("RUSTFLAGS", "")  // Set empty rustflags
        .args([
            "build",
            "--bin",
            "nut-shell-credgen",
            "--features",
            "credgen",
            "--target",
            &host,
        ])
        .status()
        .expect("Failed to build nut-shell-credgen");

    if !build_status.success() {
        eprintln!("Failed to build nut-shell-credgen");
        panic!("Credential generation failed");
    }

    // Step 2: Run the built binary
    let credgen_bin = nut_shell_dir
        .join("target")
        .join(&host)
        .join("debug")
        .join("nut-shell-credgen");

    let output = Command::new(&credgen_bin)
        .arg(&credentials_path)
        .output()
        .expect("Failed to run nut-shell-credgen");

    if !output.status.success() {
        eprintln!("nut-shell-credgen failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("Credential generation failed");
    }

    // Write generated code to OUT_DIR
    std::fs::write(out_dir.join("credentials.rs"), output.stdout)
        .expect("Failed to write credentials.rs");

    println!("cargo:warning=Generated credentials.rs in OUT_DIR");
}
