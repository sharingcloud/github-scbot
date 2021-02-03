//! Pull request types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::{GHBranch, GHBranchShort, GHLabel, GHRepository, GHUser};

/// GitHub Pull request action.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GHPullRequestState {
    /// Open.
    Open,
    /// Closed.
    Closed,
    /// Merged.
    Merged,
}

/// GitHub Pull request review action.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GHPullRequestReviewAction {
    /// Submitted.
    Submitted,
    /// Edited.
    Edited,
    /// Dismissed.
    Dismissed,
}

/// GitHub Pull request review comment action.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GHPullRequestReviewCommentAction {
    /// Created.
    Created,
    /// Edited.
    Edited,
    /// Deleted.
    Deleted,
}

/// GitHub Pull request review state.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GHPullRequestReviewState {
    /// Approved.
    Approved,
    /// Changes requested.
    ChangesRequested,
    /// Commented.
    Commented,
    /// Dismissed.
    Dismissed,
    /// Pending.
    Pending,
}

impl ToString for GHPullRequestReviewState {
    fn to_string(&self) -> String {
        serde_plain::to_string(&self).unwrap()
    }
}

impl From<&str> for GHPullRequestReviewState {
    fn from(input: &str) -> Self {
        serde_plain::from_str(input).unwrap()
    }
}

/// GitHub Pull request review.
#[derive(Debug, Deserialize)]
pub struct GHPullRequestReview {
    /// ID.
    pub id: u64,
    /// User.
    pub user: GHUser,
    /// Body.
    pub body: String,
    /// Commit ID.
    pub commit_id: String,
    /// Submitted at.
    pub submitted_at: DateTime<Utc>,
    /// State.
    pub state: GHPullRequestReviewState,
}

/// GitHub Pull request.
#[derive(Debug, Deserialize)]
pub struct GHPullRequest {
    /// ID.
    pub id: u64,
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
#[derive(Debug, Deserialize)]
pub struct GHPullRequestShort {
    /// ID.
    pub id: u64,
    /// Number.
    pub number: u64,
    /// Head branch short format.
    pub head: GHBranchShort,
    /// Base branch short format.
    pub base: GHBranchShort,
}

/// GitHub Pull request review comment.
#[derive(Debug, Deserialize)]
pub struct GHPullRequestReviewComment {
    /// Review ID.
    pub pull_request_review_id: u64,
    /// ID.
    pub id: u64,
    /// Diff hunk.
    pub diff_hunk: String,
    /// Path.
    pub path: String,
    /// Position.
    pub position: usize,
    /// Original position.
    pub original_position: usize,
    /// Commit ID.
    pub commit_id: String,
    /// Original commit ID.
    pub original_commit_id: String,
    /// User.
    pub user: GHUser,
    /// Body.
    pub body: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Line.
    pub line: usize,
    /// Original line.
    pub original_line: usize,
}

/// GitHub Pull request event.
#[derive(Debug, Deserialize)]
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

/// GitHub Pull request review event.
#[derive(Debug, Deserialize)]
pub struct GHPullRequestReviewEvent {
    /// Action.
    pub action: GHPullRequestReviewAction,
    /// Review.
    pub review: GHPullRequestReview,
    /// Pull request.
    pub pull_request: GHPullRequest,
    /// Repository.
    pub repository: GHRepository,
    /// Organization.
    pub organization: GHUser,
    /// Sender.
    pub sender: GHUser,
}

/// GitHub Pull request review comment event.
#[derive(Debug, Deserialize)]
pub struct GHPullRequestReviewCommentEvent {
    /// Action.
    pub action: GHPullRequestReviewCommentAction,
    /// Comment.
    pub comment: GHPullRequestReviewComment,
    /// Pull request.
    pub pull_request: GHPullRequest,
    /// Repository.
    pub repository: GHRepository,
    /// Organization.
    pub organization: GHUser,
    /// Sender.
    pub sender: GHUser,
}
