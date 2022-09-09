//! Pull types.

use snafu::prelude::*;
use std::{convert::TryFrom, str::FromStr};
use time::OffsetDateTime;

use serde::{Deserialize, Serialize};
use serde_plain;
use smart_default::SmartDefault;

use super::common::{GhBranch, GhBranchShort, GhLabel, GhRepository, GhUser};
use super::errors::{TypeError, UnknownMergeStrategySnafu};

/// GitHub Merge strategy.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GhMergeStrategy {
    /// Merge
    Merge,
    /// Squash
    Squash,
    /// Rebase
    Rebase,
}

impl std::fmt::Display for GhMergeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_plain::to_string(&self).unwrap())
    }
}

impl FromStr for GhMergeStrategy {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<&str> for GhMergeStrategy {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_plain::from_str(value).context(UnknownMergeStrategySnafu {
            strategy: value.to_string(),
        })
    }
}

impl TryFrom<&String> for GhMergeStrategy {
    type Error = TypeError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl Default for GhMergeStrategy {
    fn default() -> Self {
        Self::Merge
    }
}

/// GitHub Pull request action.
#[derive(Debug, Deserialize, Serialize, SmartDefault, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GhPullRequestAction {
    /// Assigned.
    #[default]
    Assigned,
    /// Closed.
    Closed,
    /// Converted to draft.
    ConvertedToDraft,
    /// Edited.
    Edited,
    /// Labeled.
    Labeled,
    /// Locked.
    Locked,
    /// Opened.
    Opened,
    /// Reopened.
    Reopened,
    /// Ready for review.
    ReadyForReview,
    /// Review requested.
    ReviewRequested,
    /// Review request removed.
    ReviewRequestRemoved,
    /// Synchronize.
    Synchronize,
    /// Unassigned.
    Unassigned,
    /// Unlabeled.
    Unlabeled,
    /// Unlocked.
    Unlocked,
}

/// GitHub Pull request state.
#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GhPullRequestState {
    /// Open.
    #[default]
    Open,
    /// Closed.
    Closed,
    /// Merged.
    Merged,
}

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

/// GitHub Pull request event.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Eq, PartialEq)]
pub struct GhPullRequestEvent {
    /// Action.
    pub action: GhPullRequestAction,
    /// Number.
    pub number: u64,
    /// Pull request.
    pub pull_request: GhPullRequest,
    /// Label.
    pub label: Option<GhLabel>,
    /// Requested reviewer.
    pub requested_reviewer: Option<GhUser>,
    /// Repository.
    pub repository: GhRepository,
    /// Organization.
    pub organization: Option<GhUser>,
    /// Sender.
    pub sender: GhUser,
}
