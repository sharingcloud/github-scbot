//! Issue types.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::common::{GHLabel, GHRepository, GHUser};

/// GitHub Issue comment action.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GHIssueCommentAction {
    /// Created.
    Created,
    /// Edited.
    Edited,
    /// Deleted.
    Deleted,
}

/// GitHub Issue state.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GHIssueState {
    /// Open.
    Open,
    /// Closed.
    Closed,
}

/// GitHub Issue.
#[derive(Debug, Deserialize)]
pub struct GHIssue {
    /// ID.
    pub id: u64,
    /// Number.
    pub number: u64,
    /// Title.
    pub title: String,
    /// User.
    pub user: GHUser,
    /// Labels.
    pub labels: Vec<GHLabel>,
    /// State.
    pub state: GHIssueState,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Closed at.
    pub closed_at: Option<DateTime<Utc>>,
    /// Body.
    pub body: String,
}

/// GitHub Issue comment changes body.
#[derive(Debug, Deserialize)]
pub struct GHIssueCommentChangesBody {
    /// From.
    pub from: String,
}

/// GitHub Issue comment changes.
#[derive(Debug, Deserialize)]
pub struct GHIssueCommentChanges {
    /// Body.
    pub body: GHIssueCommentChangesBody,
}

/// GitHub Issue comment.
#[derive(Debug, Deserialize)]
pub struct GHIssueComment {
    /// ID.
    pub id: u64,
    /// User.
    pub user: GHUser,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Body.
    pub body: String,
}

/// GitHub Issue comment event.
#[derive(Debug, Deserialize)]
pub struct GHIssueCommentEvent {
    /// Action.
    pub action: GHIssueCommentAction,
    /// Changes.
    pub changes: Option<GHIssueCommentChanges>,
    /// Issue.
    pub issue: GHIssue,
    /// Comment.
    pub comment: GHIssueComment,
    /// Repository.
    pub repository: GHRepository,
    /// Organization.
    pub organization: GHUser,
    /// Sender.
    pub sender: GHUser,
}
