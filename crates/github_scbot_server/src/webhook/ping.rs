//! Ping webhook handlers.

use actix_web::HttpResponse;
use github_scbot_types::{events::EventType, ping::GhPingEvent};
use tracing::info;

use super::parse_event_type;
use crate::errors::Result;

pub(crate) fn parse_ping_event(body: &str) -> Result<GhPingEvent> {
    parse_event_type(EventType::Ping, body)
}

pub(crate) async fn ping_event(event: GhPingEvent) -> Result<HttpResponse> {
    if let Some(repo) = event.repository {
        info!("Ping event from repository '{}'", repo.full_name);
    } else {
        info!("Ping event without repository");
    }

    Ok(HttpResponse::Ok().body("Ping."))
}
