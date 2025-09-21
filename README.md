# ferrofeed

A RSS CLI and TUI for managing, viewing, and exporting
[RSS](https://en.wikipedia.org/wiki/RSS) feeds.

## Usage (Planned API)

```
Usage: ferrofeed [OPTIONS] [COMMAND]

Commands:
  init         Interactive prompt menu for creating and configuring a fresh RSS store
  add-feed     Add a feed to the RSS store
  remove-feed  Remove a feed to the RSS store
  list         List current feeds in the RSS store
  sync         Manually trigger sync across RSS feeds
  export       Export feed(s) as OPML
  tag          Add a tag to feed(s)
  search       Search RSS store content (titles, authors, page content) with grep
  config       Display the current configuration file
  help         Print this message or the help of the given subcommand(s)

Options:
  -c, --config-path <CONFIG_PATH>
  -h, --help                       Print help
```
