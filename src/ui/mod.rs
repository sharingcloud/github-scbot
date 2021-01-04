//! Ui

mod app;
mod events;
mod errors;
mod state;

use std::io;

use events::Events;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{Terminal, backend::TermionBackend, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, text::{Span, Spans}, widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}};

use crate::database::establish_single_connection;
use self::errors::Result;

use self::{app::App, events::Event};

#[allow(clippy::too_many_lines)]
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

    let mut current_key_pressed: Option<Key> = None;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10)
                    ].as_ref()
                )
                .split(f.size());

            // let repo_items = {
            //     let items: Vec<ListItem> = app
            //         .repositories
            //         .items
            //         .iter()
            //         .map(|i| {
            //             let lines = vec![Spans::from(i.full_name())];
            //             ListItem::new(lines)
            //         })
            //         .collect();

            //     List::new(items)
            //         .block(Block::default().borders(Borders::ALL).title("Repositories"))
            //         .highlight_style(
            //             Style::default()
            //                 .bg(Color::LightGreen)
            //                 .add_modifier(Modifier::BOLD),
            //         )
            //         .highlight_symbol(">> ")
            // };

            let help_text = Paragraph::new(format!("Welcome on SC Bot! ({:?})", current_key_pressed)).block(Block::default().borders(Borders::ALL));

            // let pr_items = {
            //     let items: Vec<ListItem> = app
            //         .pull_requests
            //         .items
            //         .iter()
            //         .map(|i| {
            //             let lines = vec![Spans::from(format!("#{} - {}", i.number, i.name))];
            //             ListItem::new(lines)
            //         })
            //         .collect();

            //     List::new(items)
            //         .block(Block::default().borders(Borders::ALL).title("Pull requests"))
            //         .highlight_style(
            //             Style::default()
            //                 .bg(Color::LightGreen)
            //                 .add_modifier(Modifier::BOLD),
            //         )
            //         .highlight_symbol(">> ")
            // };

            // f.render_stateful_widget(repo_items, chunks[0], &mut app.repositories.state);
            // f.render_stateful_widget(pr_items, chunks[1], &mut app.pull_requests.state);
            f.render_widget(help_text, chunks[2]);
        })?;

        match events.next()? {
            Event::Input(input) => {
                current_key_pressed = Some(input);
                app.on_key(input);
            },
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
