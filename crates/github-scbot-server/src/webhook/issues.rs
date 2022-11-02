//! Issue webhook handlers.

use actix_web::HttpResponse;
use github_scbot_core::config::Config;
use github_scbot_core::types::{events::EventType, issues::GhIssueCommentEvent};
use github_scbot_database::DbService;
use github_scbot_domain::comments::handle_issue_comment_event;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_redis::RedisService;

use super::parse_event_type;
use crate::{Result, ServerError};

pub(crate) fn parse_issue_comment_event(body: &str) -> Result<GhIssueCommentEvent> {
    parse_event_type(EventType::IssueComment, body)
}

pub(crate) async fn issue_comment_event(
    config: &Config,
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn RedisService,
    event: GhIssueCommentEvent,
) -> Result<HttpResponse> {
    handle_issue_comment_event(config, api_adapter, db_adapter, redis_adapter, event)
        .await
        .map_err(|e| ServerError::DomainError { source: e })?;
    Ok(HttpResponse::Ok().body("Issue comment."))
}
