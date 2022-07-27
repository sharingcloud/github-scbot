//! Check types.

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use time::OffsetDateTime;

use super::{
    common::{GhApplication, GhRepository, GhUser},
    pulls::GhPullRequestShort,
};

/// GitHub Check suite action.
#[derive(Debug, Deserialize, Serialize, SmartDefault, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GhCheckSuiteAction {
    /// Completed.
    #[default]
    Completed,
    /// Requested.
    Requested,
    /// Re-requested.
    Rerequested,
}

/// GitHub Check status.
#[derive(Debug, Deserialize, Serialize, Copy, Clone, SmartDefault, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GhCheckStatus {
    /// Completed.
    #[default]
    Completed,
    /// In progress.
    InProgress,
    /// Queued.
    Queued,
    /// Requested.
    Requested,
    /// Pending.
    Pending,
}

/// GitHub Check conclusion.
#[derive(Debug, Deserialize, Serialize, SmartDefault, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GhCheckConclusion {
    /// Action required.
    ActionRequired,
    /// Cancelled.
    Cancelled,
    /// Failure.
    Failure,
    /// Neutral.
    Neutral,
    /// Skipped.
    Skipped,
    /// Stale.
    Stale,
    /// Startup failure.
    StartupFailure,
    /// Success.
    #[default]
    Success,
    /// Timed out.
    TimedOut,
}

/// GitHub Check suite.
#[derive(Debug, Deserialize, Serialize, SmartDefault, PartialEq, Clone)]
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
    pub created_at: OffsetDateTime,
    /// Updated at.
    #[default(OffsetDateTime::now_utc())]
    pub updated_at: OffsetDateTime,
}

/// GitHub Check suite event.
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct GhCheckSuiteEvent {
    /// Action.
    pub action: GhCheckSuiteAction,
    /// Check suite.
    pub check_suite: GhCheckSuite,
    /// Repository.
    pub repository: GhRepository,
    /// Organization.
    pub organization: Option<GhUser>,
    /// Sender.
    pub sender: GhUser,
}
