# ferrofeed

> [!NOTE]
> Under development, stay tuned.

A RSS CLI and TUI for managing, viewing, and exporting
[RSS](https://en.wikipedia.org/wiki/RSS) feeds.

## Usage

```text
A RSS CLI and TUI for managing, viewing, and exporting RSS/Atom feeds.


Usage: ferrofeed [OPTIONS] [COMMAND]

Commands:
  add-feed     Add a feed to the RSS store
  remove-feed  Remove a feed from the RSS store
  list         List current feeds in the RSS store
  sync         Manually trigger sync across RSS feeds
  export       Export feed(s) as OPML
  tag          Add a tag to feed(s)
  search       Search RSS store content (titles, authors, page content) with ripgrep
  config       Display the current configuration file
  schedule     Schedule sync command to run on a schedule
  help         Print this message or the help of the given subcommand(s)

Options:
  -c, --config-path <CONFIG_PATH>  Run with a specified configuration file
  -h, --help                       Print help
  -V, --version                    Print version
```

## TODOs

- [x] Support `schedule` subcommand for running background sync cronjob
  - [x] Configurable schedule (default to every 60 minutes)
  - [ ] Respect user's config
- [ ] Search over posts via `ripgrep` library/ SQL search
  - [ ] '/' key to search
- [ ] Rich HTML text display with [`html2text`](https://crates.io/crates/html2text)
- [ ] Support "go to external" mapping (similar to `gx` in Vim) when in post to
      go to original
- [ ] Fetch whole post contents (not just until `<-- more! -->` tag)
- [ ] OPML import/export support
  - [OPML specification](https://opml.org/spec2.opml)
- [ ] Sync feeds key binding (default to `s`)
- [ ] Logging
- [ ] Read/unread indicators
