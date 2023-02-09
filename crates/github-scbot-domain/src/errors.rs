//! Logic errors.

use thiserror::Error;

/// Logic error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum DomainError {
    /// Wraps [`regex::Error`].
    #[error("Error while compiling regex")]
    RegexError { source: regex::Error },

    /// Wraps [`github_scbot_ghapi_interface::ApiError`].
    #[error("API error")]
    ApiError {
        source: github_scbot_ghapi_interface::ApiError,
    },

    /// Wraps [`github_scbot_database::DatabaseError`].
    #[error("Database error")]
    DatabaseError {
        source: github_scbot_database::DatabaseError,
    },

    #[error("Lock error")]
    LockError {
        source: github_scbot_lock_interface::LockError,
    },
}

impl From<regex::Error> for DomainError {
    fn from(e: regex::Error) -> Self {
        Self::RegexError { source: e }
    }
}

impl From<github_scbot_ghapi_interface::ApiError> for DomainError {
    fn from(e: github_scbot_ghapi_interface::ApiError) -> Self {
        Self::ApiError { source: e }
    }
}

impl From<github_scbot_database::DatabaseError> for DomainError {
    fn from(e: github_scbot_database::DatabaseError) -> Self {
        Self::DatabaseError { source: e }
    }
}

impl From<github_scbot_lock_interface::LockError> for DomainError {
    fn from(e: github_scbot_lock_interface::LockError) -> Self {
        Self::LockError { source: e }
    }
}

/// Result alias for `DomainError`.
pub type Result<T> = core::result::Result<T, DomainError>;
