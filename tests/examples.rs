//! Smoke tests that run the documentation examples end-to-end.
//!
//! Each example in `examples/` runs as a subprocess with `cargo run --example` against the live
//! API. The Actor-build example gets a longer time budget.

mod common;

use std::process::Command;

/// Runs `cargo run --example <name>` and asserts it exits successfully.
fn run_example(name: &str) {
    if std::env::var("APIFY_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
        .is_none()
    {
        eprintln!("Skipping example `{name}`: APIFY_TOKEN is not set");
        return;
    }

    let status = Command::new(env!("CARGO"))
        .args(["run", "--quiet", "--example", name])
        .status()
        .expect("failed to spawn cargo run");

    assert!(
        status.success(),
        "example `{name}` exited with failure: {status}"
    );
}

#[test]
fn example_get_account() {
    run_example("get_account");
}

#[test]
fn example_storages() {
    run_example("storages");
}

#[test]
fn example_iterate_store() {
    run_example("iterate_store");
}

#[test]
fn example_run_store_actor() {
    run_example("run_store_actor");
}

#[test]
fn example_run_and_last_run_storages() {
    run_example("run_and_last_run_storages");
}

#[test]
fn example_log_redirection() {
    run_example("log_redirection");
}

#[test]
fn example_create_build_run_actor() {
    run_example("create_build_run_actor");
}

#[test]
fn example_raw_log() {
    run_example("raw_log");
}
