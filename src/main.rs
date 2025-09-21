use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use ferrofeed::{config, db, ui};

#[derive(Parser)]
struct Args {
    /// Run with a specified configuration file.
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

    // Parse user config, if it exists
    let cfg = config::Config::load(args.config_path)?;

    match args.command {
        Some(Command::Init {}) => {
            let db = db::Db::open(
                cfg.database_path
                    .to_str()
                    .expect("No database path specified"),
            )?;
            db.init_feed_table()?;
            Ok(())
        }
        Some(Command::Config {}) => {
            println!("{:?}", cfg);
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
