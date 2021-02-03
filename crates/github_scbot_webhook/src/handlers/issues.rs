//! Issues webhook handlers.

use actix_web::HttpResponse;
use github_scbot_database::{models::PullRequestModel, DbConn};
use github_scbot_logic::{commands::parse_comment, database::process_repository};
use github_scbot_types::issues::{GHIssueCommentAction, GHIssueCommentEvent};
use tracing::{info, warn};

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
        // Try fetching pull request
        if let Some(mut pr_model) = PullRequestModel::get_from_repository_id_and_number(
            conn,
            repo_model.id,
            event.issue.number as i32,
        ) {
            parse_comment(
                conn,
                &repo_model,
                &mut pr_model,
                &event.comment.user.login,
                &event.comment.body,
            )
            .await?;
        } else {
            warn!(
                "Unknown PR #{} for repository {}",
                event.issue.number, event.repository.full_name
            );
        }
    }

    Ok(HttpResponse::Ok().body("Issue comment."))
}
