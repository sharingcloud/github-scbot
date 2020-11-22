//! Database models.

use diesel::SqliteConnection;

mod pull_request;
mod repository;

pub type DbConn = SqliteConnection;

pub use pull_request::{CheckStatus, PullRequestCreation, PullRequestModel};
pub use repository::{RepositoryCreation, RepositoryModel};
