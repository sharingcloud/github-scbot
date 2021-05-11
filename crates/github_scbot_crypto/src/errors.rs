//! Crypto errors.

use thiserror::Error;

/// Crypto error.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Jwt creation failed.
    #[error("Jwt creation error.")]
    JwtCreationFailed(#[source] jsonwebtoken::errors::Error),
    /// Jwt verification failed.
    #[error("Jwt verification error.")]
    JwtVerificationFailed(#[source] jsonwebtoken::errors::Error),
}

/// Result alias for `CryptoError`.
pub type Result<T, E = CryptoError> = ::core::result::Result<T, E>;
