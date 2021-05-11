//! Application module.

use github_scbot_database::{
    models::{PullRequestModel, RepositoryModel},
    DbConn,
};
use github_scbot_types::status::{CheckStatus, QaStatus};
use termion::event::Key;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
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
        let mut pull_requests = PullRequestModel::list(conn)?;
        pull_requests.sort_by_key(|p| -(p.get_number() as i64));

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

        // Autoselect first repository if available
        self.data.set_first_selection();

        Ok(())
    }

    pub fn draw_repositories<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(area);

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

        f.render_stateful_widget(repo_items, chunks[0], &mut self.data.repositories_state);
        self.draw_current_repository_data(f, chunks[1]);
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
                    let desc = format!("#{} - {}", i.get_number(), i.name);
                    let lines = vec![if i.closed {
                        Spans::from(vec![Span::styled(
                            desc,
                            Style::default().add_modifier(Modifier::CROSSED_OUT),
                        )])
                    } else {
                        Spans::from(desc)
                    }];
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
        self.draw_current_pull_request_data(f, chunks[1]);
    }

    pub fn draw_current_repository_data<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        if let Some(selected_repo) = self.data.get_current_repository() {
            let text = vec![
                Spans::from(vec![Span::styled(
                    selected_repo.get_path(),
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Spans::from(""),
                Spans::from(vec![
                    Span::styled(
                        "Pull request count",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": "),
                    Span::styled(
                        format!(
                            "{}",
                            self.data.data[self.data.repositories_state.selected().unwrap()]
                                .1
                                .len()
                        ),
                        Style::default().fg(Color::Blue),
                    ),
                ]),
            ];

            let paragraph = Paragraph::new(text).block(
                Block::default()
                    .title("Current repository")
                    .borders(Borders::ALL),
            );
            f.render_widget(paragraph, area)
        } else {
            let paragraph = Paragraph::new("Select a repository to display information").block(
                Block::default()
                    .title("Current repository")
                    .borders(Borders::ALL),
            );
            f.render_widget(paragraph, area)
        }
    }

    pub fn draw_current_pull_request_data<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        if let Some(selected_pr) = self.data.get_current_pull_request() {
            let text = vec![
                Spans::from(vec![Span::styled(
                    format!(
                        "{title} - #{number}",
                        title = selected_pr.name,
                        number = selected_pr.get_number()
                    ),
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Spans::from(""),
                Spans::from(vec![
                    Span::styled("base", Style::default().fg(Color::Green)),
                    Span::raw(": "),
                    Span::raw(&selected_pr.base_branch),
                    Span::raw(" <-- "),
                    Span::styled("head", Style::default().fg(Color::Green)),
                    Span::raw(": "),
                    Span::raw(&selected_pr.head_branch),
                ]),
                Spans::from(""),
                Spans::from(vec![
                    Span::styled("Step", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(": "),
                    match selected_pr.get_step_label() {
                        Some(label) => {
                            Span::styled(label.to_str(), Style::default().fg(Color::Green))
                        }
                        None => Span::styled("—", Style::default().fg(Color::Yellow)),
                    },
                ]),
                Spans::from(vec![
                    Span::styled(
                        "Check status",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": "),
                    {
                        let status = selected_pr.get_checks_status();
                        let color = match status {
                            CheckStatus::Pass | CheckStatus::Skipped => Color::Green,
                            CheckStatus::Fail => Color::Red,
                            CheckStatus::Waiting => Color::Yellow,
                        };
                        Span::styled(status.to_str(), Style::default().fg(color))
                    },
                ]),
                Spans::from(vec![
                    Span::styled("QA status", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(": "),
                    {
                        let status = selected_pr.get_qa_status();
                        let color = match status {
                            QaStatus::Pass | QaStatus::Skipped => Color::Green,
                            QaStatus::Fail => Color::Red,
                            QaStatus::Waiting => Color::Yellow,
                        };
                        Span::styled(status.to_str(), Style::default().fg(color))
                    },
                ]),
                Spans::from(vec![
                    Span::styled(
                        "Needed reviewers count",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": "),
                    Span::styled(
                        format!("{}", selected_pr.needed_reviewers_count),
                        Style::default().fg(Color::Blue),
                    ),
                ]),
                Spans::from(vec![
                    Span::styled("WIP?", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    {
                        let (msg, color) = if selected_pr.wip {
                            ("Yes", Color::Yellow)
                        } else {
                            ("No", Color::Green)
                        };
                        Span::styled(msg, Style::default().fg(color))
                    },
                ]),
                Spans::from(vec![
                    Span::styled("Locked?", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    {
                        let (msg, color) = if selected_pr.locked {
                            ("Yes", Color::Red)
                        } else {
                            ("No", Color::Green)
                        };
                        Span::styled(msg, Style::default().fg(color))
                    },
                ]),
                Spans::from(vec![
                    Span::styled("Merged?", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    {
                        let (msg, color) = if selected_pr.merged {
                            ("Yes", Color::Green)
                        } else {
                            ("No", Color::Yellow)
                        };
                        Span::styled(msg, Style::default().fg(color))
                    },
                ]),
                Spans::from(vec![
                    Span::styled("Closed?", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    {
                        let (msg, color) = if selected_pr.closed {
                            ("Yes", Color::Green)
                        } else {
                            ("No", Color::Yellow)
                        };
                        Span::styled(msg, Style::default().fg(color))
                    },
                ]),
            ];

            let paragraph = Paragraph::new(text).block(
                Block::default()
                    .title("Current pull request")
                    .borders(Borders::ALL),
            );
            f.render_widget(paragraph, area)
        } else {
            let paragraph = Paragraph::new("Select a pull request to display information").block(
                Block::default()
                    .title("Current pull request")
                    .borders(Borders::ALL),
            );
            f.render_widget(paragraph, area)
        }
    }

    pub fn draw_title<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let title = Spans::from(vec![Span::styled(
            "SC Bot Management - Terminal UI",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )]);
        let p = Paragraph::new(title).alignment(Alignment::Center);
        f.render_widget(p, area);
    }

    pub fn draw_help<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        if self.data.get_current_pull_request().is_some() {
            self.draw_pull_request_help(f, area);
        } else if self.data.get_current_repository().is_some() {
            self.draw_repository_help(f, area);
        } else {
            let help_text = Paragraph::new("Welcome on SC Bot!")
                .block(Block::default().title("Help").borders(Borders::ALL));
            f.render_widget(help_text, area);
        }
    }

    pub fn draw_repository_help<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let text = vec![
            Spans::from(vec![
                Span::styled("ENTER", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Select repository"),
            ]),
            Spans::from(vec![
                Span::styled("UP", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Move selection cursor up"),
            ]),
            Spans::from(vec![
                Span::styled("DOWN", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Move selection cursor down"),
            ]),
            Spans::from(vec![
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Quit application"),
            ]),
        ];

        let paragraph =
            Paragraph::new(text).block(Block::default().title("Help").borders(Borders::ALL));
        f.render_widget(paragraph, area);
    }

    pub fn draw_pull_request_help<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let text = vec![
            Spans::from(vec![
                Span::styled("ENTER", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Select pull request"),
            ]),
            Spans::from(vec![
                Span::styled("UP", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Move selection cursor up"),
            ]),
            Spans::from(vec![
                Span::styled("DOWN", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Move selection cursor down"),
            ]),
            Spans::from(vec![
                Span::styled("ESCAPE", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Return to repository selection"),
            ]),
            Spans::from(vec![
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Quit application"),
            ]),
        ];

        let paragraph =
            Paragraph::new(text).block(Block::default().title("Help").borders(Borders::ALL));
        f.render_widget(paragraph, area);
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(2),
                    Constraint::Percentage(18),
                    Constraint::Percentage(70),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(f.size());

        let title_area = chunks[0];
        let repo_area = chunks[1];
        let pr_area = chunks[2];
        let help_area = chunks[3];

        self.draw_title(f, title_area);
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
