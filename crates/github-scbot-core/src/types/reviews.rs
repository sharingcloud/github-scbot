//! Review types.

use serde::{Deserialize, Serialize};
use serde_plain;
use smart_default::SmartDefault;
use time::OffsetDateTime;

use super::common::{GhRepository, GhUser};
use super::pulls::GhPullRequest;

/// GitHub Review action.
#[derive(Debug, Deserialize, Serialize, SmartDefault, PartialEq)]
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
#[derive(Debug, Deserialize, Serialize, PartialEq, SmartDefault, Eq, Clone, Copy)]
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
#[derive(Debug, Deserialize, Serialize, SmartDefault, PartialEq)]
pub struct GhReview {
    /// User.
    pub user: GhUser,
    /// Submitted at.
    pub submitted_at: Option<OffsetDateTime>,
    /// State.
    pub state: GhReviewState,
}

/// GitHub Review event.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
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
