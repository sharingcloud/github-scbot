//! Check webhook handlers.

use actix_web::HttpResponse;
use github_scbot_database_interface::DbService;
use github_scbot_domain::use_cases::checks::HandleCheckSuiteEventUseCase;
use github_scbot_ghapi_interface::{types::GhCheckSuiteEvent, ApiService};
use github_scbot_lock_interface::LockService;

use super::parse_event_type;
use crate::{event_type::EventType, Result, ServerError};

pub(crate) fn parse_check_suite_event(body: &str) -> Result<GhCheckSuiteEvent> {
    parse_event_type(EventType::CheckSuite, body)
}

pub(crate) async fn check_suite_event(
    api_service: &dyn ApiService,
    db_service: &dyn DbService,
    lock_service: &dyn LockService,
    event: GhCheckSuiteEvent,
) -> Result<HttpResponse> {
    HandleCheckSuiteEventUseCase {
        api_service,
        db_service,
        lock_service,
        event,
    }
    .run()
    .await
    .map_err(|e| ServerError::DomainError { source: e })?;

    Ok(HttpResponse::Ok().body("Check suite."))
}
