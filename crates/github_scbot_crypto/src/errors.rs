//! Crypto errors.

use thiserror::Error;

/// Crypto error.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// JWT creation failed.
    #[error("JWT creation error.")]
    JWTCreationFailed(#[source] jsonwebtoken::errors::Error),
    /// JWT verification failed.
    #[error("JWT verification error.")]
    JWTVerificationFailed(#[source] jsonwebtoken::errors::Error),
}

/// Result alias for `CryptoError`.
pub type Result<T, E = CryptoError> = ::core::result::Result<T, E>;
