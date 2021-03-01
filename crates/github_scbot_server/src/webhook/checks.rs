//! Check webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database::DbPool;
use github_scbot_logic::{checks::handle_check_suite_event, database::process_repository};
use github_scbot_types::checks::{GHCheckRunEvent, GHCheckSuiteEvent};
use tracing::info;

use crate::errors::Result;

pub(crate) async fn check_run_event(
    config: Config,
    pool: DbPool,
    event: GHCheckRunEvent,
) -> Result<HttpResponse> {
    info!("Check run event from repository '{}', name '{}', action '{:?}', status '{:?}', conclusion '{:?}'", event.repository.full_name, event.check_run.name, event.action, event.check_run.status, event.check_run.conclusion);

    process_repository(config, pool, event.repository).await?;

    Ok(HttpResponse::Ok().body("Check run."))
}

pub(crate) async fn check_suite_event(
    config: Config,
    pool: DbPool,
    event: GHCheckSuiteEvent,
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
