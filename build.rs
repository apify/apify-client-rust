//! Build script that captures the Rust compiler version at build time.
//!
//! Rust has no stable runtime API for querying its own version, so the closest analogue to a
//! "language/runtime version" for the `User-Agent` header is the compiler that built the crate.
//! We run `rustc --version` here and expose the semver (e.g. `1.94.1`) as the `BUILD_RUSTC_VERSION`
//! compile-time env var, read via `env!`/`option_env!` in `src/common.rs`.

use std::process::Command;

fn main() {
    let version = rustc_semver().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_RUSTC_VERSION={version}");
    // Re-run if the toolchain changes.
    println!("cargo:rerun-if-env-changed=RUSTC");
}

/// Runs `rustc --version` and extracts the semver token (e.g. `1.94.1` from
/// `rustc 1.94.1 (abc 2026-01-01)`). Returns `None` if rustc cannot be invoked or parsed.
fn rustc_semver() -> Option<String> {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = Command::new(rustc).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    // Expected format: "rustc <semver> (<hash> <date>)". Take the second whitespace token.
    stdout.split_whitespace().nth(1).map(|s| s.to_string())
}
