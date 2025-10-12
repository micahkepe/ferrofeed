//! List feeds command implementation.

use anyhow::{Context, Result};

use crate::db::Db;

/// List all feeds in the database.
pub fn list_feeds(db: &Db) -> Result<()> {
    let feeds = db.list_feeds().context("failed to list feeds")?;

    if feeds.is_empty() {
        println!("No feeds found. Add one with: ferrofeed add-feed <url>");
        return Ok(());
    }

    println!("Feeds ({})", feeds.len());
    println!();

    for feed in feeds {
        println!(
            "  [{}] {}",
            feed.id,
            feed.title.as_deref().unwrap_or("(no title)")
        );
        println!("      URL: {}", feed.url);
        println!();
    }

    Ok(())
}
