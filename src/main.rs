use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use ferrofeed::{db, ui};

#[derive(Parser)]
struct Args {
    #[clap(short = 'c', long)]
    config_path: Option<PathBuf>,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Interactive prompt menu for creating and configuring a fresh RSS store.
    Init {},
    /// Add a feed to the RSS store.
    AddFeed { url: String },
    /// Remove a feed to the RSS store.
    RemoveFeed {},
    /// List current feeds in the RSS store.
    List,
    /// Manually trigger sync across RSS feeds.
    Sync,
    /// Export feed(s) as OPML.
    Export { feed: Option<Vec<String>> },
    /// Add a tag to feed(s).
    Tag { feeds: Vec<String>, tag: String },
    /// Search RSS store content (titles, authors, page content) with grep.
    Search { query: String },
    /// Display the current configuration file.
    Config {},
}

/// Main entry point.
fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::Init {}) => {
            // TODO: replace with parsed config value
            let db = db::Db::open("ferrofeed.db")?;
            db.init_feed_table()?;
            Ok(())
        }
        Some(_) => {
            // Handle subcommands
            todo!()
        }
        None => {
            // Open TUI
            ui::init()
        }
    }
}
