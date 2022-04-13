//! Check webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database2::DbService;
use github_scbot_ghapi::adapter::IAPIAdapter;
use github_scbot_logic::checks::handle_check_suite_event;
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::{checks::GhCheckSuiteEvent, events::EventType};

use super::parse_event_type;
use crate::errors::Result;

pub(crate) fn parse_check_suite_event(body: &str) -> Result<GhCheckSuiteEvent> {
    parse_event_type(EventType::CheckSuite, body)
}

#[tracing::instrument(skip(config, api_adapter, db_adapter, redis_adapter))]
pub(crate) async fn check_suite_event(
    config: &Config,
    api_adapter: &dyn IAPIAdapter,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn IRedisAdapter,
    event: GhCheckSuiteEvent,
) -> Result<HttpResponse> {
    handle_check_suite_event(config, api_adapter, db_adapter, redis_adapter, event).await?;

    Ok(HttpResponse::Ok().body("Check suite."))
}
