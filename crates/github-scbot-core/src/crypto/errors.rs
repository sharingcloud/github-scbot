//! Crypto errors.

use thiserror::Error;

/// Crypto error.
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Jwt creation error,\n  caused by: {}", source)]
    JwtCreationFailed { source: jsonwebtoken::errors::Error },
    #[error("Jwt verification error,\n  caused by: {}", source)]
    JwtVerificationFailed { source: jsonwebtoken::errors::Error },
    #[error("Invalid encoding key,\n  caused by: {}", source)]
    InvalidEncodingKey { source: jsonwebtoken::errors::Error },
    #[error("Invalid decoding key,\n  caused by: {}", source)]
    InvalidDecodingKey { source: jsonwebtoken::errors::Error },
    #[error("Invalid signature format: {}", sig)]
    InvalidSignatureFormat {
        sig: String,
        source: hex::FromHexError,
    },
    #[error("Invalid HMAC secret key length: {}", key)]
    InvalidSecretKeyLength {
        key: String,
        source: hmac::digest::InvalidLength,
    },
}

/// Result alias for `CryptoError`.
pub type Result<T, E = CryptoError> = ::core::result::Result<T, E>;
