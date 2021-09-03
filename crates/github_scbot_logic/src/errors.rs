//! Logic errors.

use github_scbot_libs::regex;
use thiserror::Error;

/// Logic error.
#[derive(Debug, Error)]
pub enum LogicError {
    /// Wraps [`regex::Error`].
    #[error("Error while compiling regex.")]
    RegexError(#[from] regex::Error),

    /// Wraps [`github_scbot_api::ApiError`].
    #[error(transparent)]
    ApiError(#[from] github_scbot_api::ApiError),

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] github_scbot_database::DatabaseError),

    /// Wraps [`github_scbot_redis::RedisError`].
    #[error(transparent)]
    RedisError(#[from] github_scbot_redis::RedisError),
}

/// Result alias for `LogicError`.
pub type Result<T> = core::result::Result<T, LogicError>;
