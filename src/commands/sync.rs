//! Sync feeds command implementation.

use anyhow::{Context, Result};

use crate::{client, db::Db};

/// Sync all feeds - fetch new items for all feeds in the database.
pub async fn sync_feeds(db: &Db) -> Result<()> {
    let feeds = db.list_feeds().context("failed to list feeds")?;

    if feeds.is_empty() {
        println!("No feeds to sync. Add one with: ferrofeed add-feed <url>");
        return Ok(());
    }

    println!("Syncing {} feeds...", feeds.len());
    println!();

    let mut total_new_items = 0;

    for feed in feeds {
        print!("{} ... ", feed.title.as_deref().unwrap_or(&feed.url));

        match client::fetch_feed(&feed.url).await {
            Ok(parsed_feed) => {
                let mut new_items = 0;

                for item in parsed_feed.items {
                    // Convert Vec<String> to Vec<&str> for add_feed_item
                    let authors_refs: Vec<&str> = item.authors.iter().map(|s| s.as_str()).collect();

                    // add_feed_item returns true if inserted, false if duplicate
                    match db.add_feed_item(
                        feed.id,
                        item.title.as_deref(),
                        item.link.as_deref(),
                        item.description.as_deref(),
                        Some(&authors_refs),
                        item.published,
                    ) {
                        Ok(true) => new_items += 1,
                        Ok(false) => {
                            // Duplicate, skip silently
                        }
                        Err(e) => {
                            // Log error but continue with other items
                            eprintln!("Warning: failed to add item: {}", e);
                        }
                    }
                }

                total_new_items += new_items;
                println!("({} new items)", new_items);
            }
            Err(e) => {
                println!("failed: {}", e);
            }
        }
    }

    println!();
    println!("Sync complete. {} new items added.", total_new_items);

    Ok(())
}
