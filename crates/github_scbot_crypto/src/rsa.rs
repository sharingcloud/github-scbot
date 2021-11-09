use std::fmt::Display;

use openssl::rsa::Rsa;

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
        let rsa = Rsa::generate(RSA_SIZE).expect("RSA generation should work");
        let private_key = rsa
            .private_key_to_pem()
            .expect("RSA private key to PEM conversion should work");
        let public_key = rsa
            .public_key_to_pem_pkcs1()
            .expect("RSA public key to PEM conversion should work");

        (
            PrivateRsaKey(
                String::from_utf8(private_key).expect("RSA private key should be valid utf-8"),
            ),
            PublicRsaKey(
                String::from_utf8(public_key).expect("RSA public key should be valid utf-8"),
            ),
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
