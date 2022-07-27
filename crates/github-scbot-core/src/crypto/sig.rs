use hmac::{Mac, SimpleHmac};
use sha2::Sha256;

/// Check if a signature is valid.
pub fn is_valid_signature<'a>(signature: &str, body: &'a [u8], secret: &str) -> bool {
    let mut hmac = SimpleHmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    hmac.update(body);
    hmac.verify_slice(&hex::decode(signature).unwrap()).is_ok()
}
