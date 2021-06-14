//! Crypto errors.

use thiserror::Error;

/// Crypto error.
#[derive(Debug, Error, Clone)]
pub enum CryptoError {
    /// Jwt creation failed.
    #[error("Jwt creation error.")]
    JwtCreationFailed(String),
    /// Jwt verification failed.
    #[error("Jwt verification error.")]
    JwtVerificationFailed(String),
    /// Invalid encoding key.
    #[error("Invalid encoding key.")]
    InvalidEncodingKey(String),
    /// Invalid decoding key.
    #[error("Invalid decoding key.")]
    InvalidDecodingKey(String),
}

/// Result alias for `CryptoError`.
pub type Result<T, E = CryptoError> = ::core::result::Result<T, E>;
