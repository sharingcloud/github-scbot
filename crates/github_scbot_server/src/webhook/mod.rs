//! Webhook handlers.

mod checks;
mod issues;
mod ping;
mod pulls;
mod reviews;

use std::convert::TryFrom;

use actix_web::{error, web, HttpRequest, HttpResponse, Result as ActixResult};
use github_scbot_core::Config;
use github_scbot_database::DbConn;
use github_scbot_types::events::EventType;
use tracing::info;

use crate::{
    constants::GITHUB_EVENT_HEADER,
    errors::{Result, ServerError},
    server::AppContext,
    utils::convert_payload_to_string,
};

async fn parse_event(
    config: &Config,
    conn: &DbConn,
    event_type: EventType,
    body: &str,
) -> Result<HttpResponse> {
    match event_type {
        EventType::CheckRun => {
            checks::check_run_event(
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::CheckSuite => {
            checks::check_suite_event(
                config,
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::IssueComment => {
            issues::issue_comment_event(
                config,
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::Ping => ping::ping_event(
            conn,
            serde_json::from_str(body).map_err(|e| ServerError::EventParseError(event_type, e))?,
        )
        .await
        .map_err(Into::into),
        EventType::PullRequest => {
            pulls::pull_request_event(
                config,
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::PullRequestReview => {
            reviews::review_event(
                config,
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
        EventType::PullRequestReviewComment => {
            reviews::review_comment_event(
                conn,
                serde_json::from_str(body)
                    .map_err(|e| ServerError::EventParseError(event_type, e))?,
            )
            .await
        }
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
    ctx: web::Data<AppContext>,
) -> ActixResult<HttpResponse> {
    // Route event depending on header
    if let Some(event_type) = extract_event_from_request(&req) {
        if let Ok(body) = convert_payload_to_string(&mut payload).await {
            let conn = ctx.pool.get().map_err(error::ErrorInternalServerError)?;
            info!("Incoming event: {:?}", event_type);

            parse_event(&ctx.config, &conn, event_type, &body)
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

/// Configure webhook handlers.
///
/// # Arguments
///
/// * `cfg` - Actix service config
pub fn configure_webhook_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::post().to(event_handler)));
}
