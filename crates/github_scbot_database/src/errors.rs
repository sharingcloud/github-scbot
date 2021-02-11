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
    UnknownPullRequest(String, i32),

    /// Unknown review.
    #[error("Pull request review for pull request ID '{0}' and username '{1}' does not exist.")]
    UnknownReview(i32, String),

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
    #[error("Review state by user '{0}' on pull request ID '{0}' does not exist.")]
    UnknownReviewState(String, String),

    /// Wraps [`super::import_export::ExportError`].
    #[error("Export error: {0}")]
    ExportError(#[from] super::import_export::ExportError),

    /// Wraps [`super::import_export::ImportError`].
    #[error("Import error: {0}")]
    ImportError(#[from] super::import_export::ImportError),

    /// Wraps [`diesel::ConnectionError`].
    #[error(transparent)]
    ConnectionError(#[from] diesel::ConnectionError),

    /// Wraps [`diesel_migrations::RunMigrationsError`].
    #[error(transparent)]
    MigrationError(#[from] diesel_migrations::RunMigrationsError),

    /// Wraps [`github_scbot_crypto::CryptoError`].
    #[error(transparent)]
    CryptoError(#[from] github_scbot_crypto::CryptoError),

    /// Wraps [`r2d2::Error`].
    #[error(transparent)]
    R2d2Error(#[from] r2d2::Error),

    /// Wraps [`diesel::result::Error`].
    #[error(transparent)]
    SQLError(#[from] diesel::result::Error),
}

/// Result alias for `DatabaseError`.
pub type Result<T> = core::result::Result<T, DatabaseError>;
