# TODOs

## Priority levels:

- 🔥 High priority
- ‼️ Medium priority
- ❗ Low priority
- 🐛 Bugfix

---

## CLI

- [x] Support `schedule` subcommand for running background sync cronjob
  - [x] Configurable schedule (default to every 60 minutes)
  - [ ] Respect user's config &rarr; config context struct
- [ ] OPML import/export support
  - [OPML specification](https://opml.org/spec2.opml)

## TUI

- [ ] Search over posts via `ripgrep` library/ SQL search
  - [ ] `/` key to search
- [x] Rich HTML text display with [`html2text`](https://crates.io/crates/html2text)
- [x] Support "go to external" mapping `x` (similar to `gx` in Vim) when in post
      to go to original
- [ ] ‼️ Fetch whole post contents (not just until `<-- more! -->` tag)
- [ ] Sync feeds key binding (default to `s`)
- [ ] Read/unread indicators
- [x] `<Space>`/ `Backspace` for page scrolling
  - Tried `Shift` + `<Space>` but the modifier combination is not supported by a
    all terminals, so switch to `Backspace` instead
- [ ] Success/failure toasts

## Misc.

- [ ] Logging file
- [ ] Refactor `schedule` subcommand to support crontab input
  - [ ] Create minimal `crontab`pest parser crate?
- [ ] Organize modules/ helper functions for popups
  - [ ] Consolidate instruction text for popups to helper function
- [ ] More rigorous testing and integration testing
- [ ] More robust error handling (partially dependent on success/failure toasts
      implementation)
