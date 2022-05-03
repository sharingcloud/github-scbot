//! UI errors.

use thiserror::Error;

/// UI error.
#[derive(Debug, Error)]
pub enum UiError {
    /// Unsupported OS.
    #[error("Current OS is unsupported (for now).")]
    Unsupported,

    /// Wraps [`std::io::IoError`].
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Wraps [`std::sync::mpsc::RecvError`].
    #[error("Channel communication error.")]
    Recv(#[from] std::sync::mpsc::RecvError),

    /// Wraps [`github_scbot_database2::DatabaseError`].
    #[error(transparent)]
    Database(#[from] github_scbot_database2::DatabaseError),
}

/// Result alias for `UiError`.
pub type Result<T> = core::result::Result<T, UiError>;
