//! `ferrofeed` TUI
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    prelude::Stylize,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
};

use crate::{
    db::{Db, Feed, FeedItem},
    ui::popup::{PopupState, get_centered_popup_area, pad_top_lines_center},
};

pub(crate) mod popup;
pub(crate) mod rich_text;

/// Active TUI state.
pub struct App<'a> {
    /// Whether the current TUI is still active
    running: bool,
    /// Any active popup
    popup: Option<PopupState>,
    /// The current page that the user is on.
    current_page: CurrentScreen,
    /// Database connection
    db: &'a Db,
    /// Feed list state for navigation
    feed_list_state: ListState,
    /// Item list state for navigation
    item_list_state: ListState,
    /// Scrollbar state for help popup
    help_scroll_state: ScrollbarState,
    /// Scroll position for help popup
    help_scroll: u16,
}

/// The current page
enum CurrentScreen {
    /// The home library page showing feeds.
    Feeds { feeds: Vec<Feed> },
    /// Viewing items for a selected feed
    Items { feed: Feed, items: Vec<FeedItem> },
    /// Viewing content of a specific item
    ViewPost {
        feed: Feed,
        items: Vec<FeedItem>,
        item: FeedItem,
        scroll: u16,
    },
}

/// Initialize the TUI.
pub fn init(db: &Db) -> anyhow::Result<()> {
    let terminal = ratatui::init();

    // Enter main event loop
    let result = App::new(db)?.run(terminal);

    // Restore previous terminal state
    ratatui::restore();
    result
}

impl<'a> App<'a> {
    /// Construct a new instance of [`App`].
    fn new(db: &'a Db) -> Result<Self> {
        let feeds = db.list_feeds()?;
        let mut feed_list_state = ListState::default();
        if !feeds.is_empty() {
            feed_list_state.select(Some(0));
        }

        Ok(Self {
            running: true,
            popup: None,
            current_page: CurrentScreen::Feeds { feeds },
            db,
            feed_list_state,
            item_list_state: ListState::default(),
            help_scroll_state: ScrollbarState::default(),
            help_scroll: 0,
        })
    }

    /// Runs the TUI application's main loop.
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_event()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        // Clone the current page to avoid borrow checker issues
        let current_page = match &self.current_page {
            CurrentScreen::Feeds { feeds } => CurrentScreen::Feeds {
                feeds: feeds.clone(),
            },
            CurrentScreen::Items { feed, items } => CurrentScreen::Items {
                feed: feed.clone(),
                items: items.clone(),
            },
            CurrentScreen::ViewPost {
                feed,
                items,
                item,
                scroll,
            } => CurrentScreen::ViewPost {
                feed: feed.clone(),
                items: items.clone(),
                item: item.clone(),
                scroll: *scroll,
            },
        };

        match &current_page {
            CurrentScreen::Feeds { feeds } => {
                self.render_feeds_page(frame, feeds);
            }
            CurrentScreen::Items { feed, items } => {
                self.render_items_page(frame, feed, items);
            }
            CurrentScreen::ViewPost {
                feed,
                items,
                item,
                scroll,
            } => {
                self.render_post_page(frame, feed, items, item, *scroll);
            }
        }

        if let Some(popup) = self.popup.clone() {
            self.render_popup(frame, &popup);
        }
    }

    /// Render the feeds list page.
    fn render_feeds_page(&mut self, frame: &mut Frame, feeds: &[Feed]) {
        let title = Line::from(" ferrofeed - Feeds ")
            .bold()
            .blue()
            .left_aligned();

        let instructions = Line::default().spans(vec![
            " Navigate: ".into(),
            "j/k ".blue(),
            " | ".into(),
            " Select: ".into(),
            "<Enter> ".blue(),
            " | ".into(),
            " Add: ".into(),
            "a ".blue(),
            " | ".into(),
            " Delete: ".into(),
            "d ".blue(),
            " | ".into(),
            " Help: ".into(),
            "? ".blue(),
            " | ".into(),
            " Quit: ".into(),
            "q ".blue(),
        ]);

        if feeds.is_empty() {
            let mut lines: Vec<Line> = vec![
                Line::from("🦀 Looks like your feed list is feeling a bit... empty!"),
                Line::from("No worries though! Here's how to fill it up:"),
                Line::from(""),
                Line::from("📡 Add a feed from the command line:"),
                Line::from("$ ferrofeed add-feed <url>".fg(Color::Yellow)),
                Line::from(""),
                Line::from("⌨️  Or press 'a' right here to add one!"),
                Line::from("💡 Try adding a popular feed like:"),
                Line::from("https://blog.rust-lang.org/feed.xml").fg(Color::Yellow),
                Line::from("https://this-week-in-rust.org/rss.xml").fg(Color::Yellow),
            ];
            lines = pad_top_lines_center(lines, frame.area(), true);

            frame.render_widget(
                Paragraph::new(lines)
                    .block(
                        Block::bordered()
                            .title(title)
                            .title_bottom(instructions.right_aligned()),
                    )
                    .centered(),
                frame.area(),
            );
            return;
        }

        // Create list items
        let items: Vec<ListItem> = feeds
            .iter()
            .map(|feed| {
                let title = feed.title.as_deref().unwrap_or("(no title)");
                ListItem::new(Line::from(vec![Span::styled(
                    title,
                    Style::default().fg(Color::Cyan),
                )]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(title)
                    .title_bottom(instructions.right_aligned()),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, frame.area(), &mut self.feed_list_state);
    }

    /// Render the items list page for a selected feed.
    fn render_items_page(&mut self, frame: &mut Frame, feed: &Feed, items: &[FeedItem]) {
        let title = Line::from(format!(
            " {} - Items ",
            feed.title.as_deref().unwrap_or("Feed")
        ))
        .bold()
        .blue()
        .left_aligned();

        let instructions = Line::default().spans(vec![
            " Navigate: ".into(),
            "j/k ".blue(),
            " | ".into(),
            " Back: ".into(),
            "<ESC> ".blue(),
            " | ".into(),
            " Help: ".into(),
            "? ".blue(),
            " | ".into(),
            " Quit: ".into(),
            "q ".blue(),
        ]);

        if items.is_empty() {
            let text = "No items found for this feed.\n\nRun 'ferrofeed sync' to fetch items.";
            frame.render_widget(
                Paragraph::new(text)
                    .block(
                        Block::bordered()
                            .title(title)
                            .title_bottom(instructions.right_aligned()),
                    )
                    .centered(),
                frame.area(),
            );
            return;
        }

        // Create list items
        let list_items: Vec<ListItem> = items
            .iter()
            .map(|item| {
                let title = item.title.as_deref().unwrap_or("(no title)");
                let author = item
                    .author
                    .as_deref()
                    .map(|a| format!(" by {}", a))
                    .unwrap_or_default();

                let style = if item.is_read {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(title, style),
                    Span::styled(author, Style::default().fg(Color::Yellow)),
                ]))
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::bordered()
                    .title(title)
                    .title_bottom(instructions.right_aligned()),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, frame.area(), &mut self.item_list_state);
    }

    /// Render the post content page for a selected item.
    fn render_post_page(
        &mut self,
        frame: &mut Frame,
        _feed: &Feed,
        _items: &[FeedItem],
        item: &FeedItem,
        scroll: u16,
    ) {
        let title = Line::from(format!(" {} ", item.title.as_deref().unwrap_or("Post")))
            .bold()
            .blue()
            .left_aligned();

        let instructions = Line::default().spans(vec![
            " Scroll: ".into(),
            "j/k ".blue(),
            " | ".into(),
            " Back: ".into(),
            "<ESC> ".blue(),
            " | ".into(),
            " Quit: ".into(),
            "q ".blue(),
        ]);

        // Build content
        let mut lines: Vec<Line> = Vec::new();

        if let Some(link) = &item.link {
            lines.push(Line::from(vec!["Link: ".fg(Color::Yellow), link.into()]));
        }

        if let Some(author) = &item.author {
            lines.push(Line::from(vec![
                "Author: ".fg(Color::Yellow),
                author.into(),
            ]));
        }

        if let Some(published) = item.published {
            use time::OffsetDateTime;
            if let Ok(dt) = OffsetDateTime::from_unix_timestamp(published) {
                let formatted = format!("{:04}-{:02}-{:02}", dt.year(), dt.month() as u8, dt.day());
                lines.push(Line::from(vec![
                    "Published: ".fg(Color::Yellow),
                    formatted.into(),
                ]));
            } else {
                lines.push(Line::from("N/A".italic()));
            }
        }

        // Separator
        lines.push(Line::from(""));

        if let Some(desc) = &item.description {
            match rich_text::html_to_rich_text(desc) {
                Ok(styled_lines) => lines.extend(styled_lines),
                Err(_) => lines.push(Line::from("Error rendering HTML".italic())),
            }
        } else {
            lines.push(Line::from("No description available.".italic()));
        }

        frame.render_widget(
            Paragraph::new(lines)
                .block(
                    Block::bordered()
                        .title(title)
                        .title_bottom(instructions.right_aligned()),
                )
                .wrap(Wrap { trim: true })
                .scroll((scroll, 0)),
            frame.area(),
        );
    }

    /// Reads the [`crossterm`] events and updates the state of [`App`].
    ///
    /// NOTE: `event::read()` is blocking, so if work needs to be down between event handling, use
    /// [`event::poll`] function to check for available events with a timeout.
    fn handle_crossterm_event(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind.is_press() => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        // Handle popup-specific input
        if let Some(popup) = &mut self.popup {
            match popup {
                PopupState::Help => {
                    match key.code {
                        KeyCode::Char('?') | KeyCode::Esc => {
                            self.popup = None;
                            self.help_scroll = 0; // Reset scroll when closing
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.help_scroll = self.help_scroll.saturating_add(1);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.help_scroll = self.help_scroll.saturating_sub(1);
                        }
                        KeyCode::Char('g') => {
                            self.help_scroll = 0;
                        }
                        KeyCode::Char('G') => {
                            self.help_scroll = u16::MAX; // Will be clamped by rendering
                        }
                        KeyCode::Char('q') => {
                            self.quit();
                        }
                        _ => {}
                    }
                }
                PopupState::AddFeed { input } => {
                    match key.code {
                        KeyCode::Char(c) => {
                            input.push(c);
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Enter => {
                            // Submit the feed URL
                            let url = input.clone();
                            self.popup = None;
                            if !url.is_empty() {
                                self.add_feed_async(url);
                            }
                        }
                        KeyCode::Esc => {
                            self.popup = None;
                        }
                        _ => {}
                    }
                }
                PopupState::DeleteFeed { feed_url } => {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // Confirm delete
                            let url = feed_url.clone();
                            self.popup = None;
                            self.delete_feed(&url);
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            // Cancel
                            self.popup = None;
                        }
                        _ => {}
                    }
                }
            }
            return;
        }

        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('?')) => {
                self.popup = Some(PopupState::Help);
            }
            (_, KeyCode::Char('a')) => {
                // Only allow adding feeds on the feeds page
                if matches!(self.current_page, CurrentScreen::Feeds { .. }) {
                    self.popup = Some(PopupState::AddFeed {
                        input: String::new(),
                    });
                }
            }
            (_, KeyCode::Char('d')) => {
                // Only allow deleting feeds on the feeds page
                self.try_delete_feed();
            }
            (_, KeyCode::Char('j') | KeyCode::Down) => {
                self.move_down();
            }
            (_, KeyCode::Char('k') | KeyCode::Up) => {
                self.move_up();
            }
            (_, KeyCode::Char('g')) => {
                self.move_top();
            }
            (_, KeyCode::Char('G')) => {
                self.move_bottom();
            }
            (_, KeyCode::Enter) => {
                self.select_item();
            }
            (_, KeyCode::Esc) => {
                self.go_back();
            }
            _ => {}
        }
    }

    /// Move selection down.
    fn move_down(&mut self) {
        match &mut self.current_page {
            CurrentScreen::Feeds { feeds } => {
                if feeds.is_empty() {
                    return;
                }
                let i = match self.feed_list_state.selected() {
                    Some(i) => {
                        if i >= feeds.len() - 1 {
                            i
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.feed_list_state.select(Some(i));
            }
            CurrentScreen::Items { items, .. } => {
                if items.is_empty() {
                    return;
                }
                let i = match self.item_list_state.selected() {
                    Some(i) => {
                        if i >= items.len() - 1 {
                            i
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.item_list_state.select(Some(i));
            }
            CurrentScreen::ViewPost { scroll, .. } => {
                *scroll = scroll.saturating_add(1);
            }
        }
    }

    /// Move selection up.
    fn move_up(&mut self) {
        match &mut self.current_page {
            CurrentScreen::Feeds { .. } => {
                let i = match self.feed_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            0
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.feed_list_state.select(Some(i));
            }
            CurrentScreen::Items { .. } => {
                let i = match self.item_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            0
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.item_list_state.select(Some(i));
            }
            CurrentScreen::ViewPost { scroll, .. } => {
                *scroll = scroll.saturating_sub(1);
            }
        }
    }

    /// Move to the top of the list.
    fn move_top(&mut self) {
        match &mut self.current_page {
            CurrentScreen::Feeds { feeds } => {
                if !feeds.is_empty() {
                    self.feed_list_state.select(Some(0));
                }
            }
            CurrentScreen::Items { items, .. } => {
                if !items.is_empty() {
                    self.item_list_state.select(Some(0));
                }
            }
            CurrentScreen::ViewPost { scroll, .. } => {
                *scroll = 0;
            }
        }
    }

    /// Move to the bottom of the list.
    fn move_bottom(&mut self) {
        match &mut self.current_page {
            CurrentScreen::Feeds { feeds } => {
                if !feeds.is_empty() {
                    self.feed_list_state.select(Some(feeds.len() - 1));
                }
            }
            CurrentScreen::Items { items, .. } => {
                if !items.is_empty() {
                    self.item_list_state.select(Some(items.len() - 1));
                }
            }
            CurrentScreen::ViewPost { scroll, .. } => {
                *scroll = u16::MAX; // Will be clamped by paragraph rendering
            }
        }
    }

    /// Select the currently highlighted item.
    fn select_item(&mut self) {
        match &self.current_page {
            CurrentScreen::Feeds { feeds } => {
                if let Some(selected) = self.feed_list_state.selected()
                    && let Some(feed) = feeds.get(selected)
                {
                    // Load items for the selected feed
                    if let Ok(items) = self.db.get_feed_items(feed.id) {
                        self.item_list_state
                            .select(if items.is_empty() { None } else { Some(0) });
                        self.current_page = CurrentScreen::Items {
                            feed: feed.clone(),
                            items,
                        };
                    }
                }
            }
            CurrentScreen::Items { feed, items } => {
                if let Some(selected) = self.item_list_state.selected()
                    && let Some(item) = items.get(selected)
                {
                    // Open post content view
                    self.current_page = CurrentScreen::ViewPost {
                        feed: feed.clone(),
                        items: items.clone(),
                        item: item.clone(),
                        scroll: 0,
                    };
                }
            }
            CurrentScreen::ViewPost { .. } => {
                // Already viewing post, do nothing
            }
        }
    }

    /// Go back to the previous screen.
    fn go_back(&mut self) {
        match &self.current_page {
            CurrentScreen::Feeds { .. } => {
                // Already at the top level, do nothing
            }
            CurrentScreen::Items { .. } => {
                // Go back to feeds list
                if let Ok(feeds) = self.db.list_feeds() {
                    // Restore selection or select first item if available
                    let selected = self.feed_list_state.selected();
                    if selected.is_none() && !feeds.is_empty() {
                        self.feed_list_state.select(Some(0));
                    } else if let Some(sel) = selected {
                        // Clamp selection to valid range
                        if sel >= feeds.len() && !feeds.is_empty() {
                            self.feed_list_state.select(Some(feeds.len() - 1));
                        }
                    }
                    self.current_page = CurrentScreen::Feeds { feeds };
                }
            }
            CurrentScreen::ViewPost { feed, items, .. } => {
                // Go back to items list
                self.current_page = CurrentScreen::Items {
                    feed: feed.clone(),
                    items: items.clone(),
                };
            }
        }
    }

    /// Display a centered overlay with the given pane over the current screen.
    fn render_popup(&mut self, frame: &mut Frame, popup: &PopupState) {
        let area = frame.area();
        match popup {
            PopupState::Help => {
                let popup_area = get_centered_popup_area(area, 50, 60);
                let key_style = Style::default().fg(Color::Blue).bold();
                let section_title =
                    |title: &str| Line::from(format!("{}:", title).bold().bg(Color::DarkGray));
                let lines = vec![
                    // Navigation
                    section_title("Navigation"),
                    Line::from(vec![
                        Span::raw("  Move Up: "),
                        Span::styled("↑", key_style),
                        Span::raw(" / "),
                        Span::styled("k", key_style),
                    ]),
                    Line::from(vec![
                        Span::raw("  Move Down: "),
                        Span::styled("↓", key_style),
                        Span::raw(" / "),
                        Span::styled("j", key_style),
                    ]),
                    Line::from(vec![Span::raw("  To Top: "), Span::styled("g", key_style)]),
                    Line::from(vec![
                        Span::raw("  To Bottom: "),
                        Span::styled("G", key_style),
                    ]),
                    Line::from(""),
                    // Actions
                    section_title("Actions"),
                    Line::from(vec![
                        Span::raw("  Select: "),
                        Span::styled("Enter", key_style),
                    ]),
                    Line::from(vec![
                        Span::raw("  Go Back: "),
                        Span::styled("<ESC>", key_style),
                    ]),
                    Line::from(vec![
                        Span::raw("  Add Feed: "),
                        Span::styled("a", key_style),
                        Span::raw(" (Feeds page only)").dim(),
                    ]),
                    Line::from(vec![
                        Span::raw("  Delete Feed: "),
                        Span::styled("d", key_style),
                        Span::raw(" (Feeds page only)").dim(),
                    ]),
                    Line::from(""),
                    // Other
                    section_title("Other"),
                    Line::from(vec![
                        Span::raw("  Toggle Help: "),
                        Span::styled("?", key_style),
                    ]),
                    Line::from(vec![Span::raw("  Quit: "), Span::styled("q", key_style)]),
                ];

                let quit_instruction = Line::from(vec![
                    " Exit Help: ".into(),
                    Span::styled("<ESC>", key_style),
                    Span::raw(" / "),
                    Span::styled("? ", key_style),
                ]);

                // Calculate scrollbar state
                let total_lines = lines.len();
                let visible_lines = popup_area.height.saturating_sub(2) as usize; // subtract borders

                // Clamp scroll position
                let max_scroll = total_lines.saturating_sub(visible_lines);
                self.help_scroll = self.help_scroll.min(max_scroll as u16);

                self.help_scroll_state = self
                    .help_scroll_state
                    .content_length(total_lines)
                    .viewport_content_length(visible_lines)
                    .position(self.help_scroll as usize);

                frame.render_widget(Clear, popup_area);

                // Render paragraph with scroll
                let paragraph = Paragraph::new(lines)
                    .block(
                        Block::bordered()
                            .title(" Help ".blue())
                            .title_bottom(quit_instruction.right_aligned()),
                    )
                    .wrap(Wrap { trim: true })
                    .scroll((self.help_scroll, 0));

                frame.render_widget(paragraph, popup_area);

                // Render scrollbar on the right side if content is longer than viewport
                if total_lines > visible_lines {
                    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                    frame.render_stateful_widget(
                        scrollbar,
                        popup_area,
                        &mut self.help_scroll_state,
                    );
                }
            }
            PopupState::AddFeed { input } => {
                let popup_area = get_centered_popup_area(area, 60, 30);

                // Display input with cursor
                let input_with_cursor = format!("{}█", input);

                let mut lines = vec![
                    Line::from("Enter feed URL:"),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        &input_with_cursor,
                        Style::default().fg(Color::Yellow),
                    )]),
                    Line::from(""),
                    Line::from("Press Enter to add, Esc to cancel."),
                    Line::from(""),
                ];
                lines = pad_top_lines_center(lines, popup_area, true);

                frame.render_widget(Clear, popup_area);
                frame.render_widget(
                    Paragraph::new(lines)
                        .block(Block::bordered().title(" Add Feed ".blue()))
                        .wrap(Wrap { trim: true }),
                    popup_area,
                );
            }
            PopupState::DeleteFeed { feed_url } => {
                let popup_area = get_centered_popup_area(area, 60, 30);
                let mut lines = vec![
                    Line::from("Are you sure you want to delete this feed?"),
                    Line::from(vec![">> ".into(), feed_url.as_str().fg(Color::Yellow)]),
                    Line::from(""),
                    Line::from("This will also delete all items from this feed."),
                ];
                lines = pad_top_lines_center(lines, popup_area, true);

                let buttons = Line::from(vec![
                    " ".into(),
                    "[".into(),
                    "Y".bold().red(),
                    "]".into(),
                    "es  ".into(),
                    "[".into(),
                    "N".bold().blue(),
                    "]".into(),
                    "o  ".into(),
                    "[".into(),
                    "Esc".bold(),
                    "]".into(),
                    " Cancel ".into(),
                ]);

                frame.render_widget(Clear, popup_area);
                frame.render_widget(
                    Paragraph::new(lines)
                        .block(
                            Block::bordered()
                                .title(" Confirm Delete ".red())
                                .title_bottom(buttons.centered())
                                .border_style(Style::default().fg(Color::Red)),
                        )
                        .centered()
                        .wrap(Wrap { trim: true }),
                    popup_area,
                );
            }
        }
    }

    /// Try to delete the currently selected feed (shows confirmation popup).
    fn try_delete_feed(&mut self) {
        if let CurrentScreen::Feeds { feeds } = &self.current_page
            && let Some(selected) = self.feed_list_state.selected()
            && let Some(feed) = feeds.get(selected)
        {
            self.popup = Some(PopupState::DeleteFeed {
                feed_url: feed.url.clone(),
            });
        }
    }

    /// Delete a feed from the database and refresh the feed list.
    fn delete_feed(&mut self, url: &str) {
        if self.db.remove_feed(url).is_ok() {
            // Refresh feed list
            if let Ok(feeds) = self.db.list_feeds() {
                // Reset selection based on whether list is empty
                let selection = if feeds.is_empty() { None } else { Some(0) };
                self.feed_list_state.select(selection);
                self.current_page = CurrentScreen::Feeds { feeds };
            }
        }
    }

    /// Add a feed and refresh the UI.
    fn add_feed_async(&mut self, url: String) {
        // Clone URL for use in thread
        let url_clone = url.clone();

        // Create a new runtime for this blocking operation
        // This is necessary because we're already inside a Tokio runtime
        let result = std::thread::spawn(move || {
            // Create a new runtime in the spawned thread
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                // Fetch and parse the feed
                crate::client::fetch_feed(&url_clone).await
            })
        })
        .join();

        // Process the result
        match result {
            Ok(Ok(parsed_feed)) => {
                // Add to database
                if self.db.add_feed(&url, parsed_feed.title.as_deref()).is_ok() {
                    // Get the newly added feed to sync its items
                    if let Ok(feeds) = self.db.list_feeds()
                        && let Some(feed) = feeds.iter().find(|f| f.url == url)
                    {
                        // Add all feed items to the database
                        for item in parsed_feed.items {
                            let _ = self.db.add_feed_item(
                                feed.id,
                                item.title.as_deref(),
                                item.link.as_deref(),
                                item.description.as_deref(),
                                item.author.as_deref(),
                                item.published,
                            );
                        }
                    }
                }
            }
            _ => {
                // Error occurred, but we'll refresh the list anyway
            }
        }

        // Refresh the feed list
        if let Ok(feeds) = self.db.list_feeds() {
            let mut new_list_state = ListState::default();
            if !feeds.is_empty() {
                new_list_state.select(Some(0));
            }
            self.feed_list_state = new_list_state;
            self.current_page = CurrentScreen::Feeds { feeds };
        }
    }

    /// Set the running state to false to quit the application.
    fn quit(&mut self) {
        self.running = false
    }
}
