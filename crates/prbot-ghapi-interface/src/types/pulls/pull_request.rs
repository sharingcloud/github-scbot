use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use time::OffsetDateTime;

use super::GhPullRequestState;
use crate::types::common::{GhBranch, GhBranchShort, GhLabel, GhUser};

/// GitHub Pull request.
#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, PartialEq, Eq)]
pub struct GhPullRequest {
    /// Number.
    pub number: u64,
    /// State.
    pub state: GhPullRequestState,
    /// Locked.
    pub locked: bool,
    /// Title.
    pub title: String,
    /// User.
    pub user: GhUser,
    /// Body.
    pub body: Option<String>,
    /// Created at.
    #[default(OffsetDateTime::now_utc())]
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Updated at.
    #[default(OffsetDateTime::now_utc())]
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    /// Closed at.
    #[serde(with = "time::serde::rfc3339::option")]
    pub closed_at: Option<OffsetDateTime>,
    /// Merged at.
    #[serde(with = "time::serde::rfc3339::option")]
    pub merged_at: Option<OffsetDateTime>,
    /// Requested reviewers.
    pub requested_reviewers: Vec<GhUser>,
    /// Labels.
    pub labels: Vec<GhLabel>,
    /// Draft.
    pub draft: bool,
    /// Head branch.
    pub head: GhBranch,
    /// Base branch.
    pub base: GhBranch,
    /// Merged?
    pub merged: Option<bool>,
    /// Mergeable?
    pub mergeable: Option<bool>,
    /// Rebaseable?
    pub rebaseable: Option<bool>,
}

/// GitHub Pull request short format.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq, Clone)]
pub struct GhPullRequestShort {
    /// Number.
    pub number: u64,
    /// Head branch short format.
    pub head: GhBranchShort,
    /// Base branch short format.
    pub base: GhBranchShort,
}
