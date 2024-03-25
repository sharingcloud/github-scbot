use serde::{Deserialize, Serialize};

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
