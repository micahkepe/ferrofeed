//! `ferrofeed` TUI
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

/// Active TUI state.
#[derive(Debug)]
pub struct App {
    /// Whether the current TUI is still active
    running: bool,
    /// The current page that the user is on.
    current_page: CurrentScreen,
    // Additional state with go below...
}

/// The current page
#[derive(Debug)]
enum CurrentScreen {
    /// The home library page.
    ViewFeedsPage,
    /// Viewing a selected post
    PostView,
    /// The help page.
    HelpPage,
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
            current_page: CurrentScreen::ViewFeedsPage,
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
        let text = "nulla commodo culpa magna quis dolore consectetur eiusmod\n\n\
            officia ut eu voluptate ex eiusmod commodo consectetur dolor\n\
            exercitation quis ut";
        let instructions = Line::from(vec![" Quit: ".into(), "<q> ".blue()]);
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
            (_, KeyCode::Char('?')) => self.display_help_popup(),
            _ => {
                // Nothing, proceed with event loop
            }
        }
    }

    /// Display an overlay with the help pane over the current screen.
    fn display_help_popup(&self) {
        todo!()
    }

    /// Set the running state to false to quit the application.
    fn quit(&mut self) {
        self.running = false
    }
}
