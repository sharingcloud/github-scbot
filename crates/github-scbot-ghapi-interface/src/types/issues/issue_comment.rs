use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use time::OffsetDateTime;

use crate::types::common::GhUser;

/// GitHub Issue comment.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, SmartDefault)]
pub struct GhIssueComment {
    /// ID.
    pub id: u64,
    /// User.
    pub user: GhUser,
    /// Created at.
    #[default(OffsetDateTime::now_utc())]
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Updated at.
    #[default(OffsetDateTime::now_utc())]
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    /// Body.
    pub body: String,
}
