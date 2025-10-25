//! HTTP client for fetching and parsing RSS/Atom feeds.
//!
//! TODO: Fetch content past the first `<!-- more -->` tag
//!
//! TODO: Possibly restructure data model to use/interface with `feed-rs` crate directly

use anyhow::{Context, Result};
use feed_rs::parser;

/// Parsed feed data containing metadata and items.
#[derive(Debug)]
pub struct ParsedFeed {
    /// The title of the feed.
    pub title: Option<String>,
    /// The feed items/entries.
    pub items: Vec<ParsedFeedItem>,
}

/// A single item from a feed.
#[derive(Debug)]
pub struct ParsedFeedItem {
    /// The title of the item.
    pub title: Option<String>,
    /// The link to the full content.
    pub link: Option<String>,
    /// The description/summary.
    pub description: Option<String>,
    /// The author(s).
    pub authors: Vec<String>,
    /// Published date as Unix timestamp.
    pub published: Option<i64>,
}

/// Fetch and parse an RSS/Atom feed from a URL.
pub async fn fetch_feed(url: &str) -> Result<ParsedFeed> {
    // Fetch the feed content
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("failed to fetch feed from {}", url))?;

    let content = response
        .bytes()
        .await
        .context("failed to read response body")?;

    // Parse the feed
    let feed = parser::parse(&content[..])
        .with_context(|| format!("failed to parse feed from {}", url))?;

    let title = feed.title.map(|t| t.content);
    let items = feed
        .entries
        .into_iter()
        .map(|entry| {
            // Get the first link if available
            let link = entry.links.first().map(|l| l.href.clone());

            // Get description from summary or content
            let description = entry.summary.map(|s| s.content).or_else(|| {
                entry
                    .content
                    .and_then(|c| c.body.or_else(|| c.src.map(|s| s.href)))
            });

            // Get author name
            let authors = entry
                .authors
                .iter()
                .map(|a| a.name.trim().to_string())
                .collect();

            // Get published date as Unix timestamp
            let published = entry.published.or(entry.updated).map(|dt| dt.timestamp());

            ParsedFeedItem {
                title: entry.title.map(|t| t.content),
                link,
                description,
                authors,
                published,
            }
        })
        .collect();

    Ok(ParsedFeed { title, items })
}
