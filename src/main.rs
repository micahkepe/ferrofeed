use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use ferrofeed::{commands, config, db, ui};

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
    /// Add a feed to the RSS store.
    AddFeed { url: String },
    /// Remove a feed from the RSS store.
    RemoveFeed { url: String },
    /// List current feeds in the RSS store.
    List,
    /// Manually trigger sync across RSS feeds.
    Sync,
    /// Export feed(s) as OPML.
    Export { feed: Option<Vec<String>> },
    /// Add a tag to feed(s).
    Tag {
        /// The name of the tag to add.
        #[clap(long)]
        name: String,
        /// The feed(s) to add the tag to.
        feeds: Vec<String>,
    },
    /// Search RSS store content (titles, authors, page content) with grep.
    Search { query: String },
    /// Display the current configuration file.
    Config,
}

/// Main entry point.
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Parse user config, if it exists
    let cfg = config::Config::load(args.config_path.clone())?;

    // Load/create database and associated tables
    let db = db::Db::open(
        cfg.database_path
            .to_str()
            .expect("no database path specified"),
    )?;
    db.init_feed_table()?;
    db.init_feed_item_table()?;

    match args.command {
        Some(Command::AddFeed { url }) => commands::add_feed(&db, &url).await,
        Some(Command::RemoveFeed { url }) => commands::remove_feed(&db, &url),
        Some(Command::List) => commands::list_feeds(&db),
        Some(Command::Sync) => commands::sync_feeds(&db).await,
        Some(Command::Config) => {
            match toml::to_string_pretty(&cfg) {
                Ok(s) => println!("{}", s),
                Err(e) => eprintln!("{}", e),
            }
            Ok(())
        }
        Some(_) => {
            // Handle remaining subcommands
            println!("Command not yet implemented");
            Ok(())
        }
        None => {
            // Open TUI
            ui::init()
        }
    }
}
