use serde::{Deserialize, Serialize};

/// GitHub Issue state.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GhIssueState {
    /// Open.
    #[default]
    Open,
    /// Closed.
    Closed,
}
