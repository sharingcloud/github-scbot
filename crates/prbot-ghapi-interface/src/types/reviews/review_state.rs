use std::fmt::Display;

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

impl Display for GhReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_plain::to_string(&self).unwrap())
    }
}

impl From<&str> for GhReviewState {
    fn from(input: &str) -> Self {
        serde_plain::from_str(input).unwrap()
    }
}
