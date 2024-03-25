use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::GhReviewState;
use crate::types::common::GhUser;

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
