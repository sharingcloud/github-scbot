use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use time::OffsetDateTime;

use super::{GhCheckConclusion, GhCheckStatus};
use crate::types::{common::GhApplication, pulls::GhPullRequestShort};

/// GitHub Check run.
#[derive(Debug, Deserialize, Serialize, SmartDefault, PartialEq, Eq, Clone)]
pub struct GhCheckRun {
    /// ID.
    pub id: u64,
    /// Name
    pub name: String,
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
    pub started_at: OffsetDateTime,
    /// Updated at.
    #[serde(with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>,
}
