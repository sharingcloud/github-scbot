//! Webhook utilities module.

use std::error::Error;

use actix_web::web::{Bytes, BytesMut, Payload};
use futures::stream::StreamExt;

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
