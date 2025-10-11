//! Simple test to verify feed fetching works.

use ferrofeed::client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Test with a well-known RSS feed
    let test_url = "https://blog.rust-lang.org/feed.xml";

    println!("Fetching feed from: {}", test_url);

    let feed = client::fetch_feed(test_url).await?;

    println!("\nFeed Title: {:?}", feed.title);
    println!("Number of items: {}", feed.items.len());

    if !feed.items.is_empty() {
        println!("\nFirst item:");
        let item = &feed.items[0];
        println!("  Title: {:?}", item.title);
        println!("  Link: {:?}", item.link);
        println!("  Author: {:?}", item.author);
        println!("  Published: {:?}", item.published);
    }

    Ok(())
}
