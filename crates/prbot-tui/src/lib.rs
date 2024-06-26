//! UI module.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod app;
mod errors;
mod state;
mod terminal;

use std::time::{Duration, Instant};

use app::App;
use crossterm::event::{Event, KeyEventKind};
pub use errors::UiError;
use prbot_database_interface::DbService;
use terminal::TerminalWrapper;

use self::errors::Result;

/// Run TUI interface.
pub async fn run_tui(db_service: &dyn DbService) -> Result<()> {
    let mut terminal = TerminalWrapper::new()?;

    let mut app = App::new("prbot");
    app.load_from_db(db_service).await?;

    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            app.draw(f);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.on_key(key.code);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
