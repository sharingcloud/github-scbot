//! Database models.

use diesel::PgConnection;

mod pull_request;
mod repository;

pub type DbConn = PgConnection;

pub use pull_request::{CheckStatus, PullRequestCreation, PullRequestModel};
pub use repository::{RepositoryCreation, RepositoryModel};
