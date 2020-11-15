//! Webhook pull request types

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::common::{Branch, Label, Repository, User};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestAction {
    Opened,
    Edited,
    Closed,
    Assigned,
    Unassigned,
    ReviewRequested,
    ReviewRequestRemoved,
    ReadyForReview,
    Labeled,
    Unlabeled,
    Synchronize,
    Locked,
    Unlocked,
    Reopened,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub id: u32,
    pub node_id: String,
    pub number: u32,
    pub state: PullRequestState,
    pub locked: bool,
    pub title: String,
    pub user: User,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub requested_reviewers: Vec<User>,
    pub labels: Vec<Label>,
    pub draft: bool,
    pub head: Branch,
    pub base: Branch,
    pub merged: Option<bool>,
    pub mergeable: Option<bool>,
    pub rebaseable: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
    pub action: PullRequestAction,
    pub number: u32,
    pub pull_request: PullRequest,
    pub label: Option<Label>,
    pub requested_reviewer: Option<User>,
    pub repository: Repository,
    pub organization: User,
    pub sender: User,
}
