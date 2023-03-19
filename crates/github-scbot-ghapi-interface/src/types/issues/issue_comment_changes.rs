use serde::{Deserialize, Serialize};

/// GitHub Issue comment changes body.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct GhIssueCommentChangesBody {
    /// From.
    pub from: String,
}

/// GitHub Issue comment changes.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct GhIssueCommentChanges {
    /// Body.
    pub body: GhIssueCommentChangesBody,
}
