# TODOs

- [x] Support `schedule` subcommand for running background sync cronjob
  - [x] Configurable schedule (default to every 60 minutes)
  - [ ] Respect user's config &rarr; config context struct
- [ ] Search over posts via `ripgrep` library/ SQL search
  - [ ] `/` key to search
- [x] Rich HTML text display with [`html2text`](https://crates.io/crates/html2text)
- [ ] Support "go to external" mapping `x` (similar to `gx` in Vim) when in post
      to go to original
- [ ] Fetch whole post contents (not just until `<-- more! -->` tag)
- [ ] OPML import/export support
  - [OPML specification](https://opml.org/spec2.opml)
- [ ] Sync feeds key binding (default to `s`)
- [ ] Logging file
- [ ] Read/unread indicators
- [x] `<Space>`/ `Backspace` for page scrolling
  - Tried `Shift` + `<Space>` but the modifier combination is not supported by a
    all terminals, so switch to `Backspace` instead
- [ ] Refactor `schedule` subcommand to support crontab input
  - [ ] Create minimal `crontab`pest parser crate?
