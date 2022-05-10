//! UI errors.

use snafu::{prelude::*, Backtrace};

/// UI error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum UiError {
    /// Unsupported OS.
    #[snafu(display("Current OS is unsupported (for now)."))]
    Unsupported { backtrace: Backtrace },

    /// Wraps [`std::io::IoError`].
    #[snafu(display("I/O error,\n  caused by: {}", source))]
    Io {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    /// Wraps [`std::sync::mpsc::RecvError`].
    #[snafu(display("Channel communication error,\n  caused by: {}", source))]
    Recv {
        source: std::sync::mpsc::RecvError,
        backtrace: Backtrace,
    },

    /// Wraps [`github_scbot_database2::DatabaseError`].
    #[snafu(display("Database error,\n  caused by: {}", source))]
    Database {
        #[snafu(backtrace)]
        source: github_scbot_database2::DatabaseError,
    },
}

impl From<std::io::Error> for UiError {
    fn from(e: std::io::Error) -> Self {
        Self::Io {
            source: e,
            backtrace: Backtrace::new(),
        }
    }
}

impl From<std::sync::mpsc::RecvError> for UiError {
    fn from(e: std::sync::mpsc::RecvError) -> Self {
        Self::Recv {
            source: e,
            backtrace: Backtrace::new(),
        }
    }
}

impl From<github_scbot_database2::DatabaseError> for UiError {
    fn from(e: github_scbot_database2::DatabaseError) -> Self {
        Self::Database { source: e }
    }
}

/// Result alias for `UiError`.
pub type Result<T> = core::result::Result<T, UiError>;
