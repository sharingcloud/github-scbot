//! Webhook pull request types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::{Branch, BranchShort, Label, Repository, User};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestAction {
    Assigned,
    Closed,
    ConvertedToDraft,
    Edited,
    Labeled,
    Locked,
    Opened,
    Reopened,
    ReadyForReview,
    ReviewRequested,
    ReviewRequestRemoved,
    Synchronize,
    Unassigned,
    Unlabeled,
    Unlocked,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestReviewAction {
    Submitted,
    Edited,
    Dismissed,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestReviewCommentAction {
    Created,
    Edited,
    Deleted,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
    Pending,
}

impl ToString for PullRequestReviewState {
    fn to_string(&self) -> String {
        serde_plain::to_string(&self).unwrap()
    }
}

impl From<&str> for PullRequestReviewState {
    fn from(input: &str) -> Self {
        serde_plain::from_str(input).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct PullRequestReview {
    pub id: u64,
    pub user: User,
    pub body: String,
    pub commit_id: String,
    pub submitted_at: DateTime<Utc>,
    pub state: PullRequestReviewState,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub id: u64,
    pub number: u64,
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
pub struct PullRequestShort {
    pub id: u64,
    pub number: u64,
    pub head: BranchShort,
    pub base: BranchShort,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestReviewComment {
    pub pull_request_review_id: u64,
    pub id: u64,
    pub diff_hunk: String,
    pub path: String,
    pub position: usize,
    pub original_position: usize,
    pub commit_id: String,
    pub original_commit_id: String,
    pub user: User,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub line: usize,
    pub original_line: usize,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
    pub action: PullRequestAction,
    pub number: u64,
    pub pull_request: PullRequest,
    pub label: Option<Label>,
    pub requested_reviewer: Option<User>,
    pub repository: Repository,
    pub organization: User,
    pub sender: User,
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

#[derive(Debug, Deserialize)]
pub struct PullRequestReviewCommentEvent {
    pub action: PullRequestReviewCommentAction,
    pub comment: PullRequestReviewComment,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub organization: User,
    pub sender: User,
}