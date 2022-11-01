//! Crypto errors.

use snafu::prelude::*;

/// Crypto error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum CryptoError {
    #[snafu(display("Jwt creation error,\n  caused by: {}", source))]
    JwtCreationFailed { source: jsonwebtoken::errors::Error },
    #[snafu(display("Jwt verification error,\n  caused by: {}", source))]
    JwtVerificationFailed { source: jsonwebtoken::errors::Error },
    #[snafu(display("Invalid encoding key,\n  caused by: {}", source))]
    InvalidEncodingKey { source: jsonwebtoken::errors::Error },
    #[snafu(display("Invalid decoding key,\n  caused by: {}", source))]
    InvalidDecodingKey { source: jsonwebtoken::errors::Error },
    #[snafu(display("Invalid signature format: {}", sig))]
    InvalidSignatureFormat {
        sig: String,
        source: hex::FromHexError,
    },
    #[snafu(display("Invalid HMAC secret key length: {}", key))]
    InvalidSecretKeyLength {
        key: String,
        source: hmac::digest::InvalidLength,
    },
}

/// Result alias for `CryptoError`.
pub type Result<T, E = CryptoError> = ::core::result::Result<T, E>;
