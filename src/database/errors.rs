//! Database errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Import error: {0}")]
    ImportError(String),

    #[error("Repository `{0}` not found")]
    UnknownRepositoryError(String),

    #[error("Badly formatted repository path: {0}")]
    BadRepositoryPathError(String),

    #[error("Pull request `#{0}` not found for repository `{1}`")]
    UnknownPullRequestError(u64, String),

    #[error("Unknown check status: {0}")]
    UnknownCheckStatusError(String),

    #[error("Unknown QA status: {0}")]
    UnknownQAStatusError(String),

    #[error("Unknown step label: {0}")]
    UnknownStepLabelError(String),

    #[error(transparent)]
    ConnectionError(#[from] diesel::ConnectionError),

    #[error(transparent)]
    MigrationError(#[from] diesel_migrations::RunMigrationsError),

    #[error(transparent)]
    R2d2Error(#[from] r2d2::Error),

    #[error(transparent)]
    SQLError(#[from] diesel::result::Error),
}

pub type Result<T> = core::result::Result<T, DatabaseError>;
