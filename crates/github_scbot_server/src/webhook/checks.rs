//! Check webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database::DbPool;
use github_scbot_logic::checks::handle_check_suite_event;
use github_scbot_types::{checks::GhCheckSuiteEvent, events::EventType};
use tracing::info;

use super::parse_event_type;
use crate::errors::Result;

pub(crate) fn parse_check_suite_event(body: &str) -> Result<GhCheckSuiteEvent> {
    parse_event_type(EventType::CheckSuite, body)
}

pub(crate) async fn check_suite_event(
    config: Config,
    pool: DbPool,
    event: GhCheckSuiteEvent,
) -> Result<HttpResponse> {
    info!(
        "Check suite event from repository '{}', action '{:?}', status '{:?}', conclusion '{:?}'",
        event.repository.full_name,
        event.action,
        event.check_suite.status,
        event.check_suite.conclusion
    );

    handle_check_suite_event(config, pool, event).await?;

    Ok(HttpResponse::Ok().body("Check suite."))
}
