//! Common types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// GitHub User.
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct GhUser {
    /// Username.
    pub login: String,
}

/// GitHub User permission.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GhUserPermission {
    /// Admin.
    Admin,
    /// Write.
    Write,
    /// Read.
    Read,
    /// None.
    None,
}

impl GhUserPermission {
    /// Can write?
    pub fn can_write(&self) -> bool {
        matches!(self, Self::Admin | Self::Write)
    }
}

/// GitHub Commit user.
#[derive(Debug, Deserialize)]
pub struct GhCommitUser {
    /// Name.
    pub name: String,
    /// Email.
    pub email: String,
    /// Username.
    pub username: Option<String>,
}

/// GitHub Commit.
#[derive(Debug, Deserialize)]
pub struct GhCommit {
    /// Message.
    pub message: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
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

/// GitHub Branch.
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct GhBranch {
    /// Label.
    pub label: Option<String>,
    #[serde(rename = "ref")]
    /// Reference.
    pub reference: String,
    /// SHA.
    pub sha: String,
    /// Creator.
    pub user: Option<GhUser>,
}

/// GitHub Branch (short format).
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct GhBranchShort {
    /// Reference.
    #[serde(rename = "ref")]
    pub reference: String,
    /// SHA.
    pub sha: String,
}

/// GitHub Repository.
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct GhRepository {
    /// Name.
    pub name: String,
    /// Full name.
    pub full_name: String,
    /// Owner.
    pub owner: GhUser,
}

/// GitHub Label.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct GhLabel {
    /// Name.
    pub name: String,
    /// Color.
    pub color: String,
    /// Description.
    pub description: Option<String>,
}

/// GitHub Application.
#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct GhApplication {
    /// Slug name.
    pub slug: String,
    /// Owner.
    pub owner: GhUser,
    /// Name.
    pub name: String,
}
