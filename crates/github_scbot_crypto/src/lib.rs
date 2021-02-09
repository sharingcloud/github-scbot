//! Crypto module.

#![warn(missing_docs)]
#![warn(clippy::all)]

use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    dangerous_insecure_decode, decode, encode, Algorithm, DecodingKey, EncodingKey, Header,
    Validation,
};
use openssl::rsa::Rsa;
use serde::{de::DeserializeOwned, Serialize};

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
/// * `rsa_priv_key` - RSA private key
/// * `claims` - Claims
pub fn create_jwt<T: Serialize>(rsa_priv_key: &str, claims: &T) -> Result<String> {
    let key = EncodingKey::from_rsa_pem(rsa_priv_key.as_bytes()).unwrap();

    match encode(&Header::new(Algorithm::RS256), &claims, &key) {
        Err(e) => Err(CryptoError::JWTCreationFailed(e.to_string())),
        Ok(s) => Ok(s),
    }
}

/// Verify and decode JWT.
///
/// # Arguments
///
/// * `token` - Token
/// * `rsa_pub_key` - RSA public key
pub fn verify_jwt<T>(token: &str, rsa_pub_key: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    println!("{:?}", token);

    let key = DecodingKey::from_rsa_pem(rsa_pub_key.as_bytes()).unwrap();
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;

    match decode(token, &key, &validation) {
        Err(e) => Err(CryptoError::JWTVerificationFailed(e.to_string())),
        Ok(s) => Ok(s.claims),
    }
}

/// Decode JWT without signature check.
///
/// # Arguments
///
/// * `token` - Token
pub fn decode_jwt<T>(token: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(dangerous_insecure_decode(token).unwrap().claims)
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
