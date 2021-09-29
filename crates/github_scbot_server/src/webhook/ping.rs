//! Ping webhook handlers.

use actix_web::HttpResponse;
use github_scbot_types::{events::EventType, ping::GhPingEvent};
use tracing::info;

use super::parse_event_type;
use crate::errors::Result;

pub(crate) fn parse_ping_event(body: &str) -> Result<GhPingEvent> {
    parse_event_type(EventType::Ping, body)
}

pub(crate) fn ping_event(event: GhPingEvent) -> HttpResponse {
    if let Some(repo) = event.repository {
        info!(
            message = "Ping event from repository",
            repository_path = %repo.full_name
        );
    } else {
        info!("Ping event without repository");
    }

    HttpResponse::Ok().body("Ping.")
}
