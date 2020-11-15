//! Webhook common types

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u32,
    pub node_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CommitUser {
    pub name: String,
    pub email: String,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub id: String,
    pub tree_id: String,
    pub distinct: bool,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub author: CommitUser,
    pub committer: CommitUser,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Branch {
    pub label: Option<String>,
    #[serde(rename = "ref")]
    pub reference: String,
    pub sha: String,
    pub user: Option<User>,
}

#[derive(Debug, Deserialize)]
pub struct BranchShort {
    #[serde(rename = "ref")]
    pub reference: String,
    pub sha: String,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub id: u32,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub owner: User,
    pub html_url: String,
    pub description: String,
    pub fork: bool,
    pub size: usize,
    pub language: String,
    pub default_branch: String,
}

#[derive(Debug, Deserialize)]
pub struct Label {
    pub id: u32,
    pub node_id: String,
    pub name: String,
    pub color: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Application {
    pub id: u32,
    pub slug: String,
    pub node_id: String,
    pub owner: User,
    pub name: String,
    pub description: String,
}
