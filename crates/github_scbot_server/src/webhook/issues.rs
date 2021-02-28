//! Issue webhook handlers.

use actix_web::HttpResponse;
use github_scbot_conf::Config;
use github_scbot_database::{get_connection, DbPool};
use github_scbot_logic::comments::handle_issue_comment_event;
use github_scbot_types::issues::GHIssueCommentEvent;
use tracing::info;

use crate::errors::Result;

pub(crate) async fn issue_comment_event(
    config: &Config,
    pool: DbPool,
    event: GHIssueCommentEvent,
) -> Result<HttpResponse> {
    info!(
        "Issue comment event from repository '{}', issue number #{}, action '{:?}' (comment from '{}')",
        event.repository.full_name, event.issue.number, event.action, event.comment.user.login
    );

    handle_issue_comment_event(config, &*get_connection(&pool)?, &event).await?;
    Ok(HttpResponse::Ok().body("Issue comment."))
}
