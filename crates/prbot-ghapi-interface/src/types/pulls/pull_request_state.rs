use serde::{Deserialize, Serialize};

/// GitHub Pull request state.
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GhPullRequestState {
    /// Open.
    #[default]
    Open,
    /// Closed.
    Closed,
    /// Merged.
    Merged,
}
