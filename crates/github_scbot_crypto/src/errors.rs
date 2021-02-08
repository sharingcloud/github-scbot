//! Crypto errors.

use thiserror::Error;

/// Crypto error.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// JWT creation failed.
    #[error("JWT creation error: {0}")]
    JWTCreationFailed(String),
}

/// Result alias for `CryptoError`.
pub type Result<T, E = CryptoError> = ::core::result::Result<T, E>;
