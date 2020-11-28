//! Webhook handlers

mod checks;
mod issues;
mod ping;
mod pull_request;
mod push;

use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use color_eyre::eyre::{Result, WrapErr};
use log::info;

use super::constants::GITHUB_EVENT_HEADER;
use super::errors::WebhookError;
use super::types::EventType;
use super::utils::convert_payload_to_string;
use crate::database::{DbConn, DbPool};

async fn parse_event(conn: &DbConn, event_type: &EventType, body: &str) -> Result<HttpResponse> {
    match event_type {
        EventType::CheckRun => {
            checks::check_run_event(
                conn,
                serde_json::from_str(body).wrap_err("Malformed 'check_run' event payload")?,
            )
            .await
        }
        EventType::CheckSuite => {
            checks::check_suite_event(
                conn,
                serde_json::from_str(body).wrap_err("Malformed 'check_suite' event payload")?,
            )
            .await
        }
        EventType::IssueComment => {
            issues::issue_comment_event(
                conn,
                serde_json::from_str(body).wrap_err("Malformed 'issue_comment' event payload")?,
            )
            .await
        }
        EventType::Ping => ping::ping_event(
            conn,
            serde_json::from_str(body).wrap_err("Malformed 'ping' event payload")?,
        )
        .await
        .map_err(Into::into),
        EventType::PullRequest => {
            pull_request::pull_request_event(
                conn,
                serde_json::from_str(body).wrap_err("Malformed 'pull_request' event payload")?,
            )
            .await
        }
        EventType::PullRequestReview => {
            pull_request::pull_request_review_event(
                conn,
                serde_json::from_str(body)
                    .wrap_err("Malformed 'pull_request_review' event payload")?,
            )
            .await
        }
        EventType::PullRequestReviewComment => {
            pull_request::pull_request_review_comment_event(
                conn,
                serde_json::from_str(body)
                    .wrap_err("Malformed 'pull_request_review_comment' event payload")?,
            )
            .await
        }
        EventType::Push => {
            push::push_event(
                conn,
                serde_json::from_str(body).wrap_err("Malformed 'push' event payload")?,
            )
            .await
        }
        e => Ok(HttpResponse::BadRequest()
            .body(format!("Event handling is not yet implemented for {:?}", e))),
    }
}

pub async fn event_handler(
    req: HttpRequest,
    mut payload: web::Payload,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    // Route event depending on header
    if let Ok(Some(event_type)) = req
        .headers()
        .get(GITHUB_EVENT_HEADER)
        .map(|x| EventType::try_from_str(x.to_str()?))
        .map_or(Ok(None), |r| r.map(Some))
    {
        if let Ok(body) = convert_payload_to_string(&mut payload).await {
            let conn = pool.get().map_err(error::ErrorInternalServerError)?;
            info!("Incoming event: {:?}", event_type);

            parse_event(&conn, &event_type, &body).await.map_err(|e| {
                let error: WebhookError = e.into();
                error.into()
            })
        } else {
            Ok(HttpResponse::BadRequest()
                .body(format!("Bad payload for event '{}'.", event_type.as_str())))
        }
    } else {
        Ok(HttpResponse::BadRequest().body("Unhandled event."))
    }
}