//! Webhook handlers

mod checks;
mod issues;
mod ping;
mod pull_request;
mod push;

use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use eyre::Result;
use log::{error, info};

use super::constants::GITHUB_EVENT_HEADER;
use super::types::EventType;
use super::utils::convert_payload_to_string;
use crate::database::DbPool;

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
            let conn = pool.get().map_err(|_| {
                error!("Error while getting connection from database pool.");
                HttpResponse::InternalServerError()
            })?;

            info!("Incoming event: {:?}, payload: {:?}", event_type, body);

            match event_type {
                EventType::CheckRun => {
                    checks::check_run_event(&conn, serde_json::from_str(&body)?).await
                }
                EventType::CheckSuite => {
                    checks::check_suite_event(&conn, serde_json::from_str(&body)?).await
                }
                EventType::IssueComment => {
                    issues::issue_comment_event(&conn, serde_json::from_str(&body)?).await
                }
                EventType::Ping => ping::ping_event(&conn, serde_json::from_str(&body)?).await,
                EventType::PullRequest => {
                    pull_request::pull_request_event(&conn, serde_json::from_str(&body)?).await
                }
                EventType::PullRequestReview => {
                    pull_request::pull_request_review_event(&conn, serde_json::from_str(&body)?)
                        .await
                }
                EventType::PullRequestReviewComment => {
                    pull_request::pull_request_review_comment_event(
                        &conn,
                        serde_json::from_str(&body)?,
                    )
                    .await
                }
                EventType::Push => push::push_event(&conn, serde_json::from_str(&body)?).await,
                e => Ok(HttpResponse::BadRequest().body(format!(
                    "Event handling is not yet implemented for {}",
                    e.as_str()
                ))),
            }
            .map_err(|e| {
                eprintln!("ERROR: {:?}", e);
                error::ErrorInternalServerError(e)
            })
        } else {
            Ok(HttpResponse::BadRequest()
                .body(format!("Bad payload for event '{}'.", event_type.as_str())))
        }
    } else {
        Ok(HttpResponse::BadRequest().body("Unhandled event."))
    }
}
