//! Remove feed command implementation.

use anyhow::{Context, Result};

use crate::db::Db;

/// Remove a feed from the database.
pub fn remove_feed(db: &Db, url: &str) -> Result<()> {
    let deleted = db
        .remove_feed(url)
        .context("failed to remove feed from database")?;

    if deleted {
        println!("Removed feed: {}", url);
    } else {
        println!("Feed not found: {}", url);
    }

    Ok(())
}
