//! Integration tests for CLI commands.

use ferrofeed::{commands, db::Db};

fn create_test_db() -> Db {
    let db = Db::open(":memory:").expect("failed to create test db");
    db.init_feed_table().expect("failed to init feed table");
    db.init_feed_item_table()
        .expect("failed to init feed_item table");
    db
}

#[test]
fn test_list_feeds_empty() {
    let db = create_test_db();
    let result = commands::list_feeds(&db);
    assert!(result.is_ok());
}

#[test]
fn test_list_feeds_with_data() {
    let db = create_test_db();
    db.add_feed("https://example.com/feed.xml", Some("Test Feed"))
        .expect("failed to add feed");

    let result = commands::list_feeds(&db);
    assert!(result.is_ok());
}

#[test]
fn test_remove_feed_success() {
    let db = create_test_db();
    db.add_feed("https://example.com/feed.xml", Some("Test Feed"))
        .expect("failed to add feed");

    let result = commands::remove_feed(&db, "https://example.com/feed.xml");
    assert!(result.is_ok());

    let feeds = db.list_feeds().expect("failed to list feeds");
    assert_eq!(feeds.len(), 0);
}

#[test]
fn test_remove_feed_not_found() {
    let db = create_test_db();
    let result = commands::remove_feed(&db, "https://nonexistent.com/feed.xml");
    assert!(result.is_ok());
}

// Note: add_feed and sync_feeds tests would require mocking HTTP requests
// or using a test server, which is more complex. For now, we test the
// database operations they rely on.
