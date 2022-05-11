//! Check webhook handlers.

use actix_web::HttpResponse;
use github_scbot_database2::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_logic::checks::handle_check_suite_event;
use github_scbot_redis::RedisService;
use github_scbot_types::{checks::GhCheckSuiteEvent, events::EventType};
use snafu::ResultExt;

use super::parse_event_type;
use crate::errors::LogicSnafu;
use crate::errors::Result;

pub(crate) fn parse_check_suite_event(body: &str) -> Result<GhCheckSuiteEvent> {
    parse_event_type(EventType::CheckSuite, body)
}

pub(crate) async fn check_suite_event(
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn RedisService,
    event: GhCheckSuiteEvent,
) -> Result<HttpResponse> {
    handle_check_suite_event(api_adapter, db_adapter, redis_adapter, event)
        .await
        .context(LogicSnafu)?;

    Ok(HttpResponse::Ok().body("Check suite."))
}
