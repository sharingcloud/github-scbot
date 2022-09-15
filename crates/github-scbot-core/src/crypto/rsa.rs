use std::fmt::Display;

use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey};

const RSA_SIZE: u32 = 2048;

/// RSA utilities.
pub struct RsaUtils;
/// Public RSA key.
pub struct PublicRsaKey(String);
/// Private RSA key.
pub struct PrivateRsaKey(String);

impl PublicRsaKey {
    /// Get key as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PrivateRsaKey {
    /// Get key as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for PublicRsaKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for PrivateRsaKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RsaUtils {
    /// Generate a RSA key-pair.
    pub fn generate_rsa_keys() -> (PrivateRsaKey, PublicRsaKey) {
        use ::rsa::{RsaPrivateKey, RsaPublicKey};
        use rand::rngs::OsRng;

        let mut rng = OsRng;
        let priv_key =
            RsaPrivateKey::new(&mut rng, RSA_SIZE as usize).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        let priv_key_pem = priv_key
            .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
            .expect("RSA private key to PEM conversion should work");
        let pub_key_pem = pub_key
            .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
            .expect("RSA public key to PEM conversion should work");

        (
            PrivateRsaKey(priv_key_pem.to_string()),
            PublicRsaKey(pub_key_pem),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::RsaUtils;

    #[test]
    fn test_rsa_generation() {
        let (private, public) = RsaUtils::generate_rsa_keys();
        let private = private.as_str().trim();
        let public = public.as_str().trim();
        assert!(private.starts_with("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(private.ends_with("-----END RSA PRIVATE KEY-----"));
        assert!(public.starts_with("-----BEGIN RSA PUBLIC KEY-----"));
        assert!(public.ends_with("-----END RSA PUBLIC KEY-----"));
    }
}
