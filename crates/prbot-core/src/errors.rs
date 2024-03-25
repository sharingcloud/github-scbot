//! Logic errors.

use thiserror::Error;

/// Logic error.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum DomainError {
    /// Wraps [`regex::Error`].
    #[error("Error while compiling regex: {source}")]
    RegexError { source: regex::Error },

    /// Wraps [`prbot_ghapi_interface::ApiError`].
    #[error("API error: {source}")]
    ApiError {
        source: prbot_ghapi_interface::ApiError,
    },

    /// Wraps [`prbot_database_interface::DatabaseError`].
    #[error("Database error: {source}")]
    DatabaseError {
        source: prbot_database_interface::DatabaseError,
    },

    #[error("Lock service error: {source}")]
    LockError {
        source: prbot_lock_interface::LockError,
    },

    #[error("Crypto error: {source}")]
    CryptoError { source: prbot_crypto::CryptoError },
}

impl From<regex::Error> for DomainError {
    fn from(e: regex::Error) -> Self {
        Self::RegexError { source: e }
    }
}

impl From<prbot_ghapi_interface::ApiError> for DomainError {
    fn from(e: prbot_ghapi_interface::ApiError) -> Self {
        Self::ApiError { source: e }
    }
}

impl From<prbot_database_interface::DatabaseError> for DomainError {
    fn from(e: prbot_database_interface::DatabaseError) -> Self {
        Self::DatabaseError { source: e }
    }
}

impl From<prbot_lock_interface::LockError> for DomainError {
    fn from(e: prbot_lock_interface::LockError) -> Self {
        Self::LockError { source: e }
    }
}

impl From<prbot_crypto::CryptoError> for DomainError {
    fn from(e: prbot_crypto::CryptoError) -> Self {
        Self::CryptoError { source: e }
    }
}

/// Result alias for `DomainError`.
pub type Result<T> = core::result::Result<T, DomainError>;
