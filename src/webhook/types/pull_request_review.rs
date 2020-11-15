//! Webhook pull request review types

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::common::{Repository, User};
use super::pull_request::PullRequest;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestReviewAction {
    Submitted,
    Edited,
    Dismissed,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
    Pending,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestReview {
    pub id: usize,
    pub node_id: String,
    pub user: User,
    pub body: String,
    pub commit_id: String,
    pub submitted_at: DateTime<Utc>,
    pub state: PullRequestReviewState,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestReviewEvent {
    pub action: PullRequestReviewAction,
    pub review: PullRequestReview,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub organization: User,
    pub sender: User,
}
