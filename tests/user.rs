//! Integration tests for the user resource.

mod common;

/// Simple GET: fetching the current account's details.
#[tokio::test(flavor = "multi_thread")]
async fn get_own_account() {
    let client = require_client!();
    let user = client
        .me()
        .get()
        .await
        .expect("get current user")
        .expect("current user should exist");
    assert!(!user.id.is_empty(), "user id should be present");
}

/// Simple GET: the current account's monthly usage.
#[tokio::test(flavor = "multi_thread")]
async fn get_monthly_usage() {
    let client = require_client!();
    let usage = client
        .me()
        .monthly_usage()
        .await
        .expect("get monthly usage");
    assert!(usage.is_object(), "monthly usage should be a JSON object");
}

/// Simple GET: the current account's limits.
#[tokio::test(flavor = "multi_thread")]
async fn get_limits() {
    let client = require_client!();
    let limits = client.me().limits().await.expect("get limits");
    assert!(limits.is_object(), "limits should be a JSON object");
}
