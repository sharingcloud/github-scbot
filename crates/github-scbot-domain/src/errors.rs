//! Logic errors.

use snafu::prelude::*;

/// Logic error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
pub enum DomainError {
    /// Wraps [`regex::Error`].
    #[snafu(display("Error while compiling regex,\n  caused by: {}", source))]
    RegexError { source: regex::Error },

    /// Wraps [`github_scbot_ghapi::ApiError`].
    #[snafu(display("API error,\n  caused by: {}", source))]
    ApiError {
        source: github_scbot_ghapi::ApiError,
    },

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[snafu(display("Database error,\n  caused by: {}", source))]
    DatabaseError {
        source: github_scbot_database::DatabaseError,
    },

    /// Wraps [`github_scbot_redis::RedisError`].
    #[snafu(display("Redis error,\n  caused by: {}", source))]
    RedisError {
        source: github_scbot_redis::RedisError,
    },
}

impl From<regex::Error> for DomainError {
    fn from(e: regex::Error) -> Self {
        Self::RegexError { source: e }
    }
}

impl From<github_scbot_ghapi::ApiError> for DomainError {
    fn from(e: github_scbot_ghapi::ApiError) -> Self {
        Self::ApiError { source: e }
    }
}

impl From<github_scbot_database::DatabaseError> for DomainError {
    fn from(e: github_scbot_database::DatabaseError) -> Self {
        Self::DatabaseError { source: e }
    }
}

impl From<github_scbot_redis::RedisError> for DomainError {
    fn from(e: github_scbot_redis::RedisError) -> Self {
        Self::RedisError { source: e }
    }
}

/// Result alias for `DomainError`.
pub type Result<T> = core::result::Result<T, DomainError>;
