//! Get the current account's details and monthly usage.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example get_account`

use apify_client::ApifyClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    let user = client.me().get().await?.expect("current user");
    println!("Account id: {}", user.id);
    println!("Username: {:?}", user.username);

    // Monthly usage for the current billing cycle (`None` == current cycle).
    let usage = client.me().monthly_usage().await?;
    if let Some(cycle) = usage.get("usageCycle") {
        println!("Current usage cycle: {cycle}");
    }

    // Usage for the billing cycle containing a specific `YYYY-MM-DD` date, via the spec's
    // optional `date` query parameter on `GET /v2/users/me/usage/monthly`.
    let past_usage = client
        .me()
        .monthly_usage_for_date(Some("2026-03-15"))
        .await?;
    if let Some(cycle) = past_usage.get("usageCycle") {
        println!("Usage cycle containing 2026-03-15: {cycle}");
    }

    Ok(())
}
