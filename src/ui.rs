//! `ferrofeed` TUI
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::{
    db::Feed,
    ui::popup::{PopupState, get_centered_popup_area, pad_top_lines_center},
};

pub mod popup;

/// Active TUI state.
#[derive(Debug)]
pub struct App {
    /// Whether the current TUI is still active
    running: bool,
    /// Any active popup
    popup: Option<PopupState>,
    /// The current page that the user is on.
    current_page: CurrentScreen,
    // Additional state with go below...
}

/// The current page
#[derive(Debug)]
enum CurrentScreen {
    /// The home library page.
    ViewFeedsPage {
        feeds: Option<Vec<Feed>>,
        selected_feed: Option<Feed>,
    },
    /// Viewing a selected post
    PostView { url: String },
}

/// Initialize the TUI.
pub fn init() -> anyhow::Result<()> {
    let terminal = ratatui::init();

    // Enter main event loop
    let result = App::new().run(terminal);

    // Restore previous terminal state
    ratatui::restore();
    result
}

impl App {
    /// Construct a new instance of [`App`].
    fn new() -> Self {
        Self {
            running: true,
            popup: None,
            current_page: CurrentScreen::ViewFeedsPage {
                feeds: None,
                selected_feed: None,
            },
        }
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
        // Right now just have a placeholder frame
        let title = Line::from(" ferrofeed ").bold().blue().left_aligned();

        // TODO: populate with actual content and the correct layout
        let text = "nulla commodo culpa magna quis dolore consectetur eiusmod\n\n\
            officia ut eu voluptate ex eiusmod commodo consectetur dolor\n\
            exercitation quis ut";

        let instructions = Line::default().spans(vec![
            " Help: ".into(),
            "? ".blue(),
            " | ".into(),
            " Quit: ".into(),
            "<q> ".blue(),
        ]);
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

        if let Some(popup) = &self.popup {
            self.render_popup(frame, popup);
        }
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
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('?')) => {
                if self.popup.is_none() {
                    self.popup = Some(PopupState::Help)
                } else {
                    // Quit help menu
                    self.popup = None
                }
            }
            _ => {
                // Nothing, proceed with event loop
            }
        }
    }

    /// Display a centered overlay with the given pane over the current screen.
    fn render_popup(&self, frame: &mut Frame, popup: &PopupState) {
        let area = frame.area();
        let popup_area = get_centered_popup_area(area, 25, 20);
        match popup {
            PopupState::Help => {
                let key_style = Style::default().fg(Color::Blue).bold();
                let lines = vec![
                    Line::from(vec![
                        Span::raw("Move Up: "),
                        Span::styled("↑", key_style),
                        Span::raw(" / "),
                        Span::styled("k", key_style),
                    ]),
                    Line::from(vec![
                        Span::raw("Move Down: "),
                        Span::styled("↓", key_style),
                        Span::raw(" / "),
                        Span::styled("j", key_style),
                    ]),
                    Line::from(vec![Span::raw("To Top: "), Span::styled("g", key_style)]),
                    Line::from(vec![Span::raw("To Bottom: "), Span::styled("G", key_style)]),
                    Line::from(vec![
                        Span::raw("Toggle Help: "),
                        Span::styled("?", key_style),
                    ]),
                    Line::from(vec![Span::raw("Quit: "), Span::styled("q", key_style)]),
                ];

                // Pad top lines to center vertically
                let padded_lines = pad_top_lines_center(lines, popup_area, true);

                let quit_instruction = Line::from(vec![" Exit Help: ".into(), "? ".blue()]);
                frame.render_widget(Clear, popup_area);
                frame.render_widget(
                    Paragraph::new(padded_lines)
                        .block(
                            Block::bordered()
                                .title(" Help ".blue())
                                .title_bottom(quit_instruction.right_aligned()),
                        )
                        .centered()
                        .wrap(Wrap { trim: true }),
                    popup_area,
                );
            }
        }
    }

    /// Set the running state to false to quit the application.
    fn quit(&mut self) {
        self.running = false
    }
}
