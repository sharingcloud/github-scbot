//! Pull types.

use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::{GHBranch, GHBranchShort, GHLabel, GHRepository, GHUser};
use crate::errors::TypeError;

/// GitHub Merge strategy.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum GHMergeStrategy {
    /// Merge
    Merge,
    /// Squash
    Squash,
    /// Rebase
    Rebase,
}

impl ToString for GHMergeStrategy {
    fn to_string(&self) -> String {
        serde_plain::to_string(&self).unwrap()
    }
}

impl TryFrom<&str> for GHMergeStrategy {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_plain::from_str(value)
            .map_err(|_e| TypeError::UnknownMergeStrategy(value.to_string()))
    }
}

/// GitHub Pull request action.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GHPullRequestAction {
    /// Assigned.
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
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GHPullRequestState {
    /// Open.
    Open,
    /// Closed.
    Closed,
    /// Merged.
    Merged,
}

/// GitHub Pull request.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GHPullRequest {
    /// Number.
    pub number: u64,
    /// State.
    pub state: GHPullRequestState,
    /// Locked.
    pub locked: bool,
    /// Title.
    pub title: String,
    /// User.
    pub user: GHUser,
    /// Body.
    pub body: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Closed at.
    pub closed_at: Option<DateTime<Utc>>,
    /// Merged at.
    pub merged_at: Option<DateTime<Utc>>,
    /// Requested reviewers.
    pub requested_reviewers: Vec<GHUser>,
    /// Labels.
    pub labels: Vec<GHLabel>,
    /// Draft.
    pub draft: bool,
    /// Head branch.
    pub head: GHBranch,
    /// Base branch.
    pub base: GHBranch,
    /// Merged?
    pub merged: Option<bool>,
    /// Mergeable?
    pub mergeable: Option<bool>,
    /// Rebaseable?
    pub rebaseable: Option<bool>,
}

/// GitHub Pull request short format.
#[derive(Debug, Deserialize, Serialize)]
pub struct GHPullRequestShort {
    /// Number.
    pub number: u64,
    /// Head branch short format.
    pub head: GHBranchShort,
    /// Base branch short format.
    pub base: GHBranchShort,
}

/// GitHub Pull request event.
#[derive(Debug, Deserialize, Serialize)]
pub struct GHPullRequestEvent {
    /// Action.
    pub action: GHPullRequestAction,
    /// Number.
    pub number: u64,
    /// Pull request.
    pub pull_request: GHPullRequest,
    /// Label.
    pub label: Option<GHLabel>,
    /// Requested reviewer.
    pub requested_reviewer: Option<GHUser>,
    /// Repository.
    pub repository: GHRepository,
    /// Organization.
    pub organization: GHUser,
    /// Sender.
    pub sender: GHUser,
}
