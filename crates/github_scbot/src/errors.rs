//! Global errors.

use thiserror::Error;

/// Bot error.
#[derive(Debug, Error)]
pub enum BotError {
    /// Wraps [`std::io::Error`].
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    /// Wraps [`github_scbot_api::APIError`].
    #[error(transparent)]
    APIError(#[from] github_scbot_api::APIError),
    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] github_scbot_database::DatabaseError),
    /// Wraps [`github_scbot_server::ServerError`].
    #[error(transparent)]
    ServerError(#[from] github_scbot_server::ServerError),
}

/// Result alias for `BotError`.
pub type Result<T, E = BotError> = core::result::Result<T, E>;
