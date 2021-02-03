//! UI module.

mod app;
mod errors;
mod events;
mod state;

use std::io;

use github_scbot_database::establish_single_connection;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

use self::{
    app::App,
    errors::Result,
    events::{Event, Events},
};

/// Run TUI interface.
pub fn run_tui() -> Result<()> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::new();

    let conn = establish_single_connection()?;
    let mut app = App::new("SC Bot");
    app.load_from_db(&conn)?;

    loop {
        terminal.draw(|f| {
            app.draw(f);
        })?;

        match events.next()? {
            Event::Input(input) => {
                app.on_key(input);
            }
            Event::Tick => {
                app.on_tick();
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
