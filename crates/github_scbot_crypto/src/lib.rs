//! Crypto module.

#![warn(missing_docs)]
#![warn(clippy::all)]

use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use openssl::rsa::Rsa;
use serde::Serialize;

mod errors;

pub use crate::errors::{CryptoError, Result};

const RSA_SIZE: u32 = 2048;

/// Get current timestamp.
pub fn now() -> u64 {
    let start = SystemTime::now();
    let duration = start.duration_since(UNIX_EPOCH).expect("time collapsed");

    duration.as_secs()
}

/// Create JWT from RSA private key.
///
/// # Arguments
///
/// * `rsa_key` - RSA private key
/// * `claims` - Claims
pub fn create_jwt<T: Serialize>(rsa_key: &str, claims: &T) -> Result<String> {
    let key = EncodingKey::from_rsa_pem(rsa_key.as_bytes()).unwrap();

    match encode(&Header::new(Algorithm::RS256), &claims, &key) {
        Err(e) => Err(CryptoError::JWTCreationFailed(e.to_string())),
        Ok(s) => Ok(s),
    }
}

/// Generate a RSA key-pair.
pub fn generate_rsa_keys() -> (String, String) {
    let rsa = Rsa::generate(RSA_SIZE).unwrap();
    let private_key = rsa.private_key_to_pem().unwrap();
    let public_key = rsa.public_key_to_pem_pkcs1().unwrap();

    (
        String::from_utf8(private_key).unwrap(),
        String::from_utf8(public_key).unwrap(),
    )
}
