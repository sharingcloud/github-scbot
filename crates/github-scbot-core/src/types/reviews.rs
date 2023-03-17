//! Review types.

use serde::{Deserialize, Serialize};
use serde_plain;
use time::OffsetDateTime;

use super::{
    common::{GhRepository, GhUser},
    pulls::GhPullRequest,
};

/// GitHub Review action.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GhReviewAction {
    /// Submitted.
    #[default]
    Submitted,
    /// Edited.
    Edited,
    /// Dismissed.
    Dismissed,
}

/// GitHub Review state.
#[derive(Debug, Deserialize, Serialize, PartialEq, Default, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum GhReviewState {
    /// Approved.
    #[default]
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

impl ToString for GhReviewState {
    fn to_string(&self) -> String {
        serde_plain::to_string(&self).unwrap()
    }
}

impl From<&str> for GhReviewState {
    fn from(input: &str) -> Self {
        serde_plain::from_str(input).unwrap()
    }
}

/// GitHub Review.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GhReview {
    /// User.
    pub user: GhUser,
    /// Submitted at.
    #[serde(with = "time::serde::rfc3339::option")]
    pub submitted_at: Option<OffsetDateTime>,
    /// State.
    pub state: GhReviewState,
}

/// GitHub Review event.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GhReviewEvent {
    /// Action.
    pub action: GhReviewAction,
    /// Review.
    pub review: GhReview,
    /// Pull request.
    pub pull_request: GhPullRequest,
    /// Repository.
    pub repository: GhRepository,
    /// Organization.
    pub organization: Option<GhUser>,
    /// Sender.
    pub sender: GhUser,
}
