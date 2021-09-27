//! Issue types.

use chrono::{self, DateTime, Utc};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use super::common::{GhLabel, GhRepository, GhUser};

/// GitHub Reaction type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GhReactionType {
    /// 👍
    PlusOne,
    /// 👎
    MinusOne,
    /// 😄
    Laugh,
    /// 😕
    Confused,
    /// ❤️
    Heart,
    /// 🎉
    Hooray,
    /// 🚀
    Rocket,
    /// 👀
    Eyes,
}

impl GhReactionType {
    /// Convert reaction type to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl From<GhReactionType> for &'static str {
    fn from(reaction_type: GhReactionType) -> &'static str {
        match reaction_type {
            GhReactionType::PlusOne => "+1",
            GhReactionType::MinusOne => "-1",
            GhReactionType::Laugh => "laugh",
            GhReactionType::Confused => "confused",
            GhReactionType::Heart => "heart",
            GhReactionType::Hooray => "hooray",
            GhReactionType::Rocket => "rocket",
            GhReactionType::Eyes => "eyes",
        }
    }
}

/// GitHub Issue comment action.
#[derive(Debug, Deserialize, Serialize, PartialEq, SmartDefault)]
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

/// GitHub Issue state.
#[derive(Debug, Deserialize, Serialize, SmartDefault, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GhIssueState {
    /// Open.
    #[default]
    Open,
    /// Closed.
    Closed,
}

/// GitHub Issue.
#[derive(Debug, Deserialize, Serialize, PartialEq, SmartDefault)]
pub struct GhIssue {
    /// Number.
    pub number: u64,
    /// Title.
    pub title: String,
    /// User.
    pub user: GhUser,
    /// Labels.
    pub labels: Vec<GhLabel>,
    /// State.
    pub state: GhIssueState,
    /// Created at.
    #[default(chrono::Utc::now())]
    pub created_at: DateTime<Utc>,
    /// Updated at.
    #[default(chrono::Utc::now())]
    pub updated_at: DateTime<Utc>,
    /// Closed at.
    pub closed_at: Option<DateTime<Utc>>,
    /// Body.
    pub body: Option<String>,
}

/// GitHub Issue comment changes body.
#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct GhIssueCommentChangesBody {
    /// From.
    pub from: String,
}

/// GitHub Issue comment changes.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct GhIssueCommentChanges {
    /// Body.
    pub body: GhIssueCommentChangesBody,
}

/// GitHub Issue comment.
#[derive(Debug, Deserialize, Serialize, PartialEq, SmartDefault)]
pub struct GhIssueComment {
    /// ID.
    pub id: u64,
    /// User.
    pub user: GhUser,
    /// Created at.
    #[default(chrono::Utc::now())]
    pub created_at: DateTime<Utc>,
    /// Updated at.
    #[default(chrono::Utc::now())]
    pub updated_at: DateTime<Utc>,
    /// Body.
    pub body: String,
}

/// GitHub Issue comment event.
#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
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
    pub organization: GhUser,
    /// Sender.
    pub sender: GhUser,
}
