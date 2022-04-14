//! Issue webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database2::DbService;
use github_scbot_ghapi::adapter::ApiService;
use github_scbot_logic::comments::handle_issue_comment_event;
use github_scbot_redis::IRedisAdapter;
use github_scbot_types::{events::EventType, issues::GhIssueCommentEvent};
use tracing::info;

use super::parse_event_type;
use crate::errors::Result;

pub(crate) fn parse_issue_comment_event(body: &str) -> Result<GhIssueCommentEvent> {
    parse_event_type(EventType::IssueComment, body)
}

pub(crate) async fn issue_comment_event(
    config: &Config,
    api_adapter: &dyn ApiService,
    db_adapter: &dyn DbService,
    redis_adapter: &dyn IRedisAdapter,
    event: GhIssueCommentEvent,
) -> Result<HttpResponse> {
    info!(
        repository_path = %event.repository.full_name,
        pull_request_number = event.issue.number,
        action = ?event.action,
        comment_author = %event.comment.user.login,
        message = "Issue comment event",
    );

    handle_issue_comment_event(config, api_adapter, db_adapter, redis_adapter, event).await?;
    Ok(HttpResponse::Ok().body("Issue comment."))
}
