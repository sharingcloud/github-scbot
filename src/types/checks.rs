//! Webhook check run types

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::common::{Application, Repository, User};
use super::pull_request::PullRequestShort;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckRunAction {
    Completed,
    Created,
    RequestedAction,
    Rerequested,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckSuiteAction {
    Completed,
    Requested,
    Rerequested,
}

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Completed,
    InProgress,
    Queued,
    Requested,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckConclusion {
    ActionRequired,
    Cancelled,
    Failure,
    Neutral,
    Skipped,
    Stale,
    Success,
    TimedOut,
}

#[derive(Debug, Deserialize)]
pub struct CheckRunOutput {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckSuite {
    pub id: u64,
    pub head_branch: String,
    pub head_sha: String,
    pub status: CheckStatus,
    pub conclusion: Option<CheckConclusion>,
    pub pull_requests: Vec<PullRequestShort>,
    pub app: Application,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CheckRun {
    pub id: u64,
    pub head_sha: String,
    pub external_id: String,
    pub status: CheckStatus,
    pub conclusion: Option<CheckConclusion>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: CheckRunOutput,
    pub name: String,
    pub check_suite: CheckSuite,
    pub app: Application,
}

#[derive(Debug, Deserialize)]
pub struct CheckRunEvent {
    pub action: CheckRunAction,
    pub check_run: CheckRun,
    pub repository: Repository,
    pub organization: User,
    pub sender: User,
}

#[derive(Debug, Deserialize)]
pub struct CheckSuiteEvent {
    pub action: CheckSuiteAction,
    pub check_suite: CheckSuite,
    pub repository: Repository,
    pub organization: User,
    pub sender: User,
}
