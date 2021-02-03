//! Issues webhook handlers.

use actix_web::HttpResponse;
use github_scbot_database::DbConn;
use github_scbot_logic::{commands::handle_comment_creation, database::process_repository};
use github_scbot_types::issues::{GHIssueCommentAction, GHIssueCommentEvent};
use tracing::info;

use crate::errors::Result;

pub(crate) async fn issue_comment_event(
    conn: &DbConn,
    event: GHIssueCommentEvent,
) -> Result<HttpResponse> {
    let repo_model = process_repository(conn, &event.repository)?;

    info!(
        "Issue comment event from repository '{}', issue number #{}, action '{:?}' (comment from '{}')",
        event.repository.full_name, event.issue.number, event.action, event.comment.user.login
    );

    // Only handle comments creation
    if let GHIssueCommentAction::Created = event.action {
        handle_comment_creation(
            conn,
            &repo_model,
            event.issue.number,
            &event.issue.user.login,
            &event.issue.body,
        )
        .await?;
    }

    Ok(HttpResponse::Ok().body("Issue comment."))
}
