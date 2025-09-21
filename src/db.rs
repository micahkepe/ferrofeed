use anyhow::Result;
use rusqlite::{Connection, params};
use time::OffsetDateTime;

const DEFAULT_DB_NAME: &str = "ferrofeed.db";

/// Represents a feed entry in the `feed` table.
#[derive(Debug)]
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

    pub fn add_feed(&self, url: &str, title: Option<&str>) -> Result<()> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.conn.execute(
            "INSERT INTO feed (url, title, created_at) VALUES (?1, ?2, ?3)",
            params![url, title, now],
        )?;
        Ok(())
    }

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
}
