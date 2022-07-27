//! Webhook utilities module.

use std::error::Error;

use actix_web::web::{Bytes, BytesMut, Payload};
use crypto::{hmac::Hmac, mac::Mac, sha2::Sha256};
use futures::stream::StreamExt;

/// Check if a signature is valid.
pub fn is_valid_signature<'a>(signature: &str, body: &'a [u8], secret: &str) -> bool {
    let digest = Sha256::new();
    let mut hmac = Hmac::new(digest, secret.as_bytes());
    hmac.input(body);
    let expected_signature = hmac.result();

    crypto::util::fixed_time_eq(
        hex::encode(expected_signature.code()).as_bytes(),
        signature.as_bytes(),
    )
}

/// Convert Actix payload to bytes.
pub async fn convert_payload_to_bytes(payload: &mut Payload) -> Result<Bytes, Box<dyn Error>> {
    let mut body = BytesMut::new();

    while let Some(chunk) = payload.next().await {
        body.extend_from_slice(&chunk?);
    }

    Ok(body.into())
}

/// Convert Actix payload to string.
pub async fn convert_payload_to_string(payload: &mut Payload) -> Result<String, Box<dyn Error>> {
    let bytes = convert_payload_to_bytes(payload).await?;
    std::str::from_utf8(&bytes)
        .map(ToOwned::to_owned)
        .map_err(Into::into)
}
