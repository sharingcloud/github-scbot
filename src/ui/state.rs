//! UI state utils.

use termion::event::Key;
use tui::widgets::ListState;

use crate::database::models::{PullRequestModel, RepositoryModel};

pub enum SelectionMode {
    Repository,
    PullRequest,
}

pub struct AppState {
    pub repositories_state: ListState,
    pub pull_requests_state: ListState,
    pub data: Vec<(RepositoryModel, Vec<PullRequestModel>)>,
    pub selection_mode: SelectionMode,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            repositories_state: ListState::default(),
            pull_requests_state: ListState::default(),
            data: Vec::new(),
            selection_mode: SelectionMode::Repository,
        }
    }

    pub fn with_items(data: Vec<(RepositoryModel, Vec<PullRequestModel>)>) -> Self {
        Self {
            repositories_state: ListState::default(),
            pull_requests_state: ListState::default(),
            data,
            selection_mode: SelectionMode::Repository,
        }
    }

    pub fn next_repository(&mut self) {
        let i = match self.repositories_state.selected() {
            Some(i) => {
                if i >= self.data.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.pull_requests_state.select(None);
        self.repositories_state.select(Some(i));
    }

    pub fn previous_repository(&mut self) {
        let i = match self.repositories_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.data.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.pull_requests_state.select(None);
        self.repositories_state.select(Some(i));
    }

    pub fn pull_requests_for_repository(&self) -> Vec<&PullRequestModel> {
        self.repositories_state
            .selected()
            .map_or_else(Vec::new, |k| self.data[k].1.iter().collect())
    }

    pub fn next_pull_request(&mut self) {
        if let Some(i) = self.repositories_state.selected() {
            let values = &self.data[i].1;
            if values.is_empty() {
                self.pull_requests_state.select(None)
            } else {
                let j = match self.pull_requests_state.selected() {
                    Some(j) => {
                        if j >= self.data[i].1.len() - 1 {
                            0
                        } else {
                            j + 1
                        }
                    }
                    None => 0,
                };
                self.pull_requests_state.select(Some(j));
            }
        }
    }

    pub fn previous_pull_request(&mut self) {
        if let Some(i) = self.repositories_state.selected() {
            let values = &self.data[i].1;
            if values.is_empty() {
                self.pull_requests_state.select(None)
            } else {
                let j = match self.pull_requests_state.selected() {
                    Some(j) => {
                        if j == 0 {
                            self.data[i].1.len() - 1
                        } else {
                            j - 1
                        }
                    }
                    None => 0,
                };
                self.pull_requests_state.select(Some(j));
            }
        }
    }

    pub fn unselect_value(&mut self) {
        self.pull_requests_state.select(None);
    }

    pub fn on_ui_key(&mut self, key: Key) {
        match key {
            Key::Char(c) => {
                if c == '\n'
                    && matches!(self.selection_mode, SelectionMode::Repository)
                    && !self.pull_requests_for_repository().is_empty()
                {
                    self.selection_mode = SelectionMode::PullRequest;
                    self.pull_requests_state.select(Some(0));
                }
            }
            Key::Esc => {
                self.selection_mode = SelectionMode::Repository;
                self.unselect_value();
            }
            Key::Up => match self.selection_mode {
                SelectionMode::Repository => {
                    self.previous_repository();
                }
                SelectionMode::PullRequest => {
                    self.previous_pull_request();
                }
            },
            Key::Down => match self.selection_mode {
                SelectionMode::Repository => {
                    self.next_repository();
                }
                SelectionMode::PullRequest => {
                    self.next_pull_request();
                }
            },
            _ => (),
        }
    }
}
