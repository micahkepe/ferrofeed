use anyhow::Result;
use rusqlite::{Connection, params};
use std::fs;
use time::OffsetDateTime;

/// Represents a feed entry in the `feed` table.
#[derive(Debug, Clone)]
pub struct Feed {
    /// Unique identifier primary key.
    pub id: usize,
    /// The URL of the feed.
    pub url: String,
    /// Optional title for the feed.
    pub title: Option<String>,
    /// Creation time (Unix timestamp)
    pub created_at: i64,
}

/// Represents a feed item (post/article) in the `feed_item` table.
#[derive(Debug, Clone)]
pub struct FeedItem {
    /// Unique identifier primary key.
    pub id: usize,
    /// Foreign key to the feed this item belongs to.
    pub feed_id: usize,
    /// The title of the item/post.
    pub title: Option<String>,
    /// The link/URL to the full content.
    pub link: Option<String>,
    /// Description or summary of the item.
    pub description: Option<String>,
    /// The author of the item.
    pub author: Option<String>,
    /// Published date (Unix timestamp).
    pub published: Option<i64>,
    /// Whether the item has been read.
    pub is_read: bool,
    /// When this item was added to the database (Unix timestamp).
    pub created_at: i64,
}

/// The database object.
#[derive(Debug)]
pub struct Db {
    /// A connection to the SQLITE database.
    conn: Connection,
}

impl Db {
    /// Open a new connection to a SQLite database. If a database does not exist
    /// at the path, one is created.
    pub fn open(path: &str) -> Result<Self> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            fs::create_dir_all(parent)?
        }
        let conn = Connection::open(path)?;
        // Enable write-ahead logging and foreign key checking
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn })
    }

    /// Initialize the `feed` table for holding the user's RSS/Atom feeds.
    pub fn init_feed_table(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS feed (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT,
                created_at INTEGER NOT NULL
            )
            "#,
        )?;
        Ok(())
    }

    /// Initialize the `feed_item` table for holding individual posts/articles from feeds.
    pub fn init_feed_item_table(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS feed_item (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feed_id INTEGER NOT NULL,
                title TEXT,
                link TEXT,
                description TEXT,
                author TEXT,
                published INTEGER,
                is_read INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (feed_id) REFERENCES feed(id) ON DELETE CASCADE,
                UNIQUE(feed_id, link)
            )
            "#,
        )?;
        Ok(())
    }

    /// Add a feed specified by URL and optional title the to database.
    pub fn add_feed(&self, url: &str, title: Option<&str>) -> Result<()> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.conn.execute(
            "INSERT INTO feed (url, title, created_at) VALUES (?1, ?2, ?3)",
            params![url, title, now],
        )?;
        Ok(())
    }

    /// Remove a feed by URL. Returns true if a feed was deleted, false if not found.
    pub fn remove_feed(&self, url: &str) -> Result<bool> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM feed WHERE url = ?1", params![url])?;
        Ok(rows_affected > 0)
    }

    /// List the feeds in the database.
    pub fn list_feeds(&self) -> Result<Vec<Feed>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, url, title, created_at FROM feed")?;
        let rows = stmt.query_map([], |row| {
            Ok(Feed {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut feeds = Vec::new();
        for f in rows {
            feeds.push(f?);
        }
        Ok(feeds)
    }

    /// Add a feed item to the database. Uses INSERT OR IGNORE to skip duplicates.
    /// Returns true if the item was inserted, false if it was a duplicate.
    pub fn add_feed_item(
        &self,
        feed_id: usize,
        title: Option<&str>,
        link: Option<&str>,
        description: Option<&str>,
        author: Option<&str>,
        published: Option<i64>,
    ) -> Result<bool> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let rows_affected = self.conn.execute(
            "INSERT OR IGNORE INTO feed_item (feed_id, title, link, description, author, published, is_read, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
            params![feed_id, title, link, description, author, published, now],
        )?;
        Ok(rows_affected > 0)
    }

    /// Get all items for a specific feed.
    pub fn get_feed_items(&self, feed_id: usize) -> Result<Vec<FeedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, feed_id, title, link, description, author, published, is_read, created_at
             FROM feed_item
             WHERE feed_id = ?1
             ORDER BY published DESC",
        )?;
        let rows = stmt.query_map(params![feed_id], |row| {
            Ok(FeedItem {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                title: row.get(2)?,
                link: row.get(3)?,
                description: row.get(4)?,
                author: row.get(5)?,
                published: row.get(6)?,
                is_read: row.get::<_, i64>(7)? != 0,
                created_at: row.get(8)?,
            })
        })?;
        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    /// Mark a feed item as read.
    pub fn mark_item_read(&self, item_id: usize) -> Result<()> {
        self.conn.execute(
            "UPDATE feed_item SET is_read = 1 WHERE id = ?1",
            params![item_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Db {
        let db = Db::open(":memory:").expect("failed to create test db");
        db.init_feed_table().expect("failed to init feed table");
        db.init_feed_item_table()
            .expect("failed to init feed_item table");
        db
    }

    #[test]
    fn test_add_and_list_feeds() {
        let db = create_test_db();

        // Add a feed
        db.add_feed("https://example.com/feed.xml", Some("Test Feed"))
            .expect("failed to add feed");

        // List feeds
        let feeds = db.list_feeds().expect("failed to list feeds");
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].url, "https://example.com/feed.xml");
        assert_eq!(feeds[0].title, Some("Test Feed".to_string()));
    }

    #[test]
    fn test_add_duplicate_feed_fails() {
        let db = create_test_db();

        db.add_feed("https://example.com/feed.xml", Some("Test Feed"))
            .expect("failed to add feed");

        // Try to add duplicate
        let result = db.add_feed("https://example.com/feed.xml", Some("Duplicate"));
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_feed() {
        let db = create_test_db();

        db.add_feed("https://example.com/feed.xml", Some("Test Feed"))
            .expect("failed to add feed");

        // Remove the feed
        let deleted = db
            .remove_feed("https://example.com/feed.xml")
            .expect("failed to remove feed");
        assert!(deleted);

        // Verify it's gone
        let feeds = db.list_feeds().expect("failed to list feeds");
        assert_eq!(feeds.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_feed() {
        let db = create_test_db();

        let deleted = db
            .remove_feed("https://nonexistent.com/feed.xml")
            .expect("failed to remove feed");
        assert!(!deleted);
    }

    #[test]
    fn test_add_feed_items() {
        let db = create_test_db();

        db.add_feed("https://example.com/feed.xml", Some("Test Feed"))
            .expect("failed to add feed");

        let feeds = db.list_feeds().expect("failed to list feeds");
        let feed_id = feeds[0].id;

        // Add an item
        let inserted = db
            .add_feed_item(
                feed_id,
                Some("Test Item"),
                Some("https://example.com/item1"),
                Some("Item description"),
                Some("Author"),
                Some(1234567890),
            )
            .expect("failed to add item");
        assert!(inserted);

        // Get items for feed
        let items = db.get_feed_items(feed_id).expect("failed to get items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, Some("Test Item".to_string()));
        assert_eq!(items[0].link, Some("https://example.com/item1".to_string()));
    }

    #[test]
    fn test_add_duplicate_feed_item() {
        let db = create_test_db();

        db.add_feed("https://example.com/feed.xml", Some("Test Feed"))
            .expect("failed to add feed");

        let feeds = db.list_feeds().expect("failed to list feeds");
        let feed_id = feeds[0].id;

        // Add an item
        let inserted = db
            .add_feed_item(
                feed_id,
                Some("Test Item"),
                Some("https://example.com/item1"),
                Some("Description"),
                Some("Author"),
                Some(1234567890),
            )
            .expect("failed to add item");
        assert!(inserted);

        // Try to add duplicate (same feed_id and link)
        let inserted = db
            .add_feed_item(
                feed_id,
                Some("Different Title"),
                Some("https://example.com/item1"),
                Some("Different Description"),
                Some("Author"),
                Some(1234567890),
            )
            .expect("failed to add duplicate item");
        assert!(!inserted);

        // Verify only one item exists
        let items = db.get_feed_items(feed_id).expect("failed to get items");
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_cascade_delete_feed_items() {
        let db = create_test_db();

        db.add_feed("https://example.com/feed.xml", Some("Test Feed"))
            .expect("failed to add feed");

        let feeds = db.list_feeds().expect("failed to list feeds");
        let feed_id = feeds[0].id;

        // Add items
        db.add_feed_item(
            feed_id,
            Some("Item 1"),
            Some("https://example.com/item1"),
            None,
            None,
            None,
        )
        .expect("failed to add item");

        db.add_feed_item(
            feed_id,
            Some("Item 2"),
            Some("https://example.com/item2"),
            None,
            None,
            None,
        )
        .expect("failed to add item");

        // Verify items exist
        let items = db.get_feed_items(feed_id).expect("failed to get items");
        assert_eq!(items.len(), 2);

        // Remove the feed
        db.remove_feed("https://example.com/feed.xml")
            .expect("failed to remove feed");

        // Verify items are also deleted (can't check directly without feed_id, but feeds should be empty)
        let feeds = db.list_feeds().expect("failed to list feeds");
        assert_eq!(feeds.len(), 0);
    }

    #[test]
    fn test_mark_item_read() {
        let db = create_test_db();

        db.add_feed("https://example.com/feed.xml", Some("Test Feed"))
            .expect("failed to add feed");

        let feeds = db.list_feeds().expect("failed to list feeds");
        let feed_id = feeds[0].id;

        db.add_feed_item(
            feed_id,
            Some("Test Item"),
            Some("https://example.com/item1"),
            None,
            None,
            None,
        )
        .expect("failed to add item");

        let items = db.get_feed_items(feed_id).expect("failed to get items");
        assert!(!items[0].is_read);

        // Mark as read
        db.mark_item_read(items[0].id)
            .expect("failed to mark item as read");

        // Verify it's marked as read
        let items = db.get_feed_items(feed_id).expect("failed to get items");
        assert!(items[0].is_read);
    }
}
