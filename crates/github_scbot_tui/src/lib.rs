//! UI module.

#![warn(missing_docs)]
#![warn(clippy::all)]

#[cfg(unix)]
mod app;
#[cfg(unix)]
mod events;
#[cfg(unix)]
mod state;

mod errors;
use github_scbot_database::models::IDatabaseAdapter;

use self::errors::Result;

/// Run TUI interface.
#[cfg(windows)]
pub async fn run_tui(db_adapter: &dyn IDatabaseAdapter) -> Result<()> {
    use self::errors::UiError;

    Err(UiError::Unsupported)
}

/// Run TUI interface.
#[cfg(unix)]
pub async fn run_tui(db_adapter: &dyn IDatabaseAdapter) -> Result<()> {
    use std::io;

    use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
    use tui::{backend::TermionBackend, Terminal};

    use self::{
        app::App,
        events::{Event, Events},
    };

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = Events::new();

    let mut app = App::new("SC Bot");
    app.load_from_db(db_adapter).await?;

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
