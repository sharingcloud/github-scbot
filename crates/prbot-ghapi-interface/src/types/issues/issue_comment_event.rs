use serde::{Deserialize, Serialize};

use super::{GhIssue, GhIssueComment, GhIssueCommentAction, GhIssueCommentChanges};
use crate::types::common::{GhRepository, GhUser};

/// GitHub Issue comment event.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct GhIssueCommentEvent {
    /// Action.
    pub action: GhIssueCommentAction,
    /// Changes.
    pub changes: Option<GhIssueCommentChanges>,
    /// Issue.
    pub issue: GhIssue,
    /// Comment.
    pub comment: GhIssueComment,
    /// Repository.
    pub repository: GhRepository,
    /// Organization.
    pub organization: Option<GhUser>,
    /// Sender.
    pub sender: GhUser,
}
