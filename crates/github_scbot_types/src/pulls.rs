//! Pull types.

use std::convert::TryFrom;

use chrono::{self, DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_plain;
use smart_default::SmartDefault;

use super::common::{GhBranch, GhBranchShort, GhLabel, GhRepository, GhUser};
use crate::errors::TypeError;

/// GitHub Merge strategy.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GhMergeStrategy {
    /// Merge
    Merge,
    /// Squash
    Squash,
    /// Rebase
    Rebase,
}

impl ToString for GhMergeStrategy {
    fn to_string(&self) -> String {
        serde_plain::to_string(&self).unwrap()
    }
}

impl TryFrom<&str> for GhMergeStrategy {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_plain::from_str(value)
            .map_err(|_e| TypeError::UnknownMergeStrategy(value.to_string()))
    }
}

impl TryFrom<&String> for GhMergeStrategy {
    type Error = TypeError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

/// GitHub Pull request action.
#[derive(Debug, Deserialize, Serialize, SmartDefault, Clone, PartialEq)]
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
#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, PartialEq)]
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
#[derive(Debug, Deserialize, Serialize, Clone, SmartDefault, PartialEq)]
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
    #[default(chrono::Utc::now())]
    pub created_at: DateTime<Utc>,
    /// Updated at.
    #[default(chrono::Utc::now())]
    pub updated_at: DateTime<Utc>,
    /// Closed at.
    pub closed_at: Option<DateTime<Utc>>,
    /// Merged at.
    pub merged_at: Option<DateTime<Utc>>,
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
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Clone)]
pub struct GhPullRequestShort {
    /// Number.
    pub number: u64,
    /// Head branch short format.
    pub head: GhBranchShort,
    /// Base branch short format.
    pub base: GhBranchShort,
}

/// GitHub Pull request event.
#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
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
    pub organization: GhUser,
    /// Sender.
    pub sender: GhUser,
}
