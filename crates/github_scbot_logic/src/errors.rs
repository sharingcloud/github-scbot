//! Logic errors.

use thiserror::Error;

/// Logic error.
#[derive(Debug, Error)]
pub enum LogicError {
    /// Wraps [`regex::Error`].
    #[error("Error while compiling regex.")]
    RegexError(#[from] regex::Error),

    /// Wraps [`github_scbot_ghapi::ApiError`].
    #[error("API error.")]
    ApiError(#[from] github_scbot_ghapi::ApiError),

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error("Database error.")]
    DatabaseError(#[from] github_scbot_database::DatabaseError),

    /// Wraps [`github_scbot_redis::RedisError`].
    #[error("Redis error.")]
    RedisError(#[from] github_scbot_redis::RedisError),
}

/// Result alias for `LogicError`.
pub type Result<T> = core::result::Result<T, LogicError>;
