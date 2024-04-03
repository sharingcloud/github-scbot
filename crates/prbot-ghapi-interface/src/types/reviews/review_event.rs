use serde::{Deserialize, Serialize};

use super::{GhReview, GhReviewAction};
use crate::types::{
    common::{GhRepository, GhUser},
    pulls::GhPullRequest,
};

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
