//! Push webhook handlers.

use actix_web::HttpResponse;
use github_scbot_database::DbConn;
use github_scbot_logic::database::process_repository;
use github_scbot_types::push::GHPushEvent;
use tracing::info;

use crate::errors::Result;

pub(crate) async fn push_event(conn: &DbConn, event: GHPushEvent) -> Result<HttpResponse> {
    process_repository(conn, &event.repository)?;

    info!(
        "Push event from repository '{}', reference '{}' (from '{}')",
        event.repository.full_name, event.reference, event.pusher.name
    );

    Ok(HttpResponse::Ok().body("Push."))
}
