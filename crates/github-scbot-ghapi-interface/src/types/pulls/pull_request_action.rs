use serde::{Deserialize, Serialize};

/// GitHub Pull request action.
#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq, Eq)]
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
