//! Webhook ping handlers

use actix_web::HttpResponse;
use tracing::info;

use crate::database::models::DbConn;
use crate::webhook::errors::Result;
use crate::webhook::logic::database::process_repository;
use crate::webhook::types::PingEvent;

pub async fn ping_event(conn: &DbConn, event: PingEvent) -> Result<HttpResponse> {
    if let Some(repo) = &event.repository {
        process_repository(conn, repo)?;

        info!("Ping event from repository '{}'", repo.full_name);
    } else {
        info!("Ping event without repository",);
    }

    Ok(HttpResponse::Ok().body("Ping."))
}
