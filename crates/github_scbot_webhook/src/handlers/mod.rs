//! Webhook handlers.

mod checks;
mod issues;
mod ping;
mod pull_requests;
mod push;

use std::convert::TryFrom;

use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use github_scbot_database::{DbConn, DbPool};
use github_scbot_types::events::EventType;
use tracing::info;

use crate::{
    constants::GITHUB_EVENT_HEADER,
    errors::{Result, WebhookError},
    utils::convert_payload_to_string,
};

async fn parse_event(conn: &DbConn, event_type: EventType, body: &str) -> Result<HttpResponse> {
    match event_type {
        EventType::CheckRun => {
            checks::check_run_event(
                conn,
                serde_json::from_str(body)
                    .map_err(|e| WebhookError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::CheckSuite => {
            checks::check_suite_event(
                conn,
                serde_json::from_str(body)
                    .map_err(|e| WebhookError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::IssueComment => {
            issues::issue_comment_event(
                conn,
                serde_json::from_str(body)
                    .map_err(|e| WebhookError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::Ping => ping::ping_event(
            conn,
            serde_json::from_str(body).map_err(|e| WebhookError::EventParseError(event_type, e))?,
        )
        .await
        .map_err(Into::into),
        EventType::PullRequest => {
            pull_requests::pull_request_event(
                conn,
                serde_json::from_str(body)
                    .map_err(|e| WebhookError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::PullRequestReview => {
            pull_requests::pull_request_review_event(
                conn,
                serde_json::from_str(body)
                    .map_err(|e| WebhookError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::PullRequestReviewComment => {
            pull_requests::pull_request_review_comment_event(
                conn,
                serde_json::from_str(body)
                    .map_err(|e| WebhookError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::Push => {
            push::push_event(
                conn,
                serde_json::from_str(body)
                    .map_err(|e| WebhookError::EventParseError(event_type, e))?,
            )
            .await
        }
        e => Ok(HttpResponse::BadRequest()
            .body(format!("Event handling is not yet implemented for {:?}", e))),
    }
}

fn extract_event_from_request(req: &HttpRequest) -> Option<EventType> {
    req.headers()
        .get(GITHUB_EVENT_HEADER)
        .and_then(|x| x.to_str().ok())
        .and_then(|x| EventType::try_from(x).ok())
}

pub(crate) async fn event_handler(
    req: HttpRequest,
    mut payload: web::Payload,
    pool: web::Data<DbPool>,
) -> core::result::Result<HttpResponse, Error> {
    // Route event depending on header
    if let Some(event_type) = extract_event_from_request(&req) {
        if let Ok(body) = convert_payload_to_string(&mut payload).await {
            let conn = pool.get().map_err(error::ErrorInternalServerError)?;
            info!("Incoming event: {:?}", event_type);

            parse_event(&conn, event_type, &body)
                .await
                .map_err(Into::into)
        } else {
            let event_type: &str = event_type.into();
            Ok(HttpResponse::BadRequest().body(format!("Bad payload for event '{}'.", event_type)))
        }
    } else {
        Ok(HttpResponse::BadRequest().body("Unhandled event."))
    }
}
