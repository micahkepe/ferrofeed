# ferrofeed

> [!NOTE]
> Under development, stay tuned.

A RSS CLI and TUI for managing, viewing, and exporting
[RSS](https://en.wikipedia.org/wiki/RSS) feeds.

## Usage

```text
Usage: ferrofeed [OPTIONS] [COMMAND]

Commands:
  add-feed     Add a feed to the RSS store
  remove-feed  Remove a feed to the RSS store
  list         List current feeds in the RSS store
  sync         Manually trigger sync across RSS feeds
  export       Export feed(s) as OPML
  tag          Add a tag to feed(s)
  search       Search RSS store content (titles, authors, page content) with ripgrep
  config       Display the current configuration file
  help         Print this message or the help of the given subcommand(s)

Options:
  -c, --config-path <CONFIG_PATH>  Run with a specified configuration file
  -h, --help                       Print help
```

## TODOs

- [ ] Rich HTML text display with [`html2text`](https://crates.io/crates/html2text)
- [ ] Support `schedule` subcommand for running background sync cronjob
  - [ ] Configurable schedule (default to 60 minute)
- [ ] Support "go to external" mapping (similar to `gx` in Vim) when in post to
      go to original
- [ ] Search over posts via `ripgrep`
- [ ] OPML import/export support
  - [OPML specification](https://opml.org/spec2.opml)
