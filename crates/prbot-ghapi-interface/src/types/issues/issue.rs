use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use time::OffsetDateTime;

use super::GhIssueState;
use crate::types::common::{GhLabel, GhUser};

/// GitHub Issue.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, SmartDefault)]
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
    #[default(OffsetDateTime::now_utc())]
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Updated at.
    #[default(OffsetDateTime::now_utc())]
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    /// Closed at.
    #[serde(with = "time::serde::rfc3339::option")]
    pub closed_at: Option<OffsetDateTime>,
    /// Body.
    pub body: Option<String>,
}
