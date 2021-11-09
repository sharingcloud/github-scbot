use jsonwebtoken::{
    dangerous_insecure_decode, decode, encode, Algorithm, DecodingKey, EncodingKey, Header,
    Validation,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::errors::{CryptoError, Result};

/// JWT utilities.
pub struct JwtUtils;

impl JwtUtils {
    /// Create Jwt from RSA private key.
    pub fn create_jwt<T: Serialize>(rsa_priv_key: &str, claims: &T) -> Result<String> {
        let key = EncodingKey::from_rsa_pem(rsa_priv_key.as_bytes())
            .map_err(|e| CryptoError::InvalidEncodingKey(e.to_string()))?;

        match encode(&Header::new(Algorithm::RS256), &claims, &key) {
            Err(e) => Err(CryptoError::JwtCreationFailed(e.to_string())),
            Ok(s) => Ok(s),
        }
    }

    /// Verify and decode Jwt.
    pub fn verify_jwt<T>(token: &str, rsa_pub_key: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let key = DecodingKey::from_rsa_pem(rsa_pub_key.as_bytes())
            .map_err(|e| CryptoError::InvalidDecodingKey(e.to_string()))?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = false;

        match decode(token, &key, &validation) {
            Err(e) => Err(CryptoError::JwtVerificationFailed(e.to_string())),
            Ok(s) => Ok(s.claims),
        }
    }

    /// Decode Jwt without signature check.
    pub fn decode_jwt<T>(token: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(dangerous_insecure_decode(token)
            .map_err(|e| CryptoError::JwtVerificationFailed(e.to_string()))?
            .claims)
    }
}
