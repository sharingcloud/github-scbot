//! Webhook issues types

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::common::{Label, Repository, User};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCommentAction {
    Created,
    Edited,
    Deleted,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueState {
    Open,
    Closed,
}

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub id: u32,
    pub node_id: String,
    pub number: u32,
    pub title: String,
    pub user: User,
    pub labels: Vec<Label>,
    pub state: IssueState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct IssueCommentChangesBody {
    pub from: String,
}

#[derive(Debug, Deserialize)]
pub struct IssueCommentChanges {
    pub body: IssueCommentChangesBody,
}

#[derive(Debug, Deserialize)]
pub struct IssueComment {
    pub id: u32,
    pub node_id: String,
    pub user: User,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct IssueCommentEvent {
    pub action: IssueCommentAction,
    pub changes: Option<IssueCommentChanges>,
    pub issue: Issue,
    pub comment: IssueComment,
    pub repository: Repository,
    pub organization: User,
    pub sender: User,
}
