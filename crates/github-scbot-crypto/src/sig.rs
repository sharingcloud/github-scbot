use hmac::{Mac, SimpleHmac};
use sha2::Sha256;

use super::errors::CryptoError;

/// Signature.
pub struct Signature<'a>(pub &'a str);

impl<'a> Signature<'a> {
    /// Check if a signature is valid.
    pub fn is_valid(&self, body: &[u8], secret: &str) -> Result<bool, CryptoError> {
        let decoded_signature =
            &hex::decode(self.0).map_err(|_| CryptoError::InvalidSignatureFormat {
                sig: self.0.to_string(),
            })?;
        let mut hmac = SimpleHmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| {
            CryptoError::InvalidSecretKeyLength {
                key: secret.to_string(),
            }
        })?;

        hmac.update(body);
        Ok(hmac.verify_slice(decoded_signature).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::Signature;

    struct SigSet {
        signature: &'static str,
        body: &'static [u8],
        secret: &'static str,
    }

    fn valid_sig_set() -> SigSet {
        SigSet {
            signature: "a2b41e3bb9a09babb36b42e145eacc38916d078ba378d60db679f6ac79cd1408",
            body: r#"{"secret": "hello"}"#.as_bytes(),
            secret: "iAmAsEcReTkEy",
        }
    }

    fn invalid_sig_set() -> SigSet {
        SigSet {
            signature: "a2b41e3bb9a09babb36b42e145eacc38916d078ba378d60db679f6ac79cd1409",
            body: r#"{"secret": "hello"}"#.as_bytes(),
            secret: "iAmAsEcReTkEy",
        }
    }

    #[test]
    fn test_is_valid_signature_valid() {
        let sigset = valid_sig_set();
        assert!(
            Signature(sigset.signature)
                .is_valid(sigset.body, sigset.secret)
                .unwrap(),
            "signature should be valid"
        );
    }

    #[test]
    fn test_is_valid_signature_invalid() {
        let sigset = invalid_sig_set();
        assert!(
            !Signature(sigset.signature)
                .is_valid(sigset.body, sigset.secret)
                .unwrap(),
            "signature should NOT be valid"
        );
    }
}
