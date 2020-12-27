//! Webhook issue handlers

use actix_web::HttpResponse;
use tracing::{info, warn};

use crate::database::models::{DbConn, PullRequestModel};
use crate::types::{IssueCommentAction, IssueCommentEvent};
use crate::webhook::errors::Result;
use crate::webhook::logic::{commands::parse_issue_comment, database::process_repository};

#[allow(clippy::cast_possible_truncation)]
pub async fn issue_comment_event(conn: &DbConn, event: IssueCommentEvent) -> Result<HttpResponse> {
    let repo_model = process_repository(conn, &event.repository)?;

    info!(
        "Issue comment event from repository '{}', issue number #{}, action '{:?}' (comment from '{}')",
        event.repository.full_name, event.issue.number, event.action, event.comment.user.login
    );

    // Only handle comments creation
    if let IssueCommentAction::Created = event.action {
        // Try fetching pull request
        if let Some(mut pr_model) =
            PullRequestModel::get_from_number(conn, repo_model.id, event.issue.number as i32)
        {
            parse_issue_comment(
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
