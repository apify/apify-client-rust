//! Get the current account's details.
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

    Ok(())
}
