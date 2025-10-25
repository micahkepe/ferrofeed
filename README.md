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
  search       Search RSS store content (titles, authors, page content)
  config       Display the current configuration file
  schedule     Schedule sync command to run on a schedule
  help         Print this message or the help of the given subcommand(s)

Options:
  -c, --config-path <CONFIG_PATH>  Run with a specified configuration file
  -h, --help                       Print help
  -V, --version                    Print version
```

## License

This repository is licensed under an MIT License. See [LICENSE](./LICENSE) for
more details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for
more details.
