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

    let output = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            "../../Cargo.toml",  // Path to nut-shell root
            "--bin",
            "nut-shell-credgen",
            "--features",
            "credgen",
            "--target",
            &host,
            "--",
            "credentials.toml",
        ])
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
