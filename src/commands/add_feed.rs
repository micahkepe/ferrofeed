//! Add feed command implementation.

use anyhow::{Context, Result};

use crate::{client, db::Db};

/// Add a feed to the database. Fetches the feed to validate and extract metadata.
pub async fn add_feed(db: &Db, url: &str) -> Result<()> {
    println!("Fetching feed from {}...", url);

    // Fetch and parse the feed to validate it
    let parsed_feed = client::fetch_feed(url)
        .await
        .with_context(|| format!("failed to fetch and parse feed from {}", url))?;

    // Add feed to database
    db.add_feed(url, parsed_feed.title.as_deref())
        .context("failed to add feed to database")?;

    println!(
        "Added feed: {}",
        parsed_feed.title.as_deref().unwrap_or(url)
    );

    println!("Found {} items", parsed_feed.items.len());

    Ok(())
}
