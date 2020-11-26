//! Webhook issue handlers

use actix_web::HttpResponse;
use eyre::Result;
use log::info;

use crate::database::models::DbConn;
use crate::webhook::logic::process_repository;
use crate::webhook::types::IssueCommentEvent;

pub async fn issue_comment_event(conn: &DbConn, event: IssueCommentEvent) -> Result<HttpResponse> {
    process_repository(conn, &event.repository)?;

    info!(
        "Issue comment event from repository '{}', issue number #{}, action '{:?}' (comment from '{}')",
        event.repository.full_name, event.issue.number, event.action, event.comment.user.login
    );

    Ok(HttpResponse::Ok().body("Issue comment."))
}
