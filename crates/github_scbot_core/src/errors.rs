//! Core errors.

use thiserror::Error;

/// Core error.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Result alias for `CoreError`.
pub type Result<T, E = CoreError> = core::result::Result<T, E>;
