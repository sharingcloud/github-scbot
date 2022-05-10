use jsonwebtoken::{
    dangerous_insecure_decode, decode, encode, Algorithm, DecodingKey, EncodingKey, Header,
    Validation,
};
use serde::{de::DeserializeOwned, Serialize};
use snafu::ResultExt;

use crate::errors::{
    InvalidDecodingKeySnafu, InvalidEncodingKeySnafu, JwtCreationFailedSnafu,
    JwtVerificationFailedSnafu, Result,
};

/// JWT utilities.
pub struct JwtUtils;

impl JwtUtils {
    /// Create Jwt from RSA private key.
    pub fn create_jwt<T: Serialize>(rsa_priv_key: &str, claims: &T) -> Result<String> {
        let key = Self::parse_encoding_key(rsa_priv_key)?;

        encode(&Header::new(Algorithm::RS256), &claims, &key).context(JwtCreationFailedSnafu)
    }

    /// Verify and decode Jwt.
    pub fn verify_jwt<T>(token: &str, rsa_pub_key: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let key = Self::parse_decoding_key(rsa_pub_key)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = false;

        decode(token, &key, &validation)
            .context(JwtVerificationFailedSnafu)
            .map(|s| s.claims)
    }

    /// Decode Jwt without signature check.
    pub fn decode_jwt<T>(token: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(dangerous_insecure_decode(token)
            .context(JwtVerificationFailedSnafu)?
            .claims)
    }

    /// Parse decoding key.
    pub fn parse_decoding_key(rsa_pub_key: &str) -> Result<DecodingKey> {
        DecodingKey::from_rsa_pem(rsa_pub_key.as_bytes()).context(InvalidDecodingKeySnafu)
    }

    /// Parse encoding key.
    pub fn parse_encoding_key(rsa_priv_key: &str) -> Result<EncodingKey> {
        EncodingKey::from_rsa_pem(rsa_priv_key.as_bytes()).context(InvalidEncodingKeySnafu)
    }
}
