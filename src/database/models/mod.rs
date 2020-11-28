//! Database models.

mod pull_request;
mod repository;

pub use super::DbConn;
pub use pull_request::{CheckStatus, PullRequestCreation, PullRequestModel, QAStatus};
pub use repository::{RepositoryCreation, RepositoryModel};
