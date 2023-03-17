use serde::Deserialize;
use time::OffsetDateTime;

use super::GhCommitUser;

/// GitHub Commit.
#[derive(Debug, Deserialize)]
pub struct GhCommit {
    /// Message.
    pub message: String,
    /// Timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    /// Author.
    pub author: GhCommitUser,
    /// Committer.
    pub committer: GhCommitUser,
    /// Added.
    pub added: Vec<String>,
    /// Removed.
    pub removed: Vec<String>,
    /// Modified.
    pub modified: Vec<String>,
}
