use std::collections::HashSet;

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Serialize};

use super::{CryptoError, Result};

/// JWT utilities.
pub struct JwtUtils;

impl JwtUtils {
    /// Create Jwt from RSA private key.
    pub fn create_jwt<T: Serialize>(rsa_priv_key: &str, claims: &T) -> Result<String> {
        let key = Self::parse_encoding_key(rsa_priv_key)?;

        encode(&Header::new(Algorithm::RS256), &claims, &key)
            .map_err(|e| CryptoError::JwtCreationFailed { source: e })
    }

    /// Verify and decode Jwt.
    pub fn verify_jwt<T>(token: &str, rsa_pub_key: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let key = Self::parse_decoding_key(rsa_pub_key)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.required_spec_claims = HashSet::new();
        validation.validate_exp = false;

        decode(token, &key, &validation)
            .map_err(|e| CryptoError::JwtVerificationFailed { source: e })
            .map(|s| s.claims)
    }

    /// Decode Jwt without signature check.
    pub fn decode_jwt<T>(token: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.required_spec_claims = HashSet::new();
        validation.validate_exp = false;
        validation.insecure_disable_signature_validation();

        Ok(decode(token, &DecodingKey::from_secret(&[]), &validation)
            .map_err(|e| CryptoError::JwtVerificationFailed { source: e })?
            .claims)
    }

    /// Parse decoding key.
    pub fn parse_decoding_key(rsa_pub_key: &str) -> Result<DecodingKey> {
        DecodingKey::from_rsa_pem(rsa_pub_key.as_bytes())
            .map_err(|e| CryptoError::InvalidDecodingKey { source: e })
    }

    /// Parse encoding key.
    pub fn parse_encoding_key(rsa_priv_key: &str) -> Result<EncodingKey> {
        EncodingKey::from_rsa_pem(rsa_priv_key.as_bytes())
            .map_err(|e| CryptoError::InvalidEncodingKey { source: e })
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::{JwtUtils, RsaUtils};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct SampleClaims {
        hello: String,
    }

    #[test]
    fn create_verify() {
        let (priv_key, pub_key) = RsaUtils::generate_rsa_keys();

        let claims = SampleClaims {
            hello: "Hello!".into(),
        };

        let token = JwtUtils::create_jwt(priv_key.as_str(), &claims).unwrap();
        let extracted_claims: SampleClaims =
            JwtUtils::verify_jwt(&token, pub_key.as_str()).unwrap();

        assert_eq!(claims, extracted_claims);
    }

    #[test]
    fn create_decode() {
        let (priv_key, _pub_key) = RsaUtils::generate_rsa_keys();

        let claims = SampleClaims {
            hello: "Hello!".into(),
        };

        let token = JwtUtils::create_jwt(priv_key.as_str(), &claims).unwrap();
        let extracted_claims: SampleClaims = JwtUtils::decode_jwt(&token).unwrap();

        assert_eq!(claims, extracted_claims);
    }
}
