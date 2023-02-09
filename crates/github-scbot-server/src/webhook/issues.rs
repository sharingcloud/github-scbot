//! Issue webhook handlers.

use actix_web::HttpResponse;
use github_scbot_core::config::Config;
use github_scbot_core::types::{events::EventType, issues::GhIssueCommentEvent};
use github_scbot_database_interface::DbService;
use github_scbot_domain::use_cases::comments::HandleIssueCommentEventUseCase;
use github_scbot_ghapi_interface::ApiService;
use github_scbot_lock_interface::LockService;

use super::parse_event_type;
use crate::{Result, ServerError};

pub(crate) fn parse_issue_comment_event(body: &str) -> Result<GhIssueCommentEvent> {
    parse_event_type(EventType::IssueComment, body)
}

pub(crate) async fn issue_comment_event(
    config: &Config,
    api_adapter: &dyn ApiService,
    db_adapter: &mut dyn DbService,
    redis_adapter: &dyn LockService,
    event: GhIssueCommentEvent,
) -> Result<HttpResponse> {
    HandleIssueCommentEventUseCase {
        config,
        api_service: api_adapter,
        db_service: db_adapter,
        redis_service: redis_adapter,
        event,
    }
    .run()
    .await
    .map_err(|e| ServerError::DomainError { source: e })?;
    Ok(HttpResponse::Ok().body("Issue comment."))
}
