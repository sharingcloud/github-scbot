//! Webhook issue handlers

use std::convert::TryInto;

use actix_web::HttpResponse;
use eyre::Result;
use log::info;

use crate::database::models::{DbConn, PullRequestModel};
use crate::webhook::logic::{parse_issue_comment, process_repository};
use crate::webhook::types::{IssueCommentAction, IssueCommentEvent};

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
            PullRequestModel::get_from_number(conn, repo_model.id, event.issue.number.try_into()?)
        {
            parse_issue_comment(
                conn,
                &repo_model,
                &mut pr_model,
                &event.comment.user.login,
                &event.comment.body,
            )
            .await?;
        }
    }

    Ok(HttpResponse::Ok().body("Issue comment."))
}
