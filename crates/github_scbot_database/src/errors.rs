//! Database errors.

use thiserror::Error;
use tokio_diesel::AsyncError;

/// Database error.
#[derive(Debug, Error, Clone)]
pub enum DatabaseError {
    /// Unknown repository.
    #[error("Repository '{0}' does not exist.")]
    UnknownRepository(String),

    /// Badly formatted repository path.
    #[error("Badly formatted repository path: '{0}'")]
    BadRepositoryPath(String),

    /// Unknown pull request.
    #[error("Pull request '{0}' #{1} does not exist.")]
    UnknownPullRequest(String, u64),

    /// Unknown account.
    #[error("Account '{0}' does not exist.")]
    UnknownAccount(String),

    /// Unknown external account.
    #[error("External account '{0}' does not exist.")]
    UnknownExternalAccount(String),

    /// Unknown external account right.
    #[error("Right for external account '{0}' on repository '{1}' does not exist.")]
    UnknownExternalAccountRight(String, String),

    /// Unknown merge rule.
    #[error(
        "Merge rule for repository '{0}' and branches '{1}' (base) <- '{2}' (head) does not exist."
    )]
    UnknownMergeRule(String, String, String),

    /// Unknown review state.
    #[error("Review state by user '{0}' on pull request '{1}' #{2} does not exist.")]
    UnknownReviewState(String, String, u64),

    /// Connection error.
    #[error("Could not connect to database: {0}")]
    ConnectionError(String),

    /// Migration error.
    #[error("Could not execute migrations: {0}")]
    MigrationError(String),

    /// Pool error.
    #[error("Could not get a database connection from pool: {0}")]
    DbPoolError(String),

    /// SQL error.
    #[error("Could not run SQL query: {0}")]
    SqlError(String),

    /// Wraps [`super::import_export::ExportError`].
    #[error("Error while exporting data.")]
    ExportError(#[from] super::import_export::ExportError),

    /// Wraps [`super::import_export::ImportError`].
    #[error("Error while importing data.")]
    ImportError(#[from] super::import_export::ImportError),

    /// Wraps [`github_scbot_types::TypeError`].
    #[error(transparent)]
    TypeError(#[from] github_scbot_types::TypeError),

    /// Wraps [`github_scbot_crypto::CryptoError`].
    #[error(transparent)]
    CryptoError(#[from] github_scbot_crypto::CryptoError),
}

impl From<AsyncError> for DatabaseError {
    fn from(err: AsyncError) -> Self {
        match err {
            AsyncError::Checkout(e) => e.into(),
            AsyncError::Error(e) => e.into(),
        }
    }
}

impl From<diesel::ConnectionError> for DatabaseError {
    fn from(err: diesel::ConnectionError) -> Self {
        Self::ConnectionError(err.to_string())
    }
}

impl From<diesel_migrations::RunMigrationsError> for DatabaseError {
    fn from(err: diesel_migrations::RunMigrationsError) -> Self {
        Self::MigrationError(err.to_string())
    }
}

impl From<r2d2::Error> for DatabaseError {
    fn from(err: r2d2::Error) -> Self {
        Self::DbPoolError(err.to_string())
    }
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(err: diesel::result::Error) -> Self {
        Self::SqlError(err.to_string())
    }
}

/// Result alias for `DatabaseError`.
pub type Result<T> = core::result::Result<T, DatabaseError>;
