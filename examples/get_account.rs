//! Get the current account's details and monthly usage.
//!
//! Run with: `APIFY_TOKEN=... cargo run --example get_account`

use apify_client::ApifyClient;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("APIFY_TOKEN").expect("set APIFY_TOKEN");
    let client = ApifyClient::new(token);

    let user = client.me().get().await?.expect("current user");
    println!("Account id: {}", user.id);
    println!("Username: {}", user.username.as_deref().unwrap_or("(none)"));

    // Monthly usage for the current billing cycle (`None` == current cycle).
    let usage = client.me().monthly_usage().await?;
    if let Some(cycle) = usage.get("usageCycle") {
        println!("Current usage cycle: {cycle}");
    }

    // Usage for the billing cycle that contains a specific `YYYY-MM-DD` date — pass `Some(date)`
    // to look up a particular cycle, or `None` for the current one. We derive the date from the
    // current day (rather than hard-coding one) so the lookup always lands on a real cycle.
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let dated_usage = client.me().monthly_usage_for_date(Some(&date)).await?;
    if let Some(cycle) = dated_usage.get("usageCycle") {
        println!("Usage cycle containing {date}: {cycle}");
    }

    Ok(())
}
