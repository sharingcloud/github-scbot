//! Database errors.

use thiserror::Error;

/// Database error.
#[derive(Debug, Error)]
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

    /// Wraps [`diesel::ConnectionError`].
    #[error("Could not connect to database.")]
    ConnectionError(#[from] diesel::ConnectionError),

    /// Wraps [`diesel_migrations::RunMigrationsError`].
    #[error("Could not execute migrations.")]
    MigrationError(#[from] diesel_migrations::RunMigrationsError),

    /// Wraps [`r2d2::Error`].
    #[error("Could not get a database connection from pool.")]
    DbPoolError(#[from] r2d2::Error),

    /// Wraps [`diesel::result::Error`].
    #[error("Could not run SQL query.")]
    SQLError(#[from] diesel::result::Error),

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

/// Result alias for `DatabaseError`.
pub type Result<T> = core::result::Result<T, DatabaseError>;
