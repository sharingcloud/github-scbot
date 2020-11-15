//! Webhook push types

use super::common::{Commit, CommitUser, Repository};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PushEvent {
    #[serde(rename = "ref")]
    pub reference: String,
    pub before: String,
    pub after: String,
    pub repository: Repository,
    pub pusher: CommitUser,
    pub created: bool,
    pub deleted: bool,
    pub forced: bool,
    pub base_ref: Option<String>,
    pub commits: Vec<Commit>,
    pub head_commit: Option<Commit>,
}
