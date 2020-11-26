//! Webhook ping handlers

use actix_web::HttpResponse;
use eyre::Result;
use log::info;

use crate::database::models::DbConn;
use crate::webhook::logic::process_repository;
use crate::webhook::types::PingEvent;

pub async fn ping_event(conn: &DbConn, event: PingEvent) -> Result<HttpResponse> {
    process_repository(conn, &event.repository)?;

    info!(
        "Ping event from repository '{}'",
        event.repository.full_name
    );

    Ok(HttpResponse::Ok().body("Ping."))
}
