//! Database errors.

use thiserror::Error;

/// Database error.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Unknown repository.
    #[error("Repository `{0}` not found")]
    UnknownRepository(String),

    /// Badly formatted repository path.
    #[error("Badly formatted repository path: {0}")]
    BadRepositoryPath(String),

    /// Unknown pull request.
    #[error("Pull request `#{0}` not found for repository `{1}`")]
    UnknownPullRequest(u64, String),

    /// Unknown review.
    #[error("Unknown pull request review for PR id {0} and username {1}")]
    UnknownReview(i32, String),

    /// Unknown external account.
    #[error("Unknown external account: {0}")]
    UnknownExternalAccount(String),

    /// Unknown merge rule.
    #[error("Unknown merge rule for repository id {0} and branches {1} (base) <- {2} (head)")]
    UnknownMergeRule(i32, String, String),

    /// Unknown review state.
    #[error("Unknown review state: {0}")]
    UnknownReviewState(String),

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
