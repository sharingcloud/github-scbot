use std::collections::HashMap;

use termion::event::Key;

use crate::{database::{DbConn, models::{PullRequestModel, RepositoryModel}}};

use crate::ui::errors::Result;

use super::state::{StatefulMapList};

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub data: StatefulMapList<RepositoryModel, PullRequestModel>
}

impl<'a> App<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            should_quit: false,
            data: StatefulMapList::new()
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

        self.data = StatefulMapList::with_items(pr_kvs);

        Ok(())
    }

    pub fn on_key(&mut self, key: Key) {
        match key {
            Key::Char(c) => {
                match c {
                    'q' => {
                        self.should_quit = true;
                    }
                    _ => {}
                }
            }
            _ => ()
        }
    }

    pub fn on_tick(&mut self) {
        // TICK
    }
}
