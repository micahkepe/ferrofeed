use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Stdio;
use std::{path::PathBuf, process};
use tokio::{io::AsyncWriteExt, process::Command as TokioCommand};

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
    AddFeed {
        /// The URL of the RSS/Atom resource.
        url: String,
    },
    /// Remove a feed from the RSS store.
    RemoveFeed {
        /// The URL of the RSS/Atom resource.
        url: String,
    },
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
    /// Search RSS store content (titles, authors, page content) with ripgrep.
    Search {
        /// Pattern to match
        query: String,
    },
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
            let conf = match toml::to_string_pretty(&cfg) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1)
                }
            };

            // Pipe to `less`; (basically: `cat config.toml | less`)
            let mut child = TokioCommand::new("less")
                .stdin(Stdio::piped())
                .spawn()
                .expect("failed to spawn less");
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(conf.as_bytes()).await?;
            }
            // Wait for less to exit
            child.wait().await?;

            Ok(())
        }
        Some(_) => {
            // Handle remaining subcommands
            println!("Command not yet implemented");
            Ok(())
        }
        None => {
            // Open TUI
            ui::init(&db)
        }
    }
}
