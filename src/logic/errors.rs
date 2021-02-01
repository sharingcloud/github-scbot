//! Logic errors.

use thiserror::Error;

/// Logic error.
#[derive(Debug, Error)]
pub enum LogicError {
    /// Wraps [`regex::Error`].
    #[error(transparent)]
    RegexError(#[from] regex::Error),

    /// Wraps [`crate::api::APIError`].
    #[error(transparent)]
    APIError(#[from] crate::api::APIError),

    /// Wraps [`crate::database::DatabaseError`].
    #[error(transparent)]
    DatabaseError(#[from] crate::database::DatabaseError),
}

/// Result alias for `LogicError`.
pub type Result<T> = core::result::Result<T, LogicError>;
