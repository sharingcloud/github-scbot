//! Ping webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database::DbPool;
use github_scbot_logic::database::process_repository;
use github_scbot_types::ping::GHPingEvent;
use tracing::info;

use crate::errors::Result;

pub(crate) async fn ping_event(
    config: Config,
    pool: DbPool,
    event: GHPingEvent,
) -> Result<HttpResponse> {
    if let Some(repo) = event.repository {
        info!("Ping event from repository '{}'", repo.full_name);
        process_repository(config, pool.clone(), repo).await?;
    } else {
        info!("Ping event without repository",);
    }

    Ok(HttpResponse::Ok().body("Ping."))
}
