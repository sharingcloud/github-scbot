//! Common types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// GitHub User.
#[derive(Debug, Deserialize, Serialize)]
pub struct GHUser {
    /// Username.
    pub login: String,
}

/// GitHub Commit user.
#[derive(Debug, Deserialize)]
pub struct GHCommitUser {
    /// Name.
    pub name: String,
    /// Email.
    pub email: String,
    /// Username.
    pub username: Option<String>,
}

/// GitHub Commit.
#[derive(Debug, Deserialize)]
pub struct GHCommit {
    /// Distinct.
    pub distinct: bool,
    /// Message.
    pub message: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Author.
    pub author: GHCommitUser,
    /// Committer.
    pub committer: GHCommitUser,
    /// Added.
    pub added: Vec<String>,
    /// Removed.
    pub removed: Vec<String>,
    /// Modified.
    pub modified: Vec<String>,
}

/// GitHub Branch.
#[derive(Debug, Deserialize, Serialize)]
pub struct GHBranch {
    /// Label.
    pub label: Option<String>,
    #[serde(rename = "ref")]
    /// Reference.
    pub reference: String,
    /// SHA.
    pub sha: String,
    /// Creator.
    pub user: Option<GHUser>,
}

/// GitHub Branch (short format).
#[derive(Debug, Deserialize, Serialize)]
pub struct GHBranchShort {
    /// Reference.
    #[serde(rename = "ref")]
    pub reference: String,
    /// SHA.
    pub sha: String,
}

/// GitHub Repository.
#[derive(Debug, Deserialize, Serialize)]
pub struct GHRepository {
    /// Name.
    pub name: String,
    /// Full name.
    pub full_name: String,
    /// Private?
    pub private: bool,
    /// Owner.
    pub owner: GHUser,
    /// HTML URL.
    pub html_url: String,
    /// Description.
    pub description: String,
    /// Is a fork?
    pub fork: bool,
    /// Language.
    pub language: String,
    /// Default branch name.
    pub default_branch: String,
}

/// GitHub Label.
#[derive(Debug, Deserialize, Serialize)]
pub struct GHLabel {
    /// Name.
    pub name: String,
    /// Color.
    pub color: String,
    /// Description.
    pub description: Option<String>,
}

/// GitHub Application.
#[derive(Debug, Deserialize, Serialize)]
pub struct GHApplication {
    /// Slug name.
    pub slug: String,
    /// Owner.
    pub owner: GHUser,
    /// Name.
    pub name: String,
    /// Description.
    pub description: String,
}
