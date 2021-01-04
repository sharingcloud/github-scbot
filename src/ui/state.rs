//! UI state utils

use std::collections::HashMap;

use termion::event::Key;
use tui::widgets::ListState;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub enum MapListSelectionMode {
    Key,
    Value
}

pub struct StatefulMapList<K, V> {
    pub key_state: ListState,
    pub value_state: ListState,
    pub keyvalues: Vec<(K, Vec<V>)>,
    pub selection_mode: MapListSelectionMode
}

impl<'a, K, V> StatefulMapList<K, V> {
    pub fn new() -> Self {
        Self {
            key_state: ListState::default(),
            value_state: ListState::default(),
            keyvalues: Vec::new(),
            selection_mode: MapListSelectionMode::Key
        }
    }

    pub fn with_items(keyvalues: Vec<(K, Vec<V>)>) -> Self {
        Self {
            key_state: ListState::default(),
            value_state: ListState::default(),
            keyvalues,
            selection_mode: MapListSelectionMode::Key
        }
    }

    pub fn next_key(&mut self) {
        let i = match self.key_state.selected() {
            Some(i) => {
                if i >= self.keyvalues.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.key_state.select(Some(i));
    }

    pub fn previous_key(&mut self) {
        let i = match self.key_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.keyvalues.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.key_state.select(Some(i));
    }

    pub fn next_value(&mut self) {
        match self.key_state.selected() {
            Some(i) => {
                let j = match self.value_state.selected() {
                    Some(j) => {
                        if j >= self.keyvalues[i].1.len() - 1 {
                            0
                        } else {
                            j + 1
                        }
                    }
                    None => 0,
                };
                self.value_state.select(Some(j));
            }
            None => ()
        }
    }

    pub fn previous_value(&mut self) {
        todo!()
    }

    pub fn unselect_key(&mut self) {
        self.key_state.select(None);
    }

    pub fn unselect_value(&mut self) {
        self.value_state.select(None);
    }

    pub fn on_ui_key(&mut self, key: Key) {
        match key {
            Key::Char(c) =>
                match c {
                    '\n' => {
                        self.selection_mode = match self.selection_mode {
                            MapListSelectionMode::Key => MapListSelectionMode::Value,
                            MapListSelectionMode::Value => MapListSelectionMode::Key
                        }
                    },
                    _ => ()
                }
            _ => ()
        }
    }
}
