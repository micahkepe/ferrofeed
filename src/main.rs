/*
 * Main entry point for ferrofeed.
 */
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::{io::AsyncWriteExt, process::Command as TokioCommand};

use ferrofeed::{commands, config, db, ui};

/// A RSS CLI and TUI for managing, viewing, and exporting RSS/Atom feeds.
#[derive(Parser)]
#[command(version, about, long_about = None)]
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
    /// Search RSS store content (titles, authors, page content)
    Search {
        /// Pattern to match
        query: String,
    },
    /// Display the current configuration file.
    Config,
    /// Schedule sync command to run on a schedule.
    Schedule {
        /// Minutes to run sync command, valid range is 1..=1440 (24 hours). Default is 60 minutes,
        /// or once per hour.
        #[clap(short = 'm', long, default_value_t = 60, value_name = "MINUTES")]
        minutes: u32,
    },
}

/// Main entry point.
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Parse user config, if it exists
    let cfg = config::Config::load(args.config_path)?;

    // Load/create database and associated tables
    let db = db::Db::open(
        cfg.database_path
            .as_ref()
            .expect("no database name or default")
            .to_str()
            .expect("no database path specified"),
    )?;
    db.init_feed_table()?;
    db.init_feed_item_table()?;

    match args.command {
        Some(Command::AddFeed { url }) => commands::add_feed(&db, &url).await,
        Some(Command::RemoveFeed { url }) => commands::remove_feed(&db, &url).await,
        Some(Command::List) => commands::list_feeds(&db),
        Some(Command::Sync) => commands::sync_feeds(&db).await,
        Some(Command::Config) => {
            let conf = match toml::to_string_pretty(&cfg) {
                Ok(s) => s,
                Err(e) => return Err(anyhow::anyhow!(e)),
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
        Some(Command::Schedule { minutes }) => Ok(commands::schedule(minutes).await?),
        Some(_) => {
            // TODO: Handle remaining subcommands
            unimplemented!()
        }
        None => {
            // Open TUI
            ui::init(&db)
        }
    }
}
