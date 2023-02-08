//! Check webhook handlers.

use actix_web::HttpResponse;
use github_scbot_core::types::{checks::GhCheckSuiteEvent, events::EventType};
use github_scbot_database::DbService;
use github_scbot_domain::use_cases::checks::HandleCheckSuiteEventUseCase;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::LockService;

use super::parse_event_type;
use crate::{Result, ServerError};

pub(crate) fn parse_check_suite_event(body: &str) -> Result<GhCheckSuiteEvent> {
    parse_event_type(EventType::CheckSuite, body)
}

pub(crate) async fn check_suite_event(
    api_adapter: &dyn ApiService,
    db_adapter: &mut dyn DbService,
    redis_adapter: &dyn LockService,
    event: GhCheckSuiteEvent,
) -> Result<HttpResponse> {
    HandleCheckSuiteEventUseCase {
        api_service: api_adapter,
        db_service: db_adapter,
        redis_service: redis_adapter,
        event,
    }
    .run()
    .await
    .map_err(|e| ServerError::DomainError { source: e })?;

    Ok(HttpResponse::Ok().body("Check suite."))
}
