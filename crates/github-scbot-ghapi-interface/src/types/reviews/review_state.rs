use serde::{Deserialize, Serialize};

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
