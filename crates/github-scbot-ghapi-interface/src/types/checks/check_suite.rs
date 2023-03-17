use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use time::OffsetDateTime;

use super::{GhCheckConclusion, GhCheckStatus};
use crate::types::{common::GhApplication, pulls::GhPullRequestShort};

/// GitHub Check suite.
#[derive(Debug, Deserialize, Serialize, SmartDefault, PartialEq, Eq, Clone)]
pub struct GhCheckSuite {
    /// ID.
    pub id: u64,
    /// Head branch.
    pub head_branch: String,
    /// Head commit SHA.
    pub head_sha: String,
    /// Status.
    pub status: GhCheckStatus,
    /// Conclusion.
    pub conclusion: Option<GhCheckConclusion>,
    /// Pull requests.
    pub pull_requests: Vec<GhPullRequestShort>,
    /// Application.
    pub app: GhApplication,
    /// Created at.
    #[default(OffsetDateTime::now_utc())]
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Updated at.
    #[default(OffsetDateTime::now_utc())]
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
