//! Issue webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database::DbPool;
use github_scbot_logic::comments::handle_issue_comment_event;
use github_scbot_types::{events::EventType, issues::GhIssueCommentEvent};
use tracing::info;

use super::parse_event_type;
use crate::errors::Result;

pub(crate) fn parse_issue_comment_event(body: &str) -> Result<GhIssueCommentEvent> {
    parse_event_type(EventType::IssueComment, body)
}

pub(crate) async fn issue_comment_event(
    config: Config,
    pool: DbPool,
    event: GhIssueCommentEvent,
) -> Result<HttpResponse> {
    info!(
        repository_path = %event.repository.full_name,
        pull_request_number = event.issue.number,
        action = ?event.action,
        comment_author = %event.comment.user.login,
        message = "Issue comment event",
    );

    handle_issue_comment_event(config, pool, event).await?;
    Ok(HttpResponse::Ok().body("Issue comment."))
}
