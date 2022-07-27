//! Crypto module.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod errors;
mod jwt;
mod rsa;

pub use jwt::JwtUtils;
pub use rand;

pub use self::{
    errors::{CryptoError, Result},
    rsa::RsaUtils,
};

#[cfg(test)]
mod tests {
    use jsonwebtoken::{DecodingKey, EncodingKey};

    use super::RsaUtils;

    #[test]
    fn test_key_decoding() {
        let (private, _) = RsaUtils::generate_rsa_keys();

        DecodingKey::from_rsa_pem(private.as_str().as_bytes()).expect("Should be valid");
        EncodingKey::from_rsa_pem(private.as_str().as_bytes()).expect("Should be valid");
    }
}
