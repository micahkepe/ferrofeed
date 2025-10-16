//! Business logic for CLI commands.

mod add_feed;
mod list;
mod remove_feed;
mod schedule;
mod sync;

pub use add_feed::add_feed;
pub use list::list_feeds;
pub use remove_feed::remove_feed;
pub use schedule::schedule;
pub use sync::sync_feeds;
