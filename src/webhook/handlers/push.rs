//! Webhook push handlers

use actix_web::HttpResponse;
use eyre::Result;
use log::info;

use crate::database::models::DbConn;
use crate::webhook::logic::process_repository;
use crate::webhook::types::PushEvent;

pub async fn push_event(conn: &DbConn, event: PushEvent) -> Result<HttpResponse> {
    process_repository(conn, &event.repository)?;

    info!(
        "Push event from repository '{}', reference '{}' (from '{}')",
        event.repository.full_name, event.reference, event.pusher.name
    );

    Ok(HttpResponse::Ok().body("Push."))
}
