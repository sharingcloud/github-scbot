//! Logic errors.

use thiserror::Error;

/// Logic error.
#[derive(Debug, Error)]
pub enum LogicError {
    /// Wraps [`regex::Error`].
    #[error("Regex error.")]
    RegexError(#[from] regex::Error),

    /// Wraps [`github_scbot_api::APIError`].
    #[error(transparent)]
    APIError(#[from] github_scbot_api::APIError),

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] github_scbot_database::DatabaseError),
}

/// Result alias for `LogicError`.
pub type Result<T> = core::result::Result<T, LogicError>;
