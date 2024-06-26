//! Webhook handlers.

mod checks;
mod issues;
mod ping;
mod pulls;
mod reviews;

#[cfg(test)]
mod tests;

use std::{convert::TryFrom, sync::Arc};

use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use serde::Deserialize;

use self::{
    checks::parse_check_suite_event, issues::parse_issue_comment_event, ping::parse_ping_event,
    pulls::parse_pull_request_event, reviews::parse_review_event,
};
use crate::{
    constants::GITHUB_EVENT_HEADER, event_type::EventType, server::AppContext,
    utils::convert_payload_to_string, Result, ServerError,
};

#[tracing::instrument(skip_all, fields(event_type))]
async fn parse_event(
    ctx: Arc<AppContext>,
    event_type: EventType,
    body: &str,
) -> Result<HttpResponse> {
    match event_type {
        EventType::CheckSuite => {
            checks::check_suite_event(ctx, parse_check_suite_event(body)?).await
        }
        EventType::IssueComment => {
            issues::issue_comment_event(ctx, parse_issue_comment_event(body)?).await
        }
        EventType::Ping => Ok(ping::ping_event(parse_ping_event(body)?)),
        EventType::PullRequest => {
            pulls::pull_request_event(ctx, parse_pull_request_event(body)?).await
        }
        EventType::PullRequestReview => reviews::review_event(ctx, parse_review_event(body)?).await,
    }
}

fn parse_event_type<'de, T>(event_type: EventType, body: &'de str) -> Result<T>
where
    T: Deserialize<'de>,
{
    serde_json::from_str(body).map_err(|e| ServerError::EventParseError {
        event_type,
        source: e,
    })
}

fn extract_event_from_request(req: &HttpRequest) -> Option<EventType> {
    req.headers()
        .get(GITHUB_EVENT_HEADER)
        .and_then(|x| x.to_str().ok())
        .and_then(|x| EventType::try_from(x).ok())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn event_handler(
    req: HttpRequest,
    mut payload: web::Payload,
    ctx: web::Data<AppContext>,
) -> ActixResult<HttpResponse> {
    // Route event depending on header
    if let Some(event_type) = extract_event_from_request(&req) {
        if let Ok(body) = convert_payload_to_string(&mut payload).await {
            parse_event(ctx.into_inner(), event_type, &body)
                .await
                .map_err(Into::into)
        } else {
            let event_type: &str = event_type.into();
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Bad payload for event '{}'.", event_type)
            })))
        }
    } else {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Unhandled event."})))
    }
}

/// Configure webhook handlers.
pub fn configure_webhook_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::post().to(event_handler)));
}
