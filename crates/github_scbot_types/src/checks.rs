//! Check types.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::{
    common::{GHApplication, GHRepository, GHUser},
    pulls::GHPullRequestShort,
};

/// GitHub Check run action.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GHCheckRunAction {
    /// Completed.
    Completed,
    /// Created.
    Created,
    /// Requested action.
    RequestedAction,
    /// Re-requested.
    Rerequested,
}

/// GitHub Check suite action.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GHCheckSuiteAction {
    /// Completed.
    Completed,
    /// Requested.
    Requested,
    /// Re-requested.
    Rerequested,
}

/// GitHub Check status.
#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GHCheckStatus {
    /// Completed.
    Completed,
    /// In progress.
    InProgress,
    /// Queued.
    Queued,
    /// Requested.
    Requested,
}

/// GitHub Check conclusion.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GHCheckConclusion {
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
    /// Success.
    Success,
    /// Timed out.
    TimedOut,
}

/// GitHub Check run output.
#[derive(Debug, Deserialize)]
pub struct GHCheckRunOutput {
    /// Title.
    pub title: Option<String>,
    /// Summary.
    pub summary: Option<String>,
    /// Text content.
    pub text: Option<String>,
}

/// GitHub Check suite.
#[derive(Debug, Deserialize)]
pub struct GHCheckSuite {
    /// Head branch.
    pub head_branch: String,
    /// Head commit SHA.
    pub head_sha: String,
    /// Status.
    pub status: GHCheckStatus,
    /// Conclusion.
    pub conclusion: Option<GHCheckConclusion>,
    /// Pull requests.
    pub pull_requests: Vec<GHPullRequestShort>,
    /// Application.
    pub app: GHApplication,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
}

/// GitHub Check run.
#[derive(Debug, Deserialize)]
pub struct GHCheckRun {
    /// Head commit SHA.
    pub head_sha: String,
    /// External ID.
    pub external_id: String,
    /// Status.
    pub status: GHCheckStatus,
    /// Conclusion.
    pub conclusion: Option<GHCheckConclusion>,
    /// Started at.
    pub started_at: DateTime<Utc>,
    /// Completed at.
    pub completed_at: Option<DateTime<Utc>>,
    /// Output.
    pub output: GHCheckRunOutput,
    /// Name.
    pub name: String,
    /// Check suite.
    pub check_suite: GHCheckSuite,
    /// Application.
    pub app: GHApplication,
}

/// GitHub Check run event.
#[derive(Debug, Deserialize)]
pub struct GHCheckRunEvent {
    /// Action.
    pub action: GHCheckRunAction,
    /// Check run.
    pub check_run: GHCheckRun,
    /// Repository.
    pub repository: GHRepository,
    /// Organization.
    pub organization: GHUser,
    /// Sender.
    pub sender: GHUser,
}

/// GitHub Check suite event.
#[derive(Debug, Deserialize)]
pub struct GHCheckSuiteEvent {
    /// Action.
    pub action: GHCheckSuiteAction,
    /// Check suite.
    pub check_suite: GHCheckSuite,
    /// Repository.
    pub repository: GHRepository,
    /// Organization.
    pub organization: GHUser,
    /// Sender.
    pub sender: GHUser,
}
