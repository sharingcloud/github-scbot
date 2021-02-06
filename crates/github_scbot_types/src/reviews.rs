//! Review types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::{GHRepository, GHUser};
use crate::pulls::GHPullRequest;

/// GitHub Review action.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GHReviewAction {
    /// Submitted.
    Submitted,
    /// Edited.
    Edited,
    /// Dismissed.
    Dismissed,
}

/// GitHub Review comment action.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GHReviewCommentAction {
    /// Created.
    Created,
    /// Edited.
    Edited,
    /// Deleted.
    Deleted,
}

/// GitHub Review state.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum GHReviewState {
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

impl ToString for GHReviewState {
    fn to_string(&self) -> String {
        serde_plain::to_string(&self).unwrap()
    }
}

impl From<&str> for GHReviewState {
    fn from(input: &str) -> Self {
        serde_plain::from_str(input).unwrap()
    }
}

/// GitHub Review.
#[derive(Debug, Deserialize)]
pub struct GHReview {
    /// User.
    pub user: GHUser,
    /// Submitted at.
    pub submitted_at: DateTime<Utc>,
    /// State.
    pub state: GHReviewState,
}

/// GitHub Review event.
#[derive(Debug, Deserialize)]
pub struct GHReviewEvent {
    /// Action.
    pub action: GHReviewAction,
    /// Review.
    pub review: GHReview,
    /// Pull request.
    pub pull_request: GHPullRequest,
    /// Repository.
    pub repository: GHRepository,
    /// Organization.
    pub organization: GHUser,
    /// Sender.
    pub sender: GHUser,
}

/// GitHub Review comment.
#[derive(Debug, Deserialize)]
pub struct GHReviewComment {
    /// Review ID.
    pub pull_request_review_id: u64,
    /// Diff hunk.
    pub diff_hunk: String,
    /// Path.
    pub path: String,
    /// Position.
    pub position: usize,
    /// Original position.
    pub original_position: usize,
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

/// GitHub Review comment event.
#[derive(Debug, Deserialize)]
pub struct GHReviewCommentEvent {
    /// Action.
    pub action: GHReviewCommentAction,
    /// Comment.
    pub comment: GHReviewComment,
    /// Pull request.
    pub pull_request: GHPullRequest,
    /// Repository.
    pub repository: GHRepository,
    /// Organization.
    pub organization: GHUser,
    /// Sender.
    pub sender: GHUser,
}
