//! Push types.

use serde::Deserialize;

use super::common::{GHCommit, GHCommitUser, GHRepository};

/// GitHub Push event.
#[derive(Debug, Deserialize)]
pub struct GHPushEvent {
    /// Reference.
    #[serde(rename = "ref")]
    pub reference: String,
    /// Before.
    pub before: String,
    /// After.
    pub after: String,
    /// Repository.
    pub repository: GHRepository,
    /// Pusher.
    pub pusher: GHCommitUser,
    /// Created.
    pub created: bool,
    /// Deleted.
    pub deleted: bool,
    /// Forced.
    pub forced: bool,
    /// Base reference.
    pub base_ref: Option<String>,
    /// Commits.
    pub commits: Vec<GHCommit>,
    /// Head commit.
    pub head_commit: Option<GHCommit>,
}
