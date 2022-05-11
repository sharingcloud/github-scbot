//! Logic errors.

use snafu::{prelude::*, Backtrace};

/// Logic error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
pub enum LogicError {
    /// Wraps [`regex::Error`].
    #[snafu(display("Error while compiling regex,\n  caused by: {}", source))]
    RegexError {
        source: regex::Error,
        backtrace: Backtrace,
    },

    /// Wraps [`github_scbot_ghapi::ApiError`].
    #[snafu(display("API error,\n  caused by: {}", source))]
    ApiError {
        #[snafu(backtrace)]
        source: github_scbot_ghapi::ApiError,
    },

    /// Wraps [`github_scbot_database2::DatabaseError`].
    #[snafu(display("Database error,\n  caused by: {}", source))]
    DatabaseError {
        #[snafu(backtrace)]
        source: github_scbot_database2::DatabaseError,
    },

    /// Wraps [`github_scbot_redis::RedisError`].
    #[snafu(display("Redis error,\n  caused by: {}", source))]
    RedisError {
        #[snafu(backtrace)]
        source: github_scbot_redis::RedisError,
    },
}

impl From<regex::Error> for LogicError {
    fn from(e: regex::Error) -> Self {
        Self::RegexError {
            source: e,
            backtrace: Backtrace::new(),
        }
    }
}

impl From<github_scbot_ghapi::ApiError> for LogicError {
    fn from(e: github_scbot_ghapi::ApiError) -> Self {
        Self::ApiError { source: e }
    }
}

impl From<github_scbot_database2::DatabaseError> for LogicError {
    fn from(e: github_scbot_database2::DatabaseError) -> Self {
        Self::DatabaseError { source: e }
    }
}

impl From<github_scbot_redis::RedisError> for LogicError {
    fn from(e: github_scbot_redis::RedisError) -> Self {
        Self::RedisError { source: e }
    }
}

/// Result alias for `LogicError`.
pub type Result<T> = core::result::Result<T, LogicError>;
