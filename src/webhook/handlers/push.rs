//! Push webhook handlers.

use actix_web::HttpResponse;
use tracing::info;

use crate::{
    database::DbConn, logic::database::process_repository, types::push::GHPushEvent,
    webhook::errors::Result,
};

pub(crate) async fn push_event(conn: &DbConn, event: GHPushEvent) -> Result<HttpResponse> {
    process_repository(conn, &event.repository)?;

    info!(
        "Push event from repository '{}', reference '{}' (from '{}')",
        event.repository.full_name, event.reference, event.pusher.name
    );

    Ok(HttpResponse::Ok().body("Push."))
}
