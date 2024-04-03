//! Crypto errors.

use thiserror::Error;

/// Crypto error.
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Jwt creation error: {source}")]
    JwtCreationFailed { source: jsonwebtoken::errors::Error },
    #[error("Jwt verification error: {source}")]
    JwtVerificationFailed { source: jsonwebtoken::errors::Error },
    #[error("Invalid encoding key: {source}")]
    InvalidEncodingKey { source: jsonwebtoken::errors::Error },
    #[error("Invalid decoding key: {source}")]
    InvalidDecodingKey { source: jsonwebtoken::errors::Error },
    #[error("Invalid signature format {sig}")]
    InvalidSignatureFormat { sig: String },
    #[error("Invalid HMAC secret key length '{key}'")]
    InvalidSecretKeyLength { key: String },
}

/// Result alias for `CryptoError`.
pub type Result<T, E = CryptoError> = ::core::result::Result<T, E>;
