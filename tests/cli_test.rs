//! Integration tests for CLI commands.

use assert_cmd::Command;

use ferrofeed::{commands, db::Db};

/// Create a test database. ":memory:" is used for in-memory database that is volatile and
/// will be deleted when the process exits.
///   See: <https://www.sqlite.org/inmemorydb.html>
fn create_test_db() -> Db {
    let db = Db::open(":memory:").expect("failed to create test db");
    db.init_feed_table().expect("failed to init feed table");
    db.init_feed_item_table()
        .expect("failed to init feed_item table");
    db
}

/// Helper function to run the `query` binary with the given arguments and return a
/// [`assert_cmd::assert::Assert`].
fn run_ferrofeed_command(args: &[&str]) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("ferrofeed").expect("Failed to find binary");
    cmd.args(args);
    cmd.assert()
}

#[test]
fn test_run_help() {
    let help_args_options = vec!["-h", "--help"];
    for arg in help_args_options {
        run_ferrofeed_command(&[arg]).success();
    }
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
