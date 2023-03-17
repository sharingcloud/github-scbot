use std::{
    io::{self, Stdout},
    ops::{Deref, DerefMut},
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::Result;

pub struct TerminalWrapper {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalWrapper {
    pub fn new() -> Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        Ok(Self {
            terminal: Terminal::new(backend)?,
        })
    }
}

impl Deref for TerminalWrapper {
    type Target = Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for TerminalWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for TerminalWrapper {
    fn drop(&mut self) {
        // Restore UI
        crossterm::terminal::disable_raw_mode().unwrap();
        crossterm::execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
