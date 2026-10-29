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

/// Simple GET: the current account's monthly usage for a specific month (the spec's optional
/// `date` query parameter on `GET /v2/users/me/usage/monthly`).
///
/// Asserts the returned billing cycle actually contains the requested date, which proves the
/// `date` query parameter was honored (a dropped param would return the *current* cycle
/// instead, which would not contain a fixed past date). The client unwraps the `{ data: ... }`
/// envelope, so the response is `{ usageCycle: { startAt, endAt }, ... }`. Billing cycles are
/// not calendar-month aligned, so we compare on day (`YYYY-MM-DD`) granularity rather than a
/// month prefix or a full timestamp.
#[tokio::test(flavor = "multi_thread")]
async fn get_monthly_usage_for_date() {
    let client = require_client!();
    // A single fixed past date so the assertion is deterministic regardless of when the test
    // runs. We derive everything from this one literal.
    let requested_day = "2026-03-15"; // YYYY-MM-DD
    let usage = client
        .me()
        .monthly_usage_for_date(Some(requested_day))
        .await
        .expect("get monthly usage for date");

    let cycle = usage
        .get("usageCycle")
        .expect("monthly usage should contain usageCycle");
    let start = cycle
        .get("startAt")
        .and_then(|v| v.as_str())
        .expect("usageCycle.startAt should be a string");
    let end = cycle
        .get("endAt")
        .and_then(|v| v.as_str())
        .expect("usageCycle.endAt should be a string");

    // Compare on calendar-day granularity: the requested day must fall within the cycle's day
    // span [startAt_day, endAt_day]. RFC3339 dates are zero-padded `YYYY-MM-DD`, so the first 10
    // chars sort lexicographically as dates. Using day granularity (not the full timestamp)
    // avoids false failures when a cycle boundary lands on the requested calendar day but at a
    // non-midnight instant (billing cycles are not calendar-aligned and start at the signup
    // time of day). If the `date` param were dropped, the API would return the *current* cycle,
    // which cannot span a fixed 2026-03 day.
    let start_day = start
        .get(..10)
        .expect("usageCycle.startAt should be an RFC3339 date at least 10 chars long");
    let end_day = end
        .get(..10)
        .expect("usageCycle.endAt should be an RFC3339 date at least 10 chars long");
    assert!(
        start_day <= requested_day && requested_day <= end_day,
        "requested day {requested_day} should fall within usage cycle days [{start_day}, {end_day}] \
         (full bounds [{start}, {end}])"
    );
}

/// Simple GET: the current account's limits.
#[tokio::test(flavor = "multi_thread")]
async fn get_limits() {
    let client = require_client!();
    let limits = client.me().limits().await.expect("get limits");
    assert!(limits.is_object(), "limits should be a JSON object");
}

/// Simple GET: the by-id limits and monthly usage endpoints answer for the caller's own account,
/// not only for `me`.
#[tokio::test(flavor = "multi_thread")]
async fn get_limits_and_monthly_usage_by_user_id() {
    let client = require_client!();
    let user = client
        .me()
        .get()
        .await
        .expect("get current user")
        .expect("current user should exist");

    let by_id = client.user(&user.id);
    let limits = by_id.limits().await.expect("get limits by user id");
    assert!(limits.is_object(), "limits should be a JSON object");
    let usage = by_id
        .monthly_usage()
        .await
        .expect("get monthly usage by user id");
    assert!(usage.is_object(), "monthly usage should be a JSON object");
}

/// A resource client with an explicit timeout aborts a request that outlives it. With no retries
/// the first attempt is the only one, and no real request completes within a millisecond.
#[tokio::test(flavor = "multi_thread")]
async fn with_timeout_aborts_a_request_that_outlives_it() {
    let Some(builder) = common::make_client_builder() else {
        eprintln!("Skipping: APIFY_TOKEN is not set");
        return;
    };
    let client = builder.max_retries(0).build();

    let error = client
        .me()
        .with_timeout(apify_client::Timeout::Custom(
            std::time::Duration::from_millis(1),
        ))
        .get()
        .await
        .expect_err("no response arrives within a millisecond");
    assert!(
        matches!(error, apify_client::ApifyClientError::Timeout),
        "{error:?}"
    );
}
