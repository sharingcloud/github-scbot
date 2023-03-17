use serde::{Deserialize, Serialize};

/// GitHub Issue comment action.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum GhIssueCommentAction {
    /// Created.
    #[default]
    Created,
    /// Edited.
    Edited,
    /// Deleted.
    Deleted,
}
