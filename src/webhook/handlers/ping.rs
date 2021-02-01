//! Ping webhook handlers.

use actix_web::HttpResponse;
use tracing::info;

use crate::{
    database::DbConn, logic::database::process_repository, types::ping::GHPingEvent,
    webhook::errors::Result,
};

pub(crate) async fn ping_event(conn: &DbConn, event: GHPingEvent) -> Result<HttpResponse> {
    if let Some(repo) = &event.repository {
        process_repository(conn, repo)?;

        info!("Ping event from repository '{}'", repo.full_name);
    } else {
        info!("Ping event without repository",);
    }

    Ok(HttpResponse::Ok().body("Ping."))
}
