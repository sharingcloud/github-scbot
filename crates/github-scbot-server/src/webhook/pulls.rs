//! Pull webhook handlers.

use actix_web::HttpResponse;
use github_scbot_core::config::Config;
use github_scbot_core::types::{
    events::EventType,
    pulls::{GhPullRequestAction, GhPullRequestEvent},
};
use github_scbot_database_interface::DbService;
use github_scbot_domain::use_cases::pulls::{
    HandlePullRequestEventUseCase, ProcessPullRequestOpenedUseCase,
};
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use super::parse_event_type;
use crate::{Result, ServerError};

pub(crate) fn parse_pull_request_event(body: &str) -> Result<GhPullRequestEvent> {
    parse_event_type(EventType::PullRequest, body)
}

pub(crate) async fn pull_request_event(
    config: &Config,
    api_adapter: &dyn ApiService,
    db_adapter: &mut dyn DbService,
    redis_adapter: &dyn LockService,
    event: GhPullRequestEvent,
) -> Result<HttpResponse> {
    if matches!(event.action, GhPullRequestAction::Opened) {
        ProcessPullRequestOpenedUseCase {
            api_service: api_adapter,
            db_service: db_adapter,
            config,
            lock_service: redis_adapter,
            event,
        }
        .run()
        .await
        .map_err(|e| ServerError::DomainError { source: e })?;
    } else {
        HandlePullRequestEventUseCase {
            api_service: api_adapter,
            db_service: db_adapter,
            lock_service: redis_adapter,
            event,
        }
        .run()
        .await
        .map_err(|e| ServerError::DomainError { source: e })?;
    }

    Ok(HttpResponse::Ok().body("Pull request."))
}
