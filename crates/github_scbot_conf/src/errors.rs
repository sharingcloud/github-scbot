//! Configuration errors.

use thiserror::Error;

/// Configuration error.
#[derive(Debug, Error)]
pub enum ConfError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Result alias for `ConfError`.
pub type Result<T, E = ConfError> = core::result::Result<T, E>;
