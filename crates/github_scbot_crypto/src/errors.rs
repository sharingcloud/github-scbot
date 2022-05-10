//! Crypto errors.

use snafu::{prelude::*, Backtrace};

/// Crypto error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum CryptoError {
    #[snafu(display("Jwt creation error,\n  caused by: {}", source))]
    JwtCreationFailed {
        source: jsonwebtoken::errors::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Jwt verification error,\n  caused by: {}", source))]
    JwtVerificationFailed {
        source: jsonwebtoken::errors::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Invalid encoding key,\n  caused by: {}", source))]
    InvalidEncodingKey {
        source: jsonwebtoken::errors::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Invalid decoding key,\n  caused by: {}", source))]
    InvalidDecodingKey {
        source: jsonwebtoken::errors::Error,
        backtrace: Backtrace,
    },
}

/// Result alias for `CryptoError`.
pub type Result<T, E = CryptoError> = ::core::result::Result<T, E>;
