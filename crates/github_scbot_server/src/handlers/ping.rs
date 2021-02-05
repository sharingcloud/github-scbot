//! Ping webhook handlers.

use actix_web::HttpResponse;
use github_scbot_database::DbConn;
use github_scbot_logic::database::process_repository;
use github_scbot_types::ping::GHPingEvent;
use tracing::info;

use crate::errors::Result;

pub(crate) async fn ping_event(conn: &DbConn, event: GHPingEvent) -> Result<HttpResponse> {
    if let Some(repo) = &event.repository {
        info!("Ping event from repository '{}'", repo.full_name);
        process_repository(conn, repo)?;
    } else {
        info!("Ping event without repository",);
    }

    Ok(HttpResponse::Ok().body("Ping."))
}
