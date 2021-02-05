//! Application module.

use github_scbot_database::{
    models::{PullRequestModel, RepositoryModel},
    DbConn,
};
use termion::event::Key;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::{errors::Result, state::AppState};

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub data: AppState,
    pub last_key_pressed: Option<Key>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            should_quit: false,
            data: AppState::new(),
            last_key_pressed: None,
        }
    }

    pub fn load_from_db(&mut self, conn: &DbConn) -> Result<()> {
        let repositories = RepositoryModel::list(conn)?;
        let pull_requests = PullRequestModel::list(conn)?;

        let mut pr_kvs = Vec::new();
        for repo in repositories {
            let mut prs = Vec::new();
            for pr in &pull_requests {
                if repo.id == pr.repository_id {
                    prs.push(pr.clone());
                }
            }

            pr_kvs.push((repo, prs));
        }

        self.data = AppState::with_items(pr_kvs);

        Ok(())
    }

    pub fn draw_repositories<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let repo_items = {
            let items: Vec<ListItem> = self
                .data
                .data
                .iter()
                .map(|i| {
                    let lines = vec![Spans::from(i.0.get_path())];
                    ListItem::new(lines)
                })
                .collect();

            List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Repositories"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ")
        };

        f.render_stateful_widget(repo_items, area, &mut self.data.repositories_state);
    }

    pub fn draw_pull_requests<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(area);

        let pr_items = {
            let items: Vec<ListItem> = self
                .data
                .pull_requests_for_repository()
                .iter()
                .map(|i| {
                    let lines = vec![Spans::from(format!("#{} - {}", i.get_number(), i.name))];
                    ListItem::new(lines)
                })
                .collect();

            List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Pull requests"),
                )
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ")
        };

        f.render_stateful_widget(pr_items, chunks[0], &mut self.data.pull_requests_state);
    }

    pub fn draw_help<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let help_text = Paragraph::new(format!("Welcome on SC Bot! ({:?})", self.last_key_pressed))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help_text, area);
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(f.size());

        let repo_area = chunks[0];
        let pr_area = chunks[1];
        let help_area = chunks[2];

        self.draw_repositories(f, repo_area);
        self.draw_pull_requests(f, pr_area);
        self.draw_help(f, help_area);
    }

    pub fn on_key(&mut self, key: Key) {
        match key {
            Key::Char(c) => match c {
                'q' => {
                    self.should_quit = true;
                }
                o => {
                    self.data.on_ui_key(Key::Char(o));
                }
            },
            o => {
                self.last_key_pressed = Some(o);
                self.data.on_ui_key(o);
            }
        }
    }

    #[allow(clippy::unused_self)]
    pub fn on_tick(&mut self) {
        // TICK
    }
}
